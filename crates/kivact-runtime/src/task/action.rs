//! Role: `Action<T, I>`: a user-triggered async operation with a run policy.
//!
//! `action(policy, run)` fixes how runs execute: `ActionMode::Latest` starts
//! each run at once and cancels the previous one; `Queue` and `QueueLatest`
//! run one at a time on a worker, the second keeping only the newest waiting
//! run. `retry` re-runs the input of the latest run; `on_complete` handlers
//! (several) run in the same turn as the publish.
//!
//! The policy chooses the cell's `Runner` (an owner task for `Latest`, a
//! worker queue otherwise); only a queue is detached when the owner is
//! disposed (it then drops its `on_complete` handlers, reports failures to the
//! default sink and drains within 3 s, `drain`). Closing the App
//! (`CloseRequested`) waits for queued actions with the same deadline.
//!
//! Invariants:
//! - A `Latest` run's task belongs to the action's owner and is cancelled
//!   with it. A queue's worker (each run's synchronous part and every poll
//!   of its future) runs for the action's owner, so `context`, `spawn`,
//!   `platform::window_size` and `platform::copy_text` inside a run behave
//!   as in the owner's handlers. After the owner is disposed a queue is detached and
//!   drains for the same, now disposed, owner (`drain`): those functions
//!   then raise a located `Disposed` instead of attaching anything to an
//!   owner that is gone.
//! - The run closure and every poll of its future are untracked: what they
//!   read is not a dependency of the effect that called `run`.
//! - Each `on_complete` handler runs for the owner that registered it (`context`
//!   and `spawn` inside it use that owner); the registering owner's cleanup
//!   removes it.

use kivact_types::cell::Generation;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::panic::Location;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Waker;
use std::time::Duration;

use super::async_state::{AsyncCell, AsyncState, cancelled};
use super::drain::{CloseFuture, DEFAULT_DRAIN_NS};
use super::executor::RawTask;
use super::latest::LatestRun;
use super::state::Drain;
use crate::{AppId, ErrorKind, HandlerResult, Owned, Owner, Runtime, UiError, require_effectful};

/// How an [`Action`] schedules a new run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionMode {
    /// Start at once; a pending run is cancelled (latest wins). The run is
    /// cancelled when the action's owner is disposed.
    Latest,
    /// Run one at a time in `run` order. State stays pending while work remains;
    /// each completion is delivered to its handlers. After the action's
    /// owner is disposed the waiting runs keep going for up to 3 s, and
    /// closing the App waits for them with the same deadline; leftovers
    /// are reported as `ErrorKind::Closed`.
    Queue,
    /// Like `Queue`, but a new run replaces the waiting one; the running
    /// one is never overtaken.
    QueueLatest,
}

type RunFn<T, I> = Rc<dyn Fn(&I) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>>>;
type DoneFn<T> = crate::Handler<Result<T, UiError>>;

/// A registered `on_complete` handler.
struct DoneHandler<T: 'static> {
    /// Registration id (the registering owner's cleanup removes it).
    id: Generation,
    /// The registering owner: the handler runs for it.
    owner: Owner,
    f: DoneFn<T>,
}

/// A run's input and attempt (1 for `run`, one more per `retry`).
struct Submitted<I> {
    input: Rc<I>,
    attempt: u32,
}

impl<I> Clone for Submitted<I> {
    fn clone(&self) -> Self {
        Submitted {
            input: self.input.clone(),
            attempt: self.attempt,
        }
    }
}

/// How the runs of an action execute, fixed by its `ActionMode`.
enum Runner<I: 'static> {
    /// `Latest`: one owner task; a new run cancels the one in flight.
    Latest(Rc<LatestRun>, Cell<Acceptance>),
    /// `Queue` and `QueueLatest`: one run at a time on a worker task.
    Queue(Queue<I>),
}

/// Which waiting runs a queue keeps when a new run arrives.
#[derive(Clone, Copy, PartialEq, Eq)]
enum QueueMode {
    /// `Queue`: every run, in `run` order.
    Fifo,
    /// `QueueLatest`: only the newest waiting run.
    Newest,
}

