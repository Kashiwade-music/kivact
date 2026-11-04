//! Role: Blocking operations with cancellation support.
//!
//! `blocking`: run a `Send` closure on the App-owned fixed thread pool and
//! await its result on the UI thread. Dropping the future discards the
//! result and signals the closure's `CancelToken`. A job that panics is
//! noticed by the awaiting future, which takes the payload from the dead
//! thread and resumes it on the UI thread.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use crate::task::worker_pool::{Job, Pool, lock};

struct Slot<R> {
    outcome: Outcome<R>,
    waker: Option<Waker>,
}

/// How a blocking job went.
enum Outcome<R> {
    /// Not finished yet.
    Running,
    /// Finished with its result.
    Done(R),
    /// The job's thread unwound out of it; the thread's id finds its payload.
    Died(std::thread::ThreadId),
    /// The awaiting future returned the result.
    Taken,
}

/// Future of a blocking job.
pub(crate) struct BlockingFuture<R> {
    slot: Arc<Mutex<Slot<R>>>,
    cancel: CancelToken,
    pool: Option<Arc<Pool>>,
}

/// Held by a running job: on drop, at a normal exit and on unwind alike, it
/// settles the slot, wakes the awaiting task and stops counting the job as
/// outstanding.
struct Completion<R> {
    slot: Arc<Mutex<Slot<R>>>,
    outstanding: Option<Arc<AtomicUsize>>,
}

