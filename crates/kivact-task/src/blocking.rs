//! Role: Blocking operations with cancellation support.
//!
//! `blocking`: run a `Send` closure on the App-owned fixed thread pool and
//! await its result on the UI thread. Dropping the future discards the
//! result and signals the closure's `CancelToken`.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use super::state::pool::{Job, lock};

struct Slot<R> {
    result: Option<std::thread::Result<R>>,
    waker: Option<Waker>,
}

/// Future of a blocking job.
pub(crate) struct BlockingFuture<R> {
    slot: Arc<Mutex<Slot<R>>>,
    cancel: CancelToken,
}

impl<R> Future for BlockingFuture<R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<R> {
        let mut s = lock(&self.slot);
        match s.result.take() {
            Some(Ok(v)) => Poll::Ready(v),
            Some(Err(p)) => {
                drop(s);
                std::panic::resume_unwind(p)
            }
            None => {
                s.waker = Some(cx.waker().clone());
                Poll::Pending
            }
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
        result: None,
        waker: None,
    }));
    let s2 = slot.clone();
    let runtime = kivact_reactive::Runtime::current();
    let state = runtime.as_deref().map(super::TaskState::of);
    let pool = state.map(super::TaskState::blocking_pool);
    let outstanding = pool.as_ref().map(|p| p.outstanding.clone());
    if let Some(o) = &outstanding {
        o.fetch_add(1, Ordering::SeqCst);
    }
    let job: Job = Box::new(move || {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || f(token)));
        let waker = {
            let mut s = lock(&s2);
            s.result = Some(r);
            s.waker.take()
        };
        // Wake (schedule the awaiting task) before the job stops counting as
        // outstanding: a test harness that sees `outstanding == 0` then also
        // sees the ready task, so it cannot stop before the result is
        // consumed (this once made a history test flaky).
        if let Some(w) = waker {
            w.wake();
        }
        if let Some(o) = outstanding {
            o.fetch_sub(1, Ordering::SeqCst);
        }
    });
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
                    crate::detached::DETACHED.store(true, Ordering::Release);
                    drop(handle);
                }
            }
        }
    }
    BlockingFuture { slot, cancel }
}

/// Run `f` on the App's blocking pool. Only this closure needs `Send`: read
/// UI handles outside and pass plain values in. A panic in `f` resurfaces as
/// a `UiError` in the resource or action that awaited it.
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

mod sealed {
    //! Role: Seal of `BlockingWork` and its two argument forms.
    pub trait Sealed<M> {}
    /// The form of a closure without an argument.
    pub struct Nullary;
    /// The form of a closure that takes the `CancelToken`.
    pub struct Unary;
}

/// The closure of [`blocking`]: `FnOnce() -> R` or `FnOnce(CancelToken) -> R`,
/// `Send + 'static`. `M` only tells the two forms apart and is inferred; the
/// trait is sealed. It runs on another thread, so it cannot capture a
/// `Signal` or any other UI handle
/// ([`UiThreadOnly`](kivact_reactive::UiThreadOnly)); the trait exists to say
/// so when it does.
#[diagnostic::on_unimplemented(
    message = "this closure runs on another thread, so it cannot use what it captures here",
    label = "runs on a blocking thread: every capture must be `Send`",
    note = "`Signal`, `Value` and the other handles stay on the UI thread: read the value outside `blocking` and move the plain value in"
)]
pub trait BlockingWork<R, M>: sealed::Sealed<M> + Send + 'static {
    #[doc(hidden)]
    fn run(self, cancel: CancelToken) -> R;
}

impl<F: FnOnce() -> R, R> sealed::Sealed<sealed::Nullary> for F {}

#[diagnostic::do_not_recommend]
impl<F: FnOnce() -> R + Send + 'static, R> BlockingWork<R, sealed::Nullary> for F {
    fn run(self, _: CancelToken) -> R {
        self()
    }
}

impl<F: FnOnce(CancelToken) -> R, R> sealed::Sealed<sealed::Unary> for F {}

#[diagnostic::do_not_recommend]
impl<F: FnOnce(CancelToken) -> R + Send + 'static, R> BlockingWork<R, sealed::Unary> for F {
    fn run(self, cancel: CancelToken) -> R {
        self(cancel)
    }
}
