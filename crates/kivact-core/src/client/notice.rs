//! Role: The core's own diagnostics of the client: `tracing` messages that
//! are not errors of the App.

use kivact_runtime::Runtime;

/// A `tracing` message (target `kivact`, info level) that is not part of
/// the turn trace.
pub(crate) fn inform(msg: &str) {
    tracing::info!(target: "kivact", "{}", msg);
}

/// A warning of the layout lint (`tracing` target `kivact::layout`), also
/// recorded as a note of the turn. Not an error of the App.
pub(super) fn warn_layout(rt: &Runtime, msg: String) {
    tracing::warn!(target: "kivact::layout", "{}", msg);
    rt.note(msg);
}
