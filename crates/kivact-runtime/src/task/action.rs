//! Role: `Action<T, I>`: a user-triggered async operation with a run policy.
//!
//! `action(run)` starts each run at once and cancels the one in flight (the
//! latest input wins). `queued_action(run)` runs its inputs one at a time, in
//! submission order, on a worker. `retry` re-runs the input of the latest
//! run; `on_complete` handlers (several) run in the same turn as the publish.
//!
//! The cell's `Runner` is an owner task for `action` and a worker queue for
//! `queued_action`. Disposing the action's owner cancels every run (queued
//! ones included), drops its `on_complete` handlers, and the action never
//! publishes again. App exit waits for the queued runs
//! of live actions (the exit drain in `host.rs`).
//!
//! Invariants:
//! - A run's task belongs to the action's owner and is cancelled with it. A
//!   queue's worker (each run's synchronous part and every poll of its
//!   future) runs for the action's owner, so `context`, `spawn`,
//!   `platform::window_size` and `platform::copy_text` inside a run behave
//!   as in the owner's handlers.
//! - The run closure and every poll of its future are untracked: what they
//!   read is not a dependency of the effect that called `run`.
//! - Each `on_complete` handler runs for the owner that registered it (`context`
//!   and `spawn` inside it use that owner); the registering owner's cleanup
//!   removes it.

use kivact_types::cell::Generation;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::panic::Location;
use std::pin::Pin;
use std::rc::Rc;

use super::async_state::{AsyncCell, AsyncState, RunInfo, cancelled};
use super::executor::RawTask;
use super::latest::LatestRun;
use super::state::Drain;
use crate::driver::Runtime;
use crate::{AppId, Handler, HandlerResult, Owned, Owner, UiError, require_effectful};

type RunFn<T, I> = Rc<dyn Fn(&I) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>>>;

/// A registered `on_complete` handler.
struct DoneHandler<T: 'static> {
    /// Registration id (the registering owner's cleanup removes it).
    id: Generation,
    /// The registering owner: the handler runs for it.
    owner: Owner,
    f: Handler<Result<T, UiError>>,
}

/// How the runs of an action execute, fixed by its constructor.
enum Runner<I: 'static> {
    /// `action`: one owner task; a new run cancels the one in flight.
    Latest(Rc<LatestRun>),
    /// `queued_action`: one run at a time on a worker task.
    Queue(RefCell<Queue<I>>),
}

/// The worker task of a queue and the runs waiting for it.
struct Queue<I: 'static> {
    worker: Worker,
    waiting: VecDeque<RunInfo<I>>,
    /// Allocates worker ids; the current worker's id lives in `Worker::Picking`/`Running`.
    next_worker_id: Generation,
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

/// Whether the action's owner still shows it. Disposing the owner cancels
/// every run (queued ones included) and the action never publishes again.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Open,
    Disposed,
}

// xtask-allow(state): lifecycle is the only phase (open, or disposed with its owner); on_complete is a registration list and next_handler_id an id allocator, both valid in either phase
pub(crate) struct ActionCell<T: 'static, I: 'static> {
    app: AppId,
    /// The owner the action belongs to; runs are for it.
    owner: Owner,
    run: RunFn<T, I>,
    runner: Runner<I>,
    async_state: AsyncCell<T, I>,
    lifecycle: Cell<Lifecycle>,
    on_complete: RefCell<Vec<DoneHandler<T>>>,
    /// The id the next `on_complete` registration takes.
    next_handler_id: Cell<Generation>,
    at: &'static Location<'static>,
}

/// Copy handle of an action; made by [`action`] or [`queued_action`] and
/// started with [`run`](Self::run): `run(())` for an action without input,
/// `run(input)` for one with an input.
pub struct Action<T: 'static, I: 'static = ()> {
    handle: Owned<Rc<ActionCell<T, I>>>,
}

impl<T: 'static, I: 'static> Clone for Action<T, I> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static, I: 'static> Copy for Action<T, I> {}

/// An async operation started from handlers (Solid `createAction`) whose new
/// run cancels the one in flight: the latest input wins. The closure with no
/// argument makes an action whose input is `()` (run with `run(())`), a
/// closure with one argument an action run with `run(input)`. `run` returns
/// `Result<T, UiError>`; `?` converts any other `std::error::Error`. Owned by
/// the current setup owner; setup only.
///
/// ```ignore
/// let reload = action(|| async { api::reload().await });
/// reload.run(());
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
pub fn action<T, M, F>(run: F) -> Action<T, F::Input>
where
    T: Clone + 'static,
    F: ActionFn<T, M>,
{
    create("action", Location::caller(), false, move |i: &F::Input| {
        run.start(i)
    })
}