struct Queue<I: 'static> {
    mode: QueueMode,
    phase: RefCell<ActionPhase<I>>,
    /// Id of the last worker spawned: a worker whose id is not current returns.
    worker_ids: Cell<Generation>,
}

/// Queue ownership and lifecycle retain work together through close and detach.
enum ActionPhase<I> {
    Open {
        worker: Worker,
        waiting: VecDeque<Submitted<I>>,
    },
    Closed {
        worker: Worker,
        waiting: VecDeque<Submitted<I>>,
    },
    Detached(QueueDrain<I>),
}
struct QueueDrain<I> {
    worker: Worker,
    waiting: VecDeque<Submitted<I>>,
}
impl<I> ActionPhase<I> {
    fn parts(&self) -> (&Worker, &VecDeque<Submitted<I>>) {
        match self {
            Self::Open { worker, waiting } | Self::Closed { worker, waiting } => (worker, waiting),
            Self::Detached(drain) => (&drain.worker, &drain.waiting),
        }
    }
    fn parts_mut(&mut self) -> (&mut Worker, &mut VecDeque<Submitted<I>>) {
        match self {
            Self::Open { worker, waiting } | Self::Closed { worker, waiting } => (worker, waiting),
            Self::Detached(drain) => (&mut drain.worker, &mut drain.waiting),
        }
    }
}

/// The worker task of a queue.
enum Worker {
    /// No worker task.
    Stopped,
    /// A worker task that takes the next waiting run when it is polled.
    Picking { task: RawTask, id: Generation },
    /// A worker task awaiting a run's future.
    Running { task: RawTask, id: Generation },
}

