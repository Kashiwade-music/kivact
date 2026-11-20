//! Role: Read-only stop status of an embedded App.

use crate::UiError;
use kivact_runtime::ErrorKind;

/// What the embedded App is doing. A stop is final: the App never runs
/// again, and a stopped App keeps its last status until the handle is
/// dropped.
#[derive(Clone, Debug, PartialEq)]
pub enum EmbedStatus {
    /// The App runs.
    Running,
    /// The host destroyed the parent (or the child window) first. Not an error.
    Closed,
    /// A platform failure stopped the App and destroyed its window
    /// (`ErrorKind::Platform`).
    Failed(UiError),
    /// The App panicked (a bug). Its window is gone and no more App code runs.
    /// The error has `ErrorKind::Panicked` and the panic's message; the panic
    /// hook already printed it with its location. Kivact never resumes the
    /// panic on the embedder's thread.
    Panicked(UiError),
}

/// The status of a child whose engine stop is `stop`.
pub(crate) fn status_of(
    stop: Option<kivact_engine::embed::ChildStop>,
    panic: &kivact_engine::embed::PanicSlot,
) -> EmbedStatus {
    match stop {
        None => EmbedStatus::Running,
        Some(kivact_engine::embed::ChildStop::Closed) => EmbedStatus::Closed,
        Some(kivact_engine::embed::ChildStop::Failed(error)) => {
            EmbedStatus::Failed(UiError::new(ErrorKind::Platform, error.to_string(), None))
        }
        Some(kivact_engine::embed::ChildStop::Panicked) => EmbedStatus::Panicked(UiError::new(
            ErrorKind::Panicked,
            panic
                .message()
                .unwrap_or_else(|| "the embedded App panicked".to_owned()),
            None,
        )),
    }
}
