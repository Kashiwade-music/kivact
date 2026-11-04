//! Role: Run Send futures on one lazily created Tokio runtime per App.
use super::TaskState;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Run a Send future on the App's Tokio runtime and await it on the UI thread.
/// Dropping this future aborts its task. UI handles stay outside `fut`.
/// Requires the opt-in `tokio` feature and an active App.
pub fn on_tokio<T: Send + 'static>(
    fut: impl Future<Output = T> + Send + 'static,
) -> impl Future<Output = T> {
    let runtime = crate::Runtime::current().expect("invariant: on_tokio is called inside an App");
    crate::require_effectful("on_tokio");
    let state = TaskState::of(&runtime);
    TokioFuture(state.tokio.borrow_mut().spawn(fut))
}
struct TokioFuture<T>(tokio::task::JoinHandle<T>);
impl<T> Future for TokioFuture<T> {
    type Output = T;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(value)) => Poll::Ready(value),
            Poll::Ready(Err(error)) if error.is_panic() => {
                std::panic::resume_unwind(error.into_panic())
            }
            Poll::Ready(Err(_)) => {
                panic!("invariant: a live App's awaited Tokio task is not cancelled")
            }
        }
    }
}
impl<T> Drop for TokioFuture<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Lifecycle of the App's single Tokio executor.
pub(super) enum TokioState {
    NotStarted,
    Running(tokio::runtime::Runtime),
    Closed,
}
impl TokioState {
    fn spawn<T: Send + 'static>(
        &mut self,
        future: impl Future<Output = T> + Send + 'static,
    ) -> tokio::task::JoinHandle<T> {
        if matches!(self, Self::NotStarted) {
            *self = Self::Running(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .thread_name("kivact-tokio")
                    .build()
                    .expect("invariant: the App can create its Tokio runtime"),
            );
        }
        match self {
            Self::Running(runtime) => runtime.spawn(future),
            Self::NotStarted | Self::Closed => {
                panic!("invariant: Tokio tasks start before App shutdown")
            }
        }
    }
    pub(super) fn shutdown(&mut self) {
        if let Self::Running(runtime) = std::mem::replace(self, Self::Closed) {
            runtime.shutdown_background();
        }
    }
}