impl<I: 'static> Queue<I> {
    fn new(mode: QueueMode) -> Self {
        Queue {
            mode,
            phase: RefCell::new(ActionPhase::Open {
                worker: Worker::Stopped,
                waiting: VecDeque::new(),
            }),
            worker_ids: Cell::new(Generation::new()),
        }
    }

    fn replace_worker(&self, next: Worker) -> Worker {
        std::mem::replace(self.phase.borrow_mut().parts_mut().0, next)
    }
    fn close(&self) {
        let mut phase = self.phase.borrow_mut();
        if matches!(*phase, ActionPhase::Open { .. }) {
            let (worker, waiting) = phase.parts_mut();
            let next = ActionPhase::Closed {
                worker: std::mem::replace(worker, Worker::Stopped),
                waiting: std::mem::take(waiting),
            };
            *phase = next;
        }
    }
    fn detach(&self) {
        let mut phase = self.phase.borrow_mut();
        let (worker, waiting) = phase.parts_mut();
        let drain = QueueDrain {
            worker: std::mem::replace(worker, Worker::Stopped),
            waiting: std::mem::take(waiting),
        };
        *phase = ActionPhase::Detached(drain);
    }

    /// A run is running or waiting.
    fn busy(&self) -> bool {
        matches!(
            *self.phase.borrow().parts().0,
            Worker::Running { .. } | Worker::Picking { .. }
        ) || !self.phase.borrow().parts().1.is_empty()
    }

    /// Whether worker `id` is the current worker.
    fn is_worker(&self, id: Generation) -> bool {
        match self.phase.borrow().parts().0 {
            Worker::Picking { id: w, .. } | Worker::Running { id: w, .. } => *w == id,
            Worker::Stopped => false,
        }
    }

    /// The current worker starts (`true`) or ends (`false`) a run.
    fn set_running(&self, running: bool) {
        let mut phase = self.phase.borrow_mut();
        let (worker, _) = phase.parts_mut();
        *worker = match std::mem::replace(worker, Worker::Stopped) {
            Worker::Picking { task, id } | Worker::Running { task, id } => {
                if running {
                    Worker::Running { task, id }
                } else {
                    Worker::Picking { task, id }
                }
            }
            Worker::Stopped => unreachable!("invariant: only the current worker marks its runs"),
        };
    }

    /// Drop the waiting runs and the worker; returns how many runs were dropped.
    fn abandon(&self) -> usize {
        let running = matches!(*self.phase.borrow().parts().0, Worker::Running { .. });
        let n = self.phase.borrow().parts().1.len() + usize::from(running);
        self.phase.borrow_mut().parts_mut().1.clear();
        // Dropping the task stops the worker; one polling right now sees it is
        // not current and returns.
        drop(self.replace_worker(Worker::Stopped));
        n
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Acceptance {
    Open,
    Closed,
}

pub(crate) struct ActionCell<T: 'static, I: 'static> {
    app: AppId,
    /// The owner the action belongs to; runs are for it.
    owner: Owner,
    run: RunFn<T, I>,
    runner: Runner<I>,
    async_state: AsyncCell<T>,
    /// The run `state` describes; `None` before the first run.
    last: RefCell<Option<Submitted<I>>>,
    /// Wakers of the `close` futures waiting for the action to drain.
    drained: RefCell<Vec<Waker>>,
    on_complete: RefCell<Vec<DoneHandler<T>>>,
    next_handler: Cell<Generation>,
    at: &'static Location<'static>,
}

/// Copy handle of an action; made by [`action`] and started with
/// [`run`](Self::run): `run(())` for an action without input, `run(input)`
/// for one with an input.
pub struct Action<T: 'static, I: 'static = ()> {
    handle: Owned<Rc<ActionCell<T, I>>>,
}

impl<T: 'static, I: 'static> Clone for Action<T, I> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static, I: 'static> Copy for Action<T, I> {}

/// An async operation started from handlers (Solid `createAction`). `policy`
/// says what a new `run` does while one is still going ([`ActionMode`]); the
/// closure with no argument makes an action whose input is `()` (run with
/// `run(())`), a closure with one argument an action run with `run(input)`.
/// `run` returns `Result<T, UiError>`; `?` converts any other
/// `std::error::Error`. Owned by the current setup owner; setup only.
///
/// ```ignore
/// let reload = action(ActionMode::Latest, || async { api::reload().await });
/// reload.run(());
/// let save = action(ActionMode::Queue, |draft: Draft| async move { api::save(draft).await });
/// save.run(draft);
/// ```
///
/// The run closure and the future it returns run untracked: a read in them
/// is not a dependency of the effect or handler that called `run`.
///
/// `run` is one method for both forms, so an input type that only the call
/// to `run` decides is inferred from it. A closure that calls a method on
/// its argument still needs the argument's type (`|n: u32| ..`): Rust checks
/// the closure before it sees `run`.
#[track_caller]
pub fn action<T, M, F>(policy: ActionMode, run: F) -> Action<T, F::Input>
where
    T: Clone + 'static,
    F: ActionFn<T, M>,
{
    create("action", Location::caller(), policy, move |i: &F::Input| {
        run.start(i)
    })
}

/// The run closure of an [`action`]: `Fn() -> Fut` or `Fn(I) -> Fut`, where
/// `Fut` is a future of `Result<T, UiError>`. `M` only tells the two forms
/// apart and is inferred; the trait is sealed.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be the run closure of an `action`",
    note = "pass `|| async {{ .. }}` or `|input: I| async move {{ .. }}` whose future returns `Result<T, UiError>`"
)]
pub trait ActionFn<T, M>: crate::task::sealed::ActionFnSeal<M> + 'static {
    /// What `Action::run` takes: `()` for a closure without an argument.
    type Input: 'static;
    #[doc(hidden)]
    fn start(&self, input: &Self::Input) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>>;
}

impl<F, Fut, T> crate::task::sealed::ActionFnSeal<crate::task::sealed::Nullary> for F
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, UiError>>,
{
}

impl<F, Fut, T> ActionFn<T, crate::task::sealed::Nullary> for F
where
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = Result<T, UiError>> + 'static,
{
    type Input = ();
    fn start(&self, _: &()) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>> {
        Box::pin(self())
    }
}

impl<F, Fut, T, I> crate::task::sealed::ActionFnSeal<crate::task::sealed::Unary<I>> for F
where
    F: Fn(I) -> Fut,
    Fut: Future<Output = Result<T, UiError>>,
{
}

