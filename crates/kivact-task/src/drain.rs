//! Role: Drain of queued actions on owner disposal, `close` and App exit.
//!
//! An action with a queue policy keeps running after its owner is disposed
//! until it is drained or its deadline (default 3 s) passes; App exit waits
//! for every queued action with the same deadline. Leftovers are reported.

use std::future::Future;
use std::pin::Pin;
use std::rc::Weak;
use std::task::{Context, Poll, Waker};

use super::state::Drain;
use kivact_reactive::{ErrorKind, UiError};

/// How long a queued action may keep draining after its owner is disposed,
/// and how long App exit waits for queued actions (ns).
pub const DEFAULT_DRAIN_NS: u64 = 3_000_000_000;

/// The future of `Action::close`.
pub(crate) struct CloseFuture {
    pub(crate) action: Weak<dyn Drain>,
    pub(crate) register: Box<dyn Fn(&Waker)>,
    pub(crate) sleep: Pin<Box<super::time::Sleep>>,
}

impl Future for CloseFuture {
    type Output = Result<(), UiError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Some(a) = self.action.upgrade() else {
            return Poll::Ready(Ok(()));
        };
        if !a.busy() {
            return Poll::Ready(Ok(()));
        }
        (self.register)(cx.waker());
        match self.sleep.as_mut().poll(cx) {
            Poll::Ready(()) => {
                let n = a.abandon();
                Poll::Ready(Err(UiError::new(
                    ErrorKind::Closed,
                    format!("action closed with {n} run(s) abandoned"),
                    None,
                )))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
