//! Role: Sealed contract for standard async view sources.
use super::{AsyncSnapshot, AsyncState, Retry};

/// A resource or action that can be displayed by an async view.
#[diagnostic::on_unimplemented(
    message = "async_view takes a Resource or Action",
    note = "pass the source handle directly; do not call state()"
)]
pub trait AsyncSource<T>: super::sealed::AsyncSourceSeal + Copy + 'static {
    /// Read the current tracked state.
    fn state(&self) -> AsyncState<T>;
    #[doc(hidden)]
    fn snapshot(&self) -> AsyncSnapshot<T>;
    /// Obtain a callback that retries this source.
    fn retry(&self) -> Retry;
}