impl<F, Fut, T, I> ActionFn<T, crate::task::sealed::Unary<I>> for F
where
    F: Fn(I) -> Fut + 'static,
    Fut: Future<Output = Result<T, UiError>> + 'static,
    I: Clone + 'static,
{
    type Input = I;
    fn start(&self, input: &I) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>> {
        Box::pin(self(input.clone()))
    }
}

fn create<T, I, Fut>(
    api: &'static str,
    at: &'static Location<'static>,
    policy: ActionMode,
    f: impl Fn(&I) -> Fut + 'static,
) -> Action<T, I>
where
    T: Clone + 'static,
    Fut: Future<Output = Result<T, UiError>> + 'static,
{
    let owner = Owner::current(api, at);
    let rt = owner.runtime();
    // The run closure is untracked, wherever `run` is called from.
    let run: RunFn<T, I> = Rc::new(move |i| Box::pin(crate::untrack(|| f(i))));
    let runner = match policy {
        ActionMode::Latest => Runner::Latest(LatestRun::new(), Cell::new(Acceptance::Open)),
        ActionMode::Queue => Runner::Queue(Queue::new(QueueMode::Fifo)),
        ActionMode::QueueLatest => Runner::Queue(Queue::new(QueueMode::Newest)),
    };
    let cell = Rc::new(ActionCell {
        app: owner.app(),
        owner,
        run,
        runner,
        async_state: AsyncCell::new(owner, at),
        last: RefCell::new(None),
        drained: RefCell::new(Vec::new()),
        on_complete: RefCell::new(Vec::new()),
        next_handler: Cell::new(Generation::new()),
        at,
    });
    if matches!(cell.runner, Runner::Queue(_)) {
        // Queued actions are listed for the exit drain.
        let drain: Rc<dyn Drain> = cell.clone();
        let mut queues = super::TaskState::of(&rt).queues.borrow_mut();
        queues.insert(&drain);
    }
    let weak = Rc::downgrade(&cell);
    let on_dispose = move |rt: &Runtime| {
        let Some(cell) = weak.upgrade() else { return };
        // A `Latest` run is a task of the owner: the same disposal cancels it.
        if !matches!(cell.runner, Runner::Queue(_)) {
            return;
        }
        cell.queue().detach();
        cell.on_complete.borrow_mut().clear();
        if cell.busy() {
            let deadline = rt.now_ns().saturating_add(DEFAULT_DRAIN_NS);
            let d: Rc<dyn Drain> = cell;
            super::TaskState::of(rt)
                .draining
                .borrow_mut()
                .push(super::state::DrainEntry::new(d, deadline));
        }
    };
    Action {
        handle: owner.store_with("Action", cell, on_dispose, at),
    }
}

impl<T: Clone + 'static, I: 'static> Drain for ActionCell<T, I> {
    fn busy(&self) -> bool {
        match &self.runner {
            Runner::Latest(run, _) => run.is_running(),
            Runner::Queue(queue) => queue.busy(),
        }
    }

    fn abandon(&self) -> usize {
        self.abandon_at(self.at)
    }
}

impl<T: Clone + 'static, I: 'static> ActionCell<T, I> {
    /// Cancel the pending runs; the state becomes `Failed` with one
    /// cancellation error located at `at`.
    fn abandon_at(&self, at: &'static Location<'static>) -> usize {
        let n = match &self.runner {
            Runner::Latest(run, _) => usize::from(run.cancel()),
            Runner::Queue(queue) => queue.abandon(),
        };
        if n > 0 && !self.is_detached() {
            self.async_state.finish(Err(cancelled(at)));
        }
        self.wake_drained();
        n
    }
}

