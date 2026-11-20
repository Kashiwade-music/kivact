//! Role: Sealed contract for standard async view sources.
use super::AsyncState;

/// A resource or action that can be displayed by an async view.
#[diagnostic::on_unimplemented(
    message = "async_view takes a Resource or Action",
    note = "pass the source handle directly; do not call state()"
)]
pub trait AsyncSource<T>: super::sealed::AsyncSourceSeal + Copy + 'static {
    /// Read the current tracked state.
    fn state(&self) -> AsyncState<T>;
    /// A callback that runs the source again (refetch, or retry the last run). Setup or handler.
    fn retry(&self) -> crate::Callback<()>;
    /// How many times the state was published (tracked).
    #[doc(hidden)]
    fn publications(&self) -> u64;
}
