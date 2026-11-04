//! Role: UI-thread local executor (async-task).
//!
//! `async_task::spawn_local` on the UI thread, so `!Send` futures run and
//! may capture UI handles; wakes (Send + Sync) schedule runnables from
//! worker threads and wake the driver of the App's turns (the client makes
//! the ready waker from the host's `HostWaker`). Each runnable carries the
//! owner its task runs for (the frame owner of its polls), so `spawn` and
//! `context` after an `.await` use the task's owner.
//!
//! Invariants:
//! - A panic in a poll propagates out of `poll_tasks` and the turn.
//! - When the window is destroyed the owners are disposed, the remaining
//!   queues stop, and cancelled runnables are collected on the UI thread.

use std::collections::VecDeque;
use std::future::Future;
use std::sync::{Arc, Mutex};

use crate::{OwnerId, ThreadWaker};

/// A runnable task; the metadata is the owner the task runs for.
pub(crate) type Runnable = async_task::Runnable<Option<OwnerId>>;

/// The handle of a spawned task (dropping it cancels the future).
pub(crate) type RawTask = async_task::Task<(), Option<OwnerId>>;

#[derive(Clone)]
pub(crate) struct Shared {
    queue: Arc<Mutex<VecDeque<Runnable>>>,
    waker: ThreadWaker,
}

pub(crate) struct Executor {
    shared: Shared,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock()
        .expect("executor lock poisoned: a panic already happened")
}

impl Executor {
    pub(crate) fn new(waker: ThreadWaker) -> Self {
        Executor {
            shared: Shared {
                queue: Arc::new(Mutex::new(VecDeque::new())),
                waker,
            },
        }
    }
    /// Spawn `fut`; its polls run for `owner` (see the module doc).
    pub(crate) fn spawn(
        &self,
        owner: Option<OwnerId>,
        fut: impl Future<Output = ()> + 'static,
    ) -> RawTask {
        let shared = self.shared.clone();
        let schedule = move |r: Runnable| {
            lock(&shared.queue).push_back(r);
            shared.waker.wake();
        };
        let (runnable, task) = async_task::Builder::new()
            .metadata(owner)
            .spawn_local(move |_| fut, schedule);
        runnable.schedule();
        task
    }

    pub(crate) fn has_ready(&self) -> bool {
        !lock(&self.shared.queue).is_empty()
    }

    pub(crate) fn pop(&self) -> Option<Runnable> {
        lock(&self.shared.queue).pop_front()
    }
}