impl<T: 'static, I: 'static> ActionCell<T, I> {
    fn is_open(&self) -> bool {
        match &self.runner {
            Runner::Latest(_, acceptance) => acceptance.get() == Acceptance::Open,
            Runner::Queue(queue) => matches!(*queue.phase.borrow(), ActionPhase::Open { .. }),
        }
    }
    fn is_detached(&self) -> bool {
        match &self.runner {
            Runner::Latest(..) => false,
            Runner::Queue(queue) => matches!(*queue.phase.borrow(), ActionPhase::Detached(_)),
        }
    }
    /// Wake the `close` futures waiting for the action to drain.
    fn wake_drained(&self) {
        for w in self.drained.borrow_mut().drain(..) {
            w.wake();
        }
    }

    /// The queue of a queued action.
    fn queue(&self) -> &Queue<I> {
        match &self.runner {
            Runner::Queue(queue) => queue,
            Runner::Latest(..) => unreachable!("invariant: only a queued action has a worker"),
        }
    }
}

impl<T: Clone + 'static, I: 'static> ActionCell<T, I> {
    /// Publish a finished run: the state, the latest value and the
    /// `on_complete` handlers (none once a queue is detached; its failure goes
    /// to the default sink instead).
    fn finish(&self, result: Result<T, UiError>) {
        if self.is_detached() {
            if let (Err(e), Some(rt)) = (&result, Runtime::get(self.app)) {
                rt.report(None, e.clone());
            }
            return;
        }
        self.async_state.finish(result.clone());
        if matches!(&self.runner, Runner::Queue(queue) if queue.busy()) {
            self.async_state.start();
        }
        let handlers: Vec<(Owner, DoneFn<T>)> = self
            .on_complete
            .borrow()
            .iter()
            .map(|h| (h.owner, h.f.clone()))
            .collect();
        if Runtime::get(self.app).is_none() {
            return;
        }
        for (owner, f) in handlers {
            owner.run_handler(|| {
                if let Err(err) = f.call(result.clone()) {
                    owner.report(err);
                }
            });
        }
    }

    /// Start a `Latest` run now, replacing the pending one.
    fn start_latest(self: &Rc<Self>, rt: &Runtime, run: &Rc<LatestRun>, submitted: Submitted<I>) {
        let weak = Rc::downgrade(self);
        run.start(
            rt,
            self.owner,
            self.at,
            || {
                *self.last.borrow_mut() = Some(submitted.clone());
                self.async_state.start();
                (self.run)(&submitted.input)
            },
            move |result| {
                let Some(cell) = weak.upgrade() else { return };
                cell.finish(result);
                cell.wake_drained();
            },
        );
    }

    /// Queue a run for the worker.
    fn enqueue(self: &Rc<Self>, rt: &Runtime, submitted: Submitted<I>) {
        let queue = self.queue();
        {
            let mut phase = queue.phase.borrow_mut();
            let (_, waiting) = phase.parts_mut();
            if queue.mode == QueueMode::Newest {
                waiting.clear();
            }
            waiting.push_back(submitted);
        }
        self.queue_status();
        self.ensure_worker(rt);
    }

    /// Keep the public pending state synchronized with queued work.
    /// Always notify so `pending_len` readers see each change.
    fn queue_status(&self) {
        if self.is_detached() {
            return;
        }
        let busy = self.queue().busy();
        if busy {
            self.async_state.start();
        }
    }