/// An async operation whose runs wait for each other and run in submission
/// order. Disposing its owner cancels its runs, the queued ones included
/// (work that must outlive a screen uses an action made in the App's root).
/// App exit waits for the queued runs of live actions. Setup only.
///
/// ```ignore
/// let save = queued_action(|draft: Draft| async move { api::save(draft).await });
/// save.run(draft);
/// ```
#[track_caller]
pub fn queued_action<T, M, F>(run: F) -> Action<T, F::Input>
where
    T: Clone + 'static,
    F: ActionFn<T, M>,
{
    create(
        "queued_action",
        Location::caller(),
        true,
        move |i: &F::Input| run.start(i),
    )
}

/// The run closure of an [`action`]: `Fn() -> Fut` or `Fn(I) -> Fut`, where
/// `Fut` is a future of `Result<T, UiError>`. `M` only tells the two forms
/// apart and is inferred; the trait is sealed.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be the run closure of an `action`",
    note = "pass `|| async {{ .. }}` or `|input: I| async move {{ .. }}` whose future returns `Result<T, UiError>`"
)]
pub trait ActionFn<T, M>: crate::task::sealed::ActionFnSeal<M> + 'static {
    /// What `Action::run` takes: `()` for a closure without an argument.
    type Input: 'static;
    // xtask-allow(api-shape): sealed hidden contract - the type-erased future of an action run closure.
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
    queued: bool,
    f: impl Fn(&I) -> Fut + 'static,
) -> Action<T, I>
where
    T: Clone + 'static,
    Fut: Future<Output = Result<T, UiError>> + 'static,
{
    let owner = Owner::setup(api, at);
    let rt = owner.runtime();
    // The run closure is untracked, wherever `run` is called from.
    let run: RunFn<T, I> = Rc::new(move |i| Box::pin(crate::untrack(|| f(i))));
    let runner = if queued {
        Runner::Queue(RefCell::new(Queue {
            worker: Worker::Stopped,
            waiting: VecDeque::new(),
            next_worker_id: Generation::new(),
        }))
    } else {
        Runner::Latest(LatestRun::new())
    };
    let cell = Rc::new(ActionCell {
        app: owner.app(),
        owner,
        run,
        runner,
        async_state: AsyncCell::new(owner, at),
        lifecycle: Cell::new(Lifecycle::Open),
        on_complete: RefCell::new(Vec::new()),
        next_handler_id: Cell::new(Generation::new()),
        at,
    });
    if queued {
        // Queued actions are listed for the exit drain.
        let drain: Rc<dyn Drain> = cell.clone();
        rt.tasks().queues.borrow_mut().insert(&drain);
    }
    let weak = Rc::downgrade(&cell);
    // Disposing the owner cancels every run of the action, queued ones
    // included, and it publishes nothing afterwards.
    // Work that must outlive a screen uses an action of the App's root owner.
    let on_dispose = move |_rt: &Runtime| {
        let Some(cell) = weak.upgrade() else { return };
        cell.lifecycle.set(Lifecycle::Disposed);
        cell.on_complete.borrow_mut().clear();
        cell.abandon_at(cell.at);
    };
    Action {
        handle: owner.store_with("Action", cell, on_dispose, at),
    }
}

impl<T: Clone + 'static, I: 'static> Drain for ActionCell<T, I> {
    fn busy(&self) -> bool {
        match &self.runner {
            Runner::Latest(run) => run.is_running(),
            Runner::Queue(q) => {
                let q = q.borrow();
                !matches!(q.worker, Worker::Stopped) || !q.waiting.is_empty()
            }
        }
    }

    fn abandon(&self) -> usize {
        self.abandon_at(self.at)
    }
}

impl<T: Clone + 'static, I: 'static> ActionCell<T, I> {
    /// The only way the action writes its public state: nothing once its
    /// owner is disposed.
    fn publish(&self, f: impl FnOnce(&AsyncCell<T, I>)) {
        if self.lifecycle.get() == Lifecycle::Open {
            f(&self.async_state);
        }
    }

