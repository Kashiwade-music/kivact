//! Role: The errors of the running App that reached its default sink.

use kivact_reactive::{Runtime, UiError};

/// The errors of the running App that reached its default sink (no
/// `error_boundary` handled them), oldest first.
///
/// The App keeps the most recent 256 in a ring; each one also went to
/// `tracing` when it was reported. Use it to show or attach recent errors
/// in a running app, for example in a "report a problem" dialog; tests read
/// the same ring with `Screen::errors`. The result is a snapshot that
/// subscribes to nothing: read it when it is needed (in a handler, or in
/// the factory of a dialog that opens). Outside an App it is empty.
pub fn errors() -> Vec<UiError> {
    Runtime::current().map(|rt| rt.errors()).unwrap_or_default()
}