    fn ensure_worker(self: &Rc<Self>, rt: &Runtime) {
        let queue = self.queue();
        if !matches!(*queue.phase.borrow().parts().0, Worker::Stopped) {
            return;
        }
        let mut generation = queue.worker_ids.get();
        let id = generation.next();
        queue.worker_ids.set(generation);
        let weak = Rc::downgrade(self);
        // The worker runs for the action's owner (see the module doc); once
        // the owner is disposed that owner is dead and owner-relative
        // functions report `Disposed`.
        let task = super::TaskState::of(rt)
            .exec
            .spawn(Some(self.owner.id()), async move {
                loop {
                    let fut = {
                        let Some(cell) = weak.upgrade() else { return };
                        let queue = cell.queue();
                        if !queue.is_worker(id) {
                            return;
                        }
                        let next = queue.phase.borrow_mut().parts_mut().1.pop_front();
                        let Some(submitted) = next else {
                            // The worker ends: its own task handle goes.
                            drop(queue.replace_worker(Worker::Stopped));
                            cell.queue_status();
                            cell.wake_drained();
                            return;
                        };
                        queue.set_running(true);
                        *cell.last.borrow_mut() = Some(submitted.clone());
                        cell.queue_status();
                        (cell.run)(&submitted.input)
                    };
                    let result = fut.await;
                    let Some(cell) = weak.upgrade() else { return };
                    let queue = cell.queue();
                    if !queue.is_worker(id) {
                        return;
                    }
                    let has_waiting = !queue.phase.borrow().parts().1.is_empty();
                    if has_waiting {
                        queue.set_running(false);
                    } else {
                        drop(queue.replace_worker(Worker::Stopped));
                    }
                    cell.finish(result);
                    if has_waiting {
                        cell.queue_status();
                    } else {
                        cell.wake_drained();
                        return;
                    }
                }
            });
        queue.replace_worker(Worker::Picking { task, id });
        rt.request_turn();
    }

    /// Start or queue a run, per the policy.
    fn submit(
        self: &Rc<Self>,
        rt: &Runtime,
        submitted: Submitted<I>,
        at: &'static Location<'static>,
    ) {
        if !self.is_open() {
            rt.report(
                None,
                UiError::new(
                    ErrorKind::Closed,
                    "run of a closed action ignored",
                    Some(at),
                ),
            );
            return;
        }
        match &self.runner {
            Runner::Latest(run, _) => self.start_latest(rt, run, submitted),
            Runner::Queue(_) => self.enqueue(rt, submitted),
        }
    }
}

impl<T: Clone + 'static, I: 'static> Action<T, I> {
    fn cell(&self, at: &'static Location<'static>) -> (Rc<Runtime>, Rc<ActionCell<T, I>>) {
        let c = self.handle.get_at(at);
        (c.owner.runtime(), c)
    }

    #[track_caller]
    fn submit(&self, input: Rc<I>, api: &'static str) {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        require_effectful(api);
        cell.submit(&rt, Submitted { input, attempt: 1 }, at);
    }

    /// State of the latest run (tracked). A queue shows `Pending` while it
    /// has work, and its final result once drained. A cancelled run
    /// is `Failed` with an error whose `is_cancelled()` is true.
    #[track_caller]
    pub fn state(&self) -> AsyncState<T> {
        self.cell(Location::caller())
            .1
            .async_state
            .state
            .get()
            .as_ref()
            .clone()
    }

    pub(super) fn snapshot(&self) -> super::async_publication::AsyncSnapshot<T> {
        super::async_publication::AsyncSnapshot(
            self.cell(Location::caller()).1.async_state.state.get(),
        )
    }

    /// Value of the last successful run, kept while a later one is pending,
    /// failed or cancelled (tracked).
    #[track_caller]
    pub fn latest(&self) -> Option<T> {
        self.cell(Location::caller()).1.async_state.latest.get()
    }

    /// Error of the latest run while it is `Failed` (a cancellation
    /// included), `None` otherwise (tracked).
    #[track_caller]
    pub fn error(&self) -> Option<UiError> {
        self.cell(Location::caller())
            .1
            .async_state
            .state
            .with(|state| state.error().cloned())
    }

    /// Attempt number of the latest run: 1 for `run`, one more for each
    /// `retry`; 0 before the first run (tracked).
    #[track_caller]
    pub fn attempt(&self) -> u32 {
        let (_, cell) = self.cell(Location::caller());
        cell.async_state.state.with(|_| ());
        cell.last.borrow().as_ref().map_or(0, |s| s.attempt)
    }

    /// Runs waiting to start (the running one excluded; always 0 for
    /// `Latest`) (tracked).
    #[track_caller]
    pub fn pending_len(&self) -> usize {
        let (_, cell) = self.cell(Location::caller());
        cell.async_state.state.with(|_| ());
        match &cell.runner {
            Runner::Latest(..) => 0,
            Runner::Queue(queue) => queue.phase.borrow().parts().1.len(),
        }
    }

