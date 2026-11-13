//! Role: Take an AppHandle's close request at the start of a core turn.
//!
//! The handle records the request from any thread; this module is the only
//! place that turns it into the runtime's exit drain (or, for an embedded
//! App, an error report to the root owner).

use kivact_runtime::{ErrorKind, Runtime, UiError};

use crate::platform::AppHandleExt;

/// What the root owner is told when code closes an embedded App.
const EMBEDDED_CLOSE: &str =
    "an embedded App is closed by its host: drop the Embedded handle instead of AppHandle::close";

/// Take a pending `AppHandle::close` request, if any.
///
/// An embedded App refuses the request: its host owns the embedded editor
/// (`Embedded`), so the close is reported instead of ending the run. The
/// runtime is the single owner of the drain that follows.
pub(super) fn take_close_request(rt: &Runtime, embedded: bool) {
    let accept = !embedded;
    if !AppHandleExt::of(rt).0.take_request(accept) {
        return;
    }
    if accept {
        rt.request_exit();
    } else {
        rt.report(
            Some(rt.root()),
            UiError::new(ErrorKind::Unsupported, EMBEDDED_CLOSE, None),
        );
    }
}
