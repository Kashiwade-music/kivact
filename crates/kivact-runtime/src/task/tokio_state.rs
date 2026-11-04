//! Role: Own the lifecycle of the App's single Tokio executor.
use std::future::Future;

/// Lifecycle of the App's single Tokio executor.
pub(super) enum TokioState {
    NotStarted,
    Running(tokio::runtime::Runtime),
    Closed,
}
impl TokioState {
    pub(super) fn spawn<T: Send + 'static>(
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