    /// Run the latest run's input again (a no-op before the first run).
    #[track_caller]
    pub fn retry(&self) {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        require_effectful("Action::retry");
        let last = cell.last.borrow().clone();
        if let Some(s) = last {
            cell.submit(
                &rt,
                Submitted {
                    input: s.input,
                    attempt: s.attempt + 1,
                },
                at,
            );
        }
    }

    /// Cancel the pending run, and for a queue the waiting ones too: the
    /// state becomes `Failed` with a cancellation error. A no-op when
    /// nothing is pending.
    #[track_caller]
    pub fn cancel(&self) {
        let at = Location::caller();
        let (_, cell) = self.cell(at);
        // Checked before any state changes.
        require_effectful("Action::cancel");
        if cell.busy() {
            cell.abandon_at(at);
        }
    }

    /// Register a completion handler (setup only; several allowed). It
    /// runs with each finished run's result (not for cancelled ones). It
    /// belongs to the owner whose setup registers it (the nearest flow
    /// branch, list row, `provide` or root, not the action's owner): it runs
    /// for that owner, so `context` and `spawn` inside it use that owner, and
    /// it is removed when that owner is disposed.
    #[track_caller]
    pub fn on_complete<R: HandlerResult>(
        self,
        f: impl Fn(ActionOutcome<T>) -> R + 'static,
    ) -> Self {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        let owner = Owner::current("Action::on_complete", at);
        let mut generation = cell.next_handler.get();
        let id = generation.next();
        cell.next_handler.set(generation);
        cell.on_complete.borrow_mut().push(DoneHandler {
            id,
            owner,
            f: crate::Handler::from_fn(move |result: Result<T, UiError>| {
                f(match result {
                    Ok(value) => ActionOutcome::Succeeded(value),
                    Err(error) => ActionOutcome::Failed(error),
                })
            }),
        });
        let weak = Rc::downgrade(&cell);
        drop(rt);
        owner.on_cleanup(move || {
            if let Some(c) = weak.upgrade() {
                c.on_complete.borrow_mut().retain(|h| h.id != id);
            }
        });
        self
    }

    /// Stop accepting runs and wait until the pending and waiting ones
    /// finish; after `deadline` (counted from this call) the rest is abandoned and `Err(Closed)`
    /// reports how many were dropped.
    #[track_caller]
    pub fn close(&self, deadline: Duration) -> impl Future<Output = Result<(), UiError>> + 'static {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        require_effectful("Action::close");
        match &cell.runner {
            Runner::Latest(_, acceptance) => acceptance.set(Acceptance::Closed),
            Runner::Queue(queue) => queue.close(),
        }
        let d: Rc<dyn Drain> = cell.clone();
        let waiters = Rc::downgrade(&cell);
        CloseFuture {
            action: Rc::downgrade(&d),
            register: Box::new(move |w: &Waker| {
                if let Some(c) = waiters.upgrade() {
                    c.drained.borrow_mut().push(w.clone());
                }
            }),
            sleep: Box::pin(super::time::sleep_ns(
                &rt,
                deadline.as_nanos().min(u64::MAX as u128) as u64,
                at,
                Some(cell.owner.id()),
            )),
        }
    }
}

impl<T: Clone + 'static, I: Clone + 'static> Action<T, I> {
    /// Run with `input` (`()` for an action without input), per the policy.
    #[track_caller]
    pub fn run(&self, input: I) {
        self.submit(Rc::new(input), "Action::run");
    }

    /// Input of the latest run; `None` before the first run (tracked).
    #[track_caller]
    pub fn input(&self) -> Option<I> {
        let (_, cell) = self.cell(Location::caller());
        cell.async_state.state.with(|_| ());
        cell.last.borrow().as_ref().map(|s| I::clone(&s.input))
    }
}

/// A naturally completed action run. Cancelled runs do not notify handlers.
#[derive(Clone, Debug)]
pub enum ActionOutcome<T> {
    /// The run succeeded.
    Succeeded(T),
    /// The run failed.
    Failed(UiError),
}
