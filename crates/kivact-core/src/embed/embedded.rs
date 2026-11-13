//! Role: The running embedded App handle.

use super::EmbedStatus;
use crate::UiError;
use kivact_engine::embed::{ChildStop, EmbeddedHost, PanicSlot};
use kivact_runtime::ErrorKind;
use kivact_types::embed::Visibility;

/// An App running inside a host window. `Send` and `Sync`.
/// Setters queue the change for the App UI thread and return at once; getters
/// return the last applied value. Sizes are physical pixels; scale is physical
/// pixels per logical pixel. A zero or oversized size, or a nonfinite or
/// nonpositive scale, is ignored with a warning. Prefer dropping on the host
/// GUI thread. Outside an App callback, dropping normally waits for cleanup of
/// the App and its native window. Each owned thread, including a thread running
/// UI callbacks, is waited for at most two seconds. After that limit, unfinished
/// cleanup may continue after drop returns; Kivact retains its code module so
/// the host may unload the plugin. Inside an App callback, the turn in progress
/// finishes and cleanup follows it; no later turn or input runs. Queued actions
/// are not drained.
/// After a platform failure or a panic the App stops, its window is destroyed
/// and [`Embedded::status`] says why. No method and no drop of `Embedded`
/// unwinds because of the App: a panic is reported as a value, never resumed
/// on the calling thread, so a C-ABI host does not abort.
/// Keep plugin state outside the App.
/// If a thread exceeds its shutdown wait, or a native library needs its code
/// after closure, Kivact keeps its code module mapped until process exit. The
/// host may unload the plugin after closing the editor.
pub struct Embedded {
    // Fields drop in declaration order: stop the host before retaining code
    // for detached threads.
    host: EmbeddedHost,
    panic: PanicSlot,
    _retention: Retention,
}
impl Embedded {
    /// Own a stopped or running host and the slot that keeps a panic message.
    pub(super) fn new(host: EmbeddedHost, panic: PanicSlot) -> Self {
        Self {
            host,
            panic,
            _retention: Retention,
        }
    }

    /// Set physical size (CLAP `set_size`, VST3 `onSize`). A zero or
    /// platform-oversized dimension is ignored with a warning. It also answers
    /// a [`ResizeAnswer::Pending`](kivact_types::ResizeAnswer::Pending) resize
    /// request: the window resizes when the host confirms with this call.
    pub fn set_size(&self, width: u32, height: u32) {
        self.host.set_size(width, height);
    }
    /// Last applied physical size (CLAP `get_size`, VST3 `getSize`).
    pub fn size(&self) -> (u32, u32) {
        self.host.size()
    }
    /// Show the editor's window (CLAP `gui.show`). Idempotent; queued like the
    /// other setters. The window does not take the keyboard focus. Ignored after
    /// the App stopped.
    pub fn show(&self) {
        self.set_visibility(Visibility::Shown);
    }
    /// Hide the editor's window (CLAP `gui.hide`). Idempotent; queued. If the
    /// editor has the keyboard focus, it goes back to the parent window first;
    /// a pressed pointer is cancelled. Drawing and frame loops stop until
    /// [`Embedded::show`].
    pub fn hide(&self) {
        self.set_visibility(Visibility::Hidden);
    }
    /// Whether the window is shown, as last applied on the UI thread (a queued
    /// `show`/`hide` is not reflected yet).
    pub fn is_visible(&self) -> bool {
        self.host.visibility() == Visibility::Shown
    }
    fn set_visibility(&self, visibility: Visibility) {
        self.host.set_visibility(visibility);
    }
    /// Set physical pixels per logical pixel, keeping physical size until the
    /// host resizes it. To retain logical size, request logical size times scale
    /// from the host (CLAP `request_resize`, VST3 `resizeView`). A nonfinite or
    /// nonpositive scale is ignored with a warning.
    pub fn set_scale(&self, scale: f64) {
        self.host.set_scale(scale);
    }
    /// Last applied scale.
    pub fn scale(&self) -> f64 {
        self.host.scale()
    }
    /// Whether the App runs, and why it stopped (CLAP: call `host->closed`;
    /// VST3: treat the view as dead when this is not `Running`).
    pub fn status(&self) -> EmbedStatus {
        match self.host.stop() {
            None => EmbedStatus::Running,
            Some(ChildStop::Closed) => EmbedStatus::Closed,
            Some(ChildStop::Failed(error)) => {
                EmbedStatus::Failed(UiError::new(ErrorKind::Platform, error.to_string(), None))
            }
            Some(ChildStop::Panicked) => EmbedStatus::Panicked(UiError::new(
                ErrorKind::Panicked,
                self.panic
                    .message()
                    .unwrap_or_else(|| "the embedded App panicked".to_owned()),
                None,
            )),
        }
    }
}

// The handle's last field: it drops after the host, whose cleanup may detach threads.
struct Retention;
impl Drop for Retention {
    fn drop(&mut self) {
        if kivact_types::process::DetachedThreads::any() {
            kivact_engine::embed::retain_module();
        }
    }
}