impl<R> Drop for Completion<R> {
    fn drop(&mut self) {
        let waker = {
            let mut s = lock(&self.slot);
            if matches!(s.outcome, Outcome::Running) {
                s.outcome = Outcome::Died(std::thread::current().id());
            }
            s.waker.take()
        };
        // Wake (schedule the awaiting task) before the job stops counting as
        // outstanding: a test harness that sees `outstanding == 0` then also
        // sees the ready task, so it cannot stop before the result is
        // consumed (this once made a history test flaky).
        if let Some(w) = waker {
            w.wake();
        }
        if let Some(o) = &self.outstanding {
            o.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

impl<R> Future for BlockingFuture<R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<R> {
        let mut s = lock(&self.slot);
        match std::mem::replace(&mut s.outcome, Outcome::Taken) {
            Outcome::Done(v) => Poll::Ready(v),
            Outcome::Died(thread) => {
                drop(s);
                let payload = self.pool.as_ref().and_then(|pool| pool.take_panic(thread));
                match payload {
                    Some(payload) => std::panic::resume_unwind(payload),
                    None => panic!(
                        "a `blocking` job panicked on its thread; its message is printed above"
                    ),
                }
            }
            Outcome::Running => {
                s.outcome = Outcome::Running;
                s.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            Outcome::Taken => panic!("a `blocking` future polled after it completed"),
        }
    }
}

impl<R> Drop for BlockingFuture<R> {
    fn drop(&mut self) {
        self.cancel.0.store(true, Ordering::SeqCst);
    }
}

/// Cooperative cancellation flag passed to [`blocking`]
/// closures that take it: it turns true when the awaiting future is dropped.
#[derive(Clone, Debug, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// Returns whether the awaiting future has requested cancellation.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

fn start<R: Send + 'static>(
    f: impl FnOnce(CancelToken) -> R + Send + 'static,
) -> BlockingFuture<R> {
    let cancel = CancelToken::default();
    let token = cancel.clone();
    let slot = Arc::new(Mutex::new(Slot {
        outcome: Outcome::Running,
        waker: None,
    }));
    let s2 = slot.clone();
    let runtime = crate::Runtime::current();
    let state = runtime.as_deref().map(super::TaskState::of);
    let pool = state.map(super::TaskState::blocking_pool);
    let outstanding = pool.as_ref().map(|p| p.outstanding.clone());
    if let Some(o) = &outstanding {
        o.fetch_add(1, Ordering::SeqCst);
    }
    let job: Job = Box::new(move || {
        let completion = Completion {
            slot: s2,
            outstanding,
        };
        let r = f(token);
        lock(&completion.slot).outcome = Outcome::Done(r);
    });
    let awaiting_pool = pool.clone();
    match pool {
        Some(p) => p.submit(job),
        None => {
            let handle = std::thread::Builder::new()
                .name("kivact-blocking".into())
                .spawn(job);
            if let Ok(handle) = handle {
                if let Some(state) = state {
                    state.retain_fallback(handle);
                } else {
                    kivact_types::process::DetachedThreads::note();
                    drop(handle);
                }
            }
        }
    }
    BlockingFuture {
        slot,
        cancel,
        pool: awaiting_pool,
    }
}

/// Run `f` on the App's blocking pool. Only this closure needs `Send`: read
/// UI handles outside and pass plain values in. A panic in `f` is resumed
/// with its original payload on the UI thread by the task awaiting it, which
/// ends the App (a test fails with its message). If the job's future was
/// already dropped, the panic resumes when the App closes.
///
/// `f` is `|| ..`, or `|cancel| ..` when the work wants the [`CancelToken`]
/// that turns true when the returned future is dropped (a newer request
/// replaced it, its owner was disposed): long work checks it to stop early.
/// A closure that calls a method on `cancel` needs its type
/// (`|cancel: CancelToken| ..`), as `action` needs it for its input.
///
/// App shutdown waits up to two seconds per worker, then warns and detaches
/// jobs that have not stopped. Outside an App runtime the thread has no
/// owner and is detached immediately. Both paths record the detach; dropping
/// an embedded App then keeps its code module mapped until process exit.
/// Work outside any App must still have an owner that keeps its code loaded
/// until it finishes, or closes an embedded App after the work was started.
///
/// ```ignore
/// let n = blocking(move || expensive(input)).await;
/// let m = blocking(move |cancel: CancelToken| long(input, &cancel)).await;
/// ```
pub fn blocking<R, M, F>(f: F) -> impl Future<Output = R>
where
    R: Send + 'static,
    F: BlockingWork<R, M>,
{
    start(move |cancel| f.run(cancel))
}

/// The closure of [`blocking`]: `FnOnce() -> R` or `FnOnce(CancelToken) -> R`,
/// `Send + 'static`. `M` only tells the two forms apart and is inferred; the
/// trait is sealed. It runs on another thread, so it cannot capture a
/// `Signal` or any other UI handle
/// ([`UiThreadOnly`](crate::UiThreadOnly)); the trait exists to say
/// so when it does.
#[diagnostic::on_unimplemented(
    message = "this closure runs on another thread, so it cannot use what it captures here",
    label = "runs on a blocking thread: every capture must be `Send`",
    note = "`Signal`, `Value` and the other handles stay on the UI thread: read the value outside `blocking` and move the plain value in"
)]
pub trait BlockingWork<R, M>: crate::task::sealed::BlockingWorkSeal<M> + Send + 'static {
    #[doc(hidden)]
    fn run(self, cancel: CancelToken) -> R;
}

impl<F: FnOnce() -> R, R> crate::task::sealed::BlockingWorkSeal<crate::task::sealed::Nullary>
    for F
{
}

#[diagnostic::do_not_recommend]
impl<F: FnOnce() -> R + Send + 'static, R> BlockingWork<R, crate::task::sealed::Nullary> for F {
    fn run(self, _: CancelToken) -> R {
        self()
    }
}

impl<F: FnOnce(CancelToken) -> R, R>
    crate::task::sealed::BlockingWorkSeal<crate::task::sealed::Unary<CancelToken>> for F
{
}

#[diagnostic::do_not_recommend]
impl<F: FnOnce(CancelToken) -> R + Send + 'static, R>
    BlockingWork<R, crate::task::sealed::Unary<CancelToken>> for F
{
    fn run(self, cancel: CancelToken) -> R {
        self(cancel)
    }
}
