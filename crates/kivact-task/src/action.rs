//! Role: `Action<T, I>`: a user-triggered async operation with a run policy.
//!
//! `ActionPolicy::Latest` (the default) starts each run at once and cancels
//! the previous one; `Queue` and `QueueLatest` run one at a time on a
//! worker, the second keeping only the newest waiting run. `retry` re-runs
//! the input of the latest run; `on_done` handlers (several) run in the
//! same turn as the publish.
//!
//! Both policies are one implementation on the same `ActionCell`: they
//! differ only in how a run starts (an owner task for `Latest`, the worker
//! for a queue) and in whether disposing the owner detaches the action
//! (only a queue does; it then drops its `on_done` handlers, reports
//! failures to the default sink and drains within 3 s, `drain`). Closing
//! the App (`CloseRequested`) waits for queued actions with the same
//! deadline.
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
//! - Each `on_done` handler runs for the owner that registered it (`context`
//!   and `spawn` inside it use that owner); the registering owner's cleanup
//!   removes it.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::panic::Location;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Waker;
use std::time::Duration;

use super::async_state::{AsyncState, cancelled};
use super::drain::{CloseFuture, DEFAULT_DRAIN_NS};
use super::executor::RawTask;
use super::state::Drain;
use super::state::TaskCell;
use kivact_reactive::{
    AppId, ErrorKind, HandlerResult, Owned, Owner, Runtime, Signal, UiError, require_effectful,
    require_setup,
};

/// How an [`Action`] schedules a new run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActionPolicy {
    /// Start at once; a pending run is cancelled (latest wins). The run is
    /// cancelled when the action's owner is disposed.
    #[default]
    Latest,
    /// Run one at a time in `run` order. A failure stays in the state until
    /// a later run succeeds; later runs still start. After the action's
    /// owner is disposed the waiting runs keep going for up to 3 s, and
    /// closing the App waits for them with the same deadline; leftovers
    /// are reported as `ErrorKind::Closed`.
    Queue,
    /// Like `Queue`, but a new run replaces the waiting one; the running
    /// one is never overtaken.
    QueueLatest,
}

impl ActionPolicy {
    fn queued(self) -> bool {
        !matches!(self, ActionPolicy::Latest)
    }
}

type Runner<T, I> = Rc<dyn Fn(&I) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>>>;
type DoneFn<T> = Rc<dyn Fn(&Result<T, UiError>) -> Result<(), UiError>>;

/// A registered `on_done` handler.
struct Handler<T: 'static> {
    /// Registration id (the registering owner's cleanup removes it).
    id: u64,
    /// The registering owner: the handler runs for it.
    owner: Owner,
    f: DoneFn<T>,
}

pub(crate) struct ActionCell<T: 'static, I: 'static> {
    app: AppId,
    /// The owner the action belongs to; runs are for it.
    owner: Owner,
    run: Runner<T, I>,
    policy: Cell<ActionPolicy>,
    state: Signal<AsyncState<T>>,
    latest: Signal<Option<T>>,
    /// The input and attempt of the run `state` describes.
    input: RefCell<Option<Rc<I>>>,
    attempt: Cell<u32>,
    /// The task of the in-flight `Latest` run.
    task: Cell<Option<Owned<TaskCell>>>,
    generation: Cell<u64>,
    /// Waiting runs of a queue policy (input, attempt).
    waiting: RefCell<VecDeque<(Rc<I>, u32)>>,
    running: Cell<bool>,
    worker: RefCell<Option<RawTask>>,
    closed: Cell<bool>,
    /// The owner was disposed while a queue was draining.
    detached: Cell<bool>,
    drained: RefCell<Vec<Waker>>,
    on_done: RefCell<Vec<Handler<T>>>,
    next_handler: Cell<u64>,
    at: &'static Location<'static>,
}

/// Copy handle of an action; made by [`action`] and started with
/// [`run`](Self::run): `run(())` for an action without input, `run(input)`
/// for one with an input.
pub struct Action<T: 'static, I: 'static = ()> {
    handle: Owned<ActionCell<T, I>>,
}

impl<T: 'static, I: 'static> Clone for Action<T, I> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static, I: 'static> Copy for Action<T, I> {}

