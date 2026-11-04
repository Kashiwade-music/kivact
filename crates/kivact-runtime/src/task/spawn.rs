//! Role: This module runs owned futures on the UI thread.

use std::cell::OnceCell;
use std::future::Future;
use std::panic::Location;
use std::rc::Rc;

use super::state::{TaskCell, TaskState};

use crate::HandlerResult;
use crate::{AppId, Owned, Owner, Runtime, require_effectful};

/// A spawned future owned by a component; cancelled when the owner is disposed.
#[derive(Clone, Copy)]
pub struct Task {
    app: AppId,
    handle: Owned<Rc<TaskCell>>,
}

impl Task {
    /// Drop the future now (no further polls). A no-op once the task or its
    /// App is gone; not allowed inside memos and render effects.
    #[track_caller]
    pub fn cancel(&self) {
        // A task of another running App is a located `WrongApp`.
        self.handle.find();
        if Runtime::get(self.app).is_some() {
            require_effectful("Task::cancel");
            self.handle.remove();
        }
    }

    /// Whether the task has completed or been cancelled (its owner's
    /// disposal included).
    #[track_caller]
    pub fn is_finished(&self) -> bool {
        self.handle.find().is_none()
    }
}

/// Spawn `fut` as an object of `owner` on `rt`'s executor. A panic in a
/// poll propagates out of the turn.
pub(super) fn spawn_owned(
    rt: &Runtime,
    owner: Owner,
    fut: impl Future<Output = ()> + 'static,
    at: &'static Location<'static>,
) -> Owned<Rc<TaskCell>> {
    let cell = Rc::new(TaskCell {
        task: OnceCell::new(),
    });
    let key = owner.store("Task", cell.clone(), at);
    let app = rt.id();
    let wrapped = async move {
        fut.await;
        let Some(rt) = Runtime::get(app) else { return };
        TaskState::of(&rt).finished.borrow_mut().push(key);
    };
    let task = TaskState::of(rt).exec.spawn(Some(owner.id()), wrapped);
    assert!(
        cell.task.set(task).is_ok(),
        "invariant: a task cell gets its task once"
    );
    rt.request_turn();
    key
}

/// Spawn `fut` as a task of `owner`, on the executor of the owner's App:
/// the way a host or a layer above runs a future for an owner it holds. A
/// panic in a poll propagates out of the turn, and the future is cancelled
/// when `owner` is disposed. `at` is the origin diagnostics name the task by (the
/// call of the user's code that led here). It adds none of the ambient and
/// purity checks of [`spawn`].
pub fn spawn_in(
    owner: Owner,
    fut: impl Future<Output = ()> + 'static,
    at: &'static Location<'static>,
) -> Task {
    let rt = owner.runtime();
    let handle = spawn_owned(&rt, owner, fut, at);
    Task {
        app: rt.id(),
        handle,
    }
}

impl TaskState {
    /// Poll ready tasks of `rt`'s App (untracked, not pure). Returns how
    /// many ran.
    pub fn poll_tasks(&self, rt: &Runtime) -> usize {
        let mut n = 0;
        while let Some(r) = self.exec.pop() {
            {
                let owner = *r.metadata();
                let poll = || {
                    r.run();
                };
                // The poll runs for the task's owner.
                match owner {
                    Some(owner) => rt.owner(owner).run_handler(poll),
                    None => rt.enter(poll),
                }
            }
            n += 1;
            let finished: Vec<Owned<Rc<TaskCell>>> =
                std::mem::take(&mut self.finished.borrow_mut());
            for k in finished {
                k.remove();
            }
            if n >= 100_000 {
                break;
            }
        }
        n
    }
}

/// Run a future on the UI thread, owned by the nearest owner: the setup
/// owner during setup, the target node's owner in an event handler, the
/// task's owner inside a task, the action's owner inside a queued run, the
/// registering owner inside an `on_complete` handler, the owner of a running
/// effect. It is cancelled when that owner is disposed. Every poll of the
/// future is untracked: a `get` in it does not subscribe anything. Not
/// allowed inside memos and render effects, nor outside every owner (located `SetupOnly`),
/// nor for an owner that is disposed (located `Disposed`). A task spawned
/// for an owner being disposed (from a cleanup) is cancelled by the same
/// disposal: it never runs.
///
/// The future may output `()` or `Result<(), UiError>`: an `Err` is reported
/// like an `Err` returned from an event handler, to the nearest `on_error` of
/// the task's owner.
///
/// ```ignore
/// button("Load").on_press(move || {
///     spawn(async move {
///         let page = fetch(url).await;
///         content.set(page);
///     });
/// })
/// ```
#[track_caller]
pub fn spawn<R: HandlerResult>(fut: impl Future<Output = R> + 'static) -> Task {
    let at = Location::caller();
    let owner = Owner::ambient("spawn", at);
    let rt = owner.runtime();
    require_effectful("spawn");
    let app = rt.id();
    let fut = async move {
        if let Err(e) = fut.await.into_outcome().map(|_| ())
            && let Some(rt) = Runtime::get(app)
        {
            rt.report(Some(owner), e);
        }
    };
    spawn_in(owner, fut, at)
}