    fn waiting(&self) -> usize {
        match &self.runner {
            Runner::Latest(_) => 0,
            Runner::Queue(q) => q.borrow().waiting.len(),
        }
    }

    fn abandon_at(&self, at: &'static Location<'static>) -> usize {
        let n = match &self.runner {
            Runner::Latest(run) => usize::from(run.cancel()),
            Runner::Queue(q) => {
                let mut q = q.borrow_mut();
                let n = q.waiting.len() + usize::from(matches!(q.worker, Worker::Running { .. }));
                q.waiting.clear();
                let worker = std::mem::replace(&mut q.worker, Worker::Stopped);
                drop(q);
                drop(worker);
                n
            }
        };
        if n > 0 {
            self.publish(|state| state.finish(Err(cancelled(at)), 0));
        }
        n
    }

    fn begin_run(&self, run: &RunInfo<I>) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>> {
        let waiting = self.waiting();
        self.publish(|state| state.begin(run.clone(), waiting));
        (self.run)(&run.input)
    }

    fn finish(&self, result: Result<T, UiError>) {
        if self.lifecycle.get() == Lifecycle::Disposed {
            return;
        }
        let waiting = self.waiting();
        let more = matches!(self.runner, Runner::Queue(_)) && self.busy();
        self.publish(|state| {
            state.finish(result.clone(), waiting);
            if more {
                state.pending(waiting);
            }
        });
        let handlers: Vec<(Owner, Handler<Result<T, UiError>>)> = self
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

    /// Start a `Latest` run now, replacing the one in flight.
    fn start_latest(self: &Rc<Self>, rt: &Runtime, run: &Rc<LatestRun>, info: RunInfo<I>) {
        let weak = Rc::downgrade(self);
        run.start(
            rt,
            self.owner,
            self.at,
            || self.begin_run(&info),
            move |result| {
                let Some(cell) = weak.upgrade() else { return };
                cell.finish(result);
            },
        );
    }

    /// Queue a run for the worker.
    fn enqueue(self: &Rc<Self>, rt: &Runtime, info: RunInfo<I>) {
        let Runner::Queue(q) = &self.runner else {
            unreachable!("invariant: only a queued action enqueues")
        };
        q.borrow_mut().waiting.push_back(info);
        let waiting = self.waiting();
        self.publish(|state| state.pending(waiting));
        self.ensure_worker(rt);
    }

    /// The next run for worker `id` (it becomes `Running`), or `None` when
    /// `id` is no longer the worker or nothing waits (the worker stops).
    fn pick(&self, id: Generation) -> Option<RunInfo<I>> {
        let Runner::Queue(q) = &self.runner else {
            return None;
        };
        let mut q = q.borrow_mut();
        let current = match &q.worker {
            Worker::Picking { id: w, .. } | Worker::Running { id: w, .. } => *w == id,
            Worker::Stopped => false,
        };
        if !current {
            return None;
        }
        let Some(info) = q.waiting.pop_front() else {
            // The worker ends: its own task handle goes.
            let worker = std::mem::replace(&mut q.worker, Worker::Stopped);
            drop(q);
            drop(worker);
            return None;
        };
        q.worker = match std::mem::replace(&mut q.worker, Worker::Stopped) {
            Worker::Picking { task, id } | Worker::Running { task, id } => {
                Worker::Running { task, id }
            }
            Worker::Stopped => unreachable!("invariant: checked above"),
        };
        Some(info)
    }

    /// After a run of worker `id`: `None` when `id` is no longer the
    /// worker, else whether more runs wait (if not, the worker stops).
    fn after_run(&self, id: Generation) -> Option<bool> {
        let Runner::Queue(q) = &self.runner else {
            return None;
        };
        let mut q = q.borrow_mut();
        let Worker::Running { id: w, .. } = &q.worker else {
            return None;
        };
        if *w != id {
            return None;
        }
        if q.waiting.is_empty() {
            let worker = std::mem::replace(&mut q.worker, Worker::Stopped);
            drop(q);
            drop(worker);
            return Some(false);
        }
        q.worker = match std::mem::replace(&mut q.worker, Worker::Stopped) {
            Worker::Running { task, id } | Worker::Picking { task, id } => {
                Worker::Picking { task, id }
            }
            Worker::Stopped => unreachable!("invariant: checked above"),
        };
        Some(true)
    }

    fn ensure_worker(self: &Rc<Self>, rt: &Runtime) {
        let Runner::Queue(q) = &self.runner else {
            return;
        };
        let id = {
            let mut q = q.borrow_mut();
            if !matches!(q.worker, Worker::Stopped) {
                return;
            }
            q.next_worker_id.next()
        };
        let weak = Rc::downgrade(self);
        // The worker runs for the action's owner (see the module doc).
        let task = rt.tasks().exec.spawn(Some(self.owner.id()), async move {
            loop {
                let fut = {
                    let Some(cell) = weak.upgrade() else { return };
                    let Some(info) = cell.pick(id) else { return };
                    cell.begin_run(&info)
                };
                let result = fut.await;
                let Some(cell) = weak.upgrade() else { return };
                let Some(more) = cell.after_run(id) else {
                    return;
                };
                cell.finish(result);
                if !more {
                    return;
                }
            }
        });
        q.borrow_mut().worker = Worker::Picking { task, id };
        rt.request_turn();
    }

    fn submit(self: &Rc<Self>, rt: &Runtime, info: RunInfo<I>) {
        match &self.runner {
            Runner::Latest(run) => self.start_latest(rt, run, info),
            Runner::Queue(_) => self.enqueue(rt, info),
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
        cell.submit(&rt, RunInfo { input, attempt: 1 });
    }

    /// The state of the run the action describes (tracked).
    #[track_caller]
    pub fn state(&self) -> AsyncState<T> {
        self.cell(Location::caller()).1.async_state.state()
    }

    /// The last successful value, kept while a new run is pending or failed.
    #[track_caller]
    pub fn latest(&self) -> Option<T> {
        self.cell(Location::caller()).1.async_state.latest()
    }

    /// The error of the last run, if it failed.
    #[track_caller]
    pub fn error(&self) -> Option<UiError> {
        self.cell(Location::caller()).1.async_state.error()
    }

    /// The attempt number of the run `state` describes (0 before the first).
    #[track_caller]
    pub fn attempt(&self) -> u32 {
        self.cell(Location::caller()).1.async_state.attempt()
    }

    /// Runs waiting behind the running one (`queued_action`; 0 otherwise).
    #[track_caller]
    pub fn pending_len(&self) -> usize {
        self.cell(Location::caller()).1.async_state.waiting()
    }

    /// Run the last input again, as the next attempt.
    #[track_caller]
    pub fn retry(&self) {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        require_effectful("Action::retry");
        if let Some(last) = cell.async_state.run_untracked() {
            cell.submit(
                &rt,
                RunInfo {
                    input: last.input,
                    attempt: last.attempt + 1,
                },
            );
        }
    }

    /// Cancel the running run and the waiting ones; the state becomes
    /// `Failed` with a cancellation error and `on_complete` is not called.
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

    /// Call `f` with the result of each run that ends (not for a
    /// cancellation). Several handlers run in registration order. Setup only.
    #[track_caller]
    pub fn on_complete<R: HandlerResult>(
        self,
        f: impl Fn(Result<T, UiError>) -> R + 'static,
    ) -> Self {
        let at = Location::caller();
        let (_, cell) = self.cell(at);
        let owner = Owner::setup("Action::on_complete", at);
        let mut generation = cell.next_handler_id.get();
        let id = generation.next();
        cell.next_handler_id.set(generation);
        cell.on_complete.borrow_mut().push(DoneHandler {
            id,
            owner,
            f: Handler::from_fn(f),
        });
        let weak = Rc::downgrade(&cell);
        owner.on_cleanup(move || {
            if let Some(c) = weak.upgrade() {
                c.on_complete.borrow_mut().retain(|h| h.id != id);
            }
        });
        self
    }

    pub(super) fn publications(&self) -> u64 {
        self.cell(Location::caller()).1.async_state.publications()
    }
}

impl<T: Clone + 'static, I: Clone + 'static> Action<T, I> {
    /// Start a run with `input` (`()` for an action without input).
    #[track_caller]
    pub fn run(&self, input: I) {
        self.submit(Rc::new(input), "Action::run");
    }

    /// The input of the run `state` describes.
    #[track_caller]
    pub fn input(&self) -> Option<I> {
        self.cell(Location::caller()).1.async_state.input()
    }
}