/// An async operation started from handlers (Solid `createAction`): a
/// closure with no argument makes an action whose input is `()` (run with
/// `run(())`), a closure with one argument an action run with `run(input)`. By default a new run
/// replaces the previous one ([`Action::policy`]). `run` returns
/// `Result<T, UiError>`; `?` converts any other `std::error::Error`. Owned
/// by the current setup owner; setup only.
///
/// ```ignore
/// let reload = action(|| async { api::reload().await });
/// reload.run(());
/// let save = action(|draft: Draft| async move { api::save(draft).await });
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
pub fn action<T, M, F>(run: F) -> Action<T, F::Input>
where
    T: Clone + 'static,
    F: ActionFn<T, M>,
{
    create("action", Location::caller(), move |i: &F::Input| {
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
pub trait ActionFn<T, M>: crate::sealed::ActionFnSeal<M> + 'static {
    /// What `Action::run` takes: `()` for a closure without an argument.
    type Input: 'static;
    #[doc(hidden)]
    fn start(&self, input: &Self::Input) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>>;
}

impl<F, Fut, T> crate::sealed::ActionFnSeal<crate::sealed::Nullary> for F
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, UiError>>,
{
}

impl<F, Fut, T> ActionFn<T, crate::sealed::Nullary> for F
where
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = Result<T, UiError>> + 'static,
{
    type Input = ();
    fn start(&self, _: &()) -> Pin<Box<dyn Future<Output = Result<T, UiError>>>> {
        Box::pin(self())
    }
}

impl<F, Fut, T, I> crate::sealed::ActionFnSeal<crate::sealed::Unary<I>> for F
where
    F: Fn(I) -> Fut,
    Fut: Future<Output = Result<T, UiError>>,
{
}

impl<F, Fut, T, I> ActionFn<T, crate::sealed::Unary<I>> for F
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
    f: impl Fn(&I) -> Fut + 'static,
) -> Action<T, I>
where
    T: Clone + 'static,
    Fut: Future<Output = Result<T, UiError>> + 'static,
{
    let owner = Owner::current(api, at);
    // The run closure is untracked, wherever `run` is called from.
    let run: Runner<T, I> = Rc::new(move |i| Box::pin(kivact_reactive::untrack(|| f(i))));
    let app = owner.app();
    let cell = Rc::new(ActionCell {
        app,
        owner,
        run,
        policy: Cell::new(ActionPolicy::Latest),
        state: owner.signal(AsyncState::<T>::Idle, at),
        latest: owner.signal(None::<T>, at),
        input: RefCell::new(None),
        attempt: Cell::new(0),
        task: Cell::new(None),
        generation: Cell::new(0),
        waiting: RefCell::new(VecDeque::new()),
        running: Cell::new(false),
        worker: RefCell::new(None),
        closed: Cell::new(false),
        detached: Cell::new(false),
        drained: RefCell::new(Vec::new()),
        on_done: RefCell::new(Vec::new()),
        next_handler: Cell::new(0),
        at,
    });
    let weak = Rc::downgrade(&cell);
    let on_dispose = move |rt: &Runtime| {
        let Some(cell) = weak.upgrade() else { return };
        if !cell.policy.get().queued() {
            return;
        }
        cell.detached.set(true);
        cell.on_done.borrow_mut().clear();
        if cell.busy() {
            let deadline = rt.now_ns().saturating_add(DEFAULT_DRAIN_NS);
            let d: Rc<dyn Drain> = cell;
            super::TaskState::of(rt)
                .draining
                .borrow_mut()
                .push((d, deadline));
        }
    };
    Action {
        handle: owner.store_with("Action", cell, on_dispose, at),
    }
}

impl<T: 'static, I: 'static> Drain for ActionCell<T, I> {
    fn busy(&self) -> bool {
        self.running.get() || !self.waiting.borrow().is_empty() || self.task.get().is_some()
    }
    fn abandon(&self) -> usize {
        let n = self.waiting.borrow().len()
            + usize::from(self.running.get())
            + usize::from(self.task.get().is_some());
        // A newer generation also stops a worker that is polling right now
        // (an `on_done` handler cancelled its own action).
        self.generation.set(self.generation.get() + 1);
        self.waiting.borrow_mut().clear();
        self.running.set(false);
        let w = self.worker.borrow_mut().take();
        drop(w);
        if let Some(t) = self.task.take() {
            t.remove();
        }
        self.wake_drained();
        n
    }
}

impl<T: 'static, I: 'static> ActionCell<T, I> {
    /// Wake the `close` futures waiting for the action to drain.
    fn wake_drained(&self) {
        for w in self.drained.borrow_mut().drain(..) {
            w.wake();
        }
    }
}

impl<T: Clone + 'static, I: 'static> ActionCell<T, I> {
    /// Publish a finished run: the state, the latest value and the
    /// `on_done` handlers (none once a queue is detached; its failure goes
    /// to the default sink instead).
    fn finish(&self, result: Result<T, UiError>) {
        if self.detached.get() {
            if let (Err(e), Some(rt)) = (&result, Runtime::get(self.app)) {
                rt.report(None, e.clone());
            }
            return;
        }
        let next = match &result {
            Ok(v) => {
                let latest = v.clone();
                self.latest.update(move |l| *l = Some(latest));
                AsyncState::Ready(v.clone())
            }
            Err(e) => AsyncState::Failed(e.clone()),
        };
        self.state.update(move |s| *s = next);
        let handlers: Vec<(Owner, DoneFn<T>)> = self
            .on_done
            .borrow()
            .iter()
            .map(|h| (h.owner, h.f.clone()))
            .collect();
        if Runtime::get(self.app).is_none() {
            return;
        }
        for (owner, f) in handlers {
            owner.run_handler(|| {
                if let Err(err) = f(&result) {
                    owner.report(err);
                }
            });
        }
    }

    /// Start a `Latest` run now, replacing the pending one.
    fn start(self: &Rc<Self>, rt: &Runtime, input: Rc<I>, attempt: u32) {
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        if let Some(t) = self.task.take() {
            t.remove();
        }
        *self.input.borrow_mut() = Some(input.clone());
        self.attempt.set(attempt);
        self.state.update(|s| *s = AsyncState::Pending);
        let fut = (self.run)(&input);
        let weak = Rc::downgrade(self);
        let at = self.at;
        let task = super::spawn::spawn_owned(
            rt,
            self.owner,
            async move {
                let result = fut.await;
                let Some(cell) = weak.upgrade() else { return };
                if cell.generation.get() != generation {
                    return;
                }
                cell.task.set(None);
                cell.finish(result);
                cell.wake_drained();
            },
            at,
        );
        self.task.set(Some(task));
    }

    /// Queue a run for the worker.
    fn enqueue(self: &Rc<Self>, rt: &Runtime, input: Rc<I>, attempt: u32) {
        {
            let mut waiting = self.waiting.borrow_mut();
            if self.policy.get() == ActionPolicy::QueueLatest {
                waiting.clear();
            }
            waiting.push_back((input, attempt));
        }
        self.queue_status();
        self.ensure_worker(rt);
    }

    /// `Pending` while a queue has work, unless a failure is still shown
    /// (it stays until a later run succeeds; a cancellation does not).
    /// Always notifies, so `pending_len` readers see each change.
    fn queue_status(&self) {
        if self.detached.get() {
            return;
        }
        let busy = self.running.get() || !self.waiting.borrow().is_empty();
        self.state.update(move |s| {
            let failure = matches!(s, AsyncState::Failed(e) if !e.is_cancelled());
            if busy && !failure {
                *s = AsyncState::Pending;
            }
        });
    }

    fn ensure_worker(self: &Rc<Self>, rt: &Runtime) {
        if self.worker.borrow().is_some() {
            return;
        }
        let weak = Rc::downgrade(self);
        let generation = self.generation.get();
        // The worker runs for the action's owner (see the module doc); once
        // the owner is disposed that owner is dead and owner-relative
        // functions report `Disposed`.
        let task = super::TaskState::of(rt)
            .exec
            .spawn(Some(self.owner.id()), async move {
                loop {
                    let fut = {
                        let Some(cell) = weak.upgrade() else { return };
                        if cell.generation.get() != generation {
                            return;
                        }
                        let next = cell.waiting.borrow_mut().pop_front();
                        let Some((input, attempt)) = next else {
                            cell.running.set(false);
                            cell.worker.borrow_mut().take();
                            cell.queue_status();
                            cell.wake_drained();
                            return;
                        };
                        cell.running.set(true);
                        *cell.input.borrow_mut() = Some(input.clone());
                        cell.attempt.set(attempt);
                        cell.queue_status();
                        (cell.run)(&input)
                    };
                    let result = fut.await;
                    let Some(cell) = weak.upgrade() else { return };
                    if cell.generation.get() != generation {
                        return;
                    }
                    cell.running.set(false);
                    cell.finish(result);
                }
            });
        *self.worker.borrow_mut() = Some(task);
        rt.request_turn();
    }

    /// Start or queue a run, per the policy.
    fn submit(
        self: &Rc<Self>,
        rt: &Runtime,
        input: Rc<I>,
        attempt: u32,
        at: &'static Location<'static>,
    ) {
        if self.closed.get() {
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
        if self.policy.get().queued() {
            self.enqueue(rt, input, attempt);
        } else {
            self.start(rt, input, attempt);
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
        cell.submit(&rt, input, 1, at);
    }

    /// Choose how a new run is scheduled (setup only, before the first
    /// run; the default is [`ActionPolicy::Latest`]).
    ///
    /// ```ignore
    /// let saver = action(|doc: Doc| async move { store(doc).await })
    ///     .policy(ActionPolicy::QueueLatest);
    /// ```
    #[track_caller]
    pub fn policy(self, policy: ActionPolicy) -> Self {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        require_setup("Action::policy");
        cell.policy.set(policy);
        // Queued actions are listed for the exit drain.
        let drain: Rc<dyn Drain> = cell;
        let me = Rc::downgrade(&drain);
        let mut queues = super::TaskState::of(&rt).queues.borrow_mut();
        queues.retain(|w| w.strong_count() > 0 && !w.ptr_eq(&me));
        if policy.queued() {
            queues.push(me);
        }
        self
    }

    /// State of the latest run (tracked). A queue shows `Pending` while it
    /// has work, and a failure until a later run succeeds. A cancelled run
    /// is `Failed` with an error whose `is_cancelled()` is true.
    #[track_caller]
    pub fn state(&self) -> AsyncState<T> {
        self.cell(Location::caller()).1.state.get()
    }

    /// Value of the last successful run, kept while a later one is pending,
    /// failed or cancelled (tracked).
    #[track_caller]
    pub fn latest(&self) -> Option<T> {
        self.cell(Location::caller()).1.latest.get()
    }

    /// Error of the latest run while it is `Failed` (a cancellation
    /// included), `None` otherwise (tracked).
    #[track_caller]
    pub fn error(&self) -> Option<UiError> {
        self.cell(Location::caller())
            .1
            .state
            .with(|state| state.error().cloned())
    }

    /// Attempt number of the latest run: 1 for `run`, one more for each
    /// `retry`; 0 before the first run (tracked).
    #[track_caller]
    pub fn attempt(&self) -> u32 {
        let (_, cell) = self.cell(Location::caller());
        cell.state.with(|_| ());
        cell.attempt.get()
    }

    /// Runs waiting to start (the running one excluded; always 0 for
    /// `Latest`) (tracked).
    #[track_caller]
    pub fn pending_len(&self) -> usize {
        let (_, cell) = self.cell(Location::caller());
        cell.state.with(|_| ());
        cell.waiting.borrow().len()
    }

    /// Run the latest run's input again (a no-op before the first run).
    #[track_caller]
    pub fn retry(&self) {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        require_effectful("Action::retry");
        let input = cell.input.borrow().clone();
        if let Some(i) = input {
            let attempt = cell.attempt.get() + 1;
            cell.submit(&rt, i, attempt, at);
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
            cell.abandon();
            let err = cancelled(at);
            cell.state.update(move |s| *s = AsyncState::Failed(err));
        }
    }

    /// Register a completion handler (setup only; several allowed). It
    /// runs with each finished run's result (not for cancelled ones). It
    /// belongs to the owner whose setup registers it (the nearest flow
    /// branch, list row, `provide` or root, not the action's owner): it runs
    /// for that owner, so `context` and `spawn` inside it use that owner, and
    /// it is removed when that owner is disposed.
    #[track_caller]
    pub fn on_done<R: HandlerResult>(&self, f: impl Fn(&Result<T, UiError>) -> R + 'static) {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        let owner = Owner::current("Action::on_done", at);
        let id = cell.next_handler.get() + 1;
        cell.next_handler.set(id);
        cell.on_done.borrow_mut().push(Handler {
            id,
            owner,
            f: Rc::new(move |result| f(result).into_result()),
        });
        let weak = Rc::downgrade(&cell);
        drop(rt);
        owner.on_cleanup(move || {
            if let Some(c) = weak.upgrade() {
                c.on_done.borrow_mut().retain(|h| h.id != id);
            }
        });
    }

    /// Stop accepting runs and wait until the pending and waiting ones
    /// finish; after `deadline` the rest is abandoned and `Err(Closed)`
    /// reports how many were dropped.
    #[track_caller]
    pub fn close(&self, deadline: Duration) -> impl Future<Output = Result<(), UiError>> + 'static {
        let at = Location::caller();
        let (rt, cell) = self.cell(at);
        require_effectful("Action::close");
        cell.closed.set(true);
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
        cell.state.with(|_| ());
        cell.input.borrow().as_deref().cloned()
    }
}
