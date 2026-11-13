//! Role: Host window, scale and host callbacks of an embedded App.

use super::EmbedNotice;
use kivact_types::NativeWindow;
use kivact_types::embed::Visibility;
use kivact_types::{ResizeAnswer, UnhandledKeys};
use std::ffi::c_void;
use std::num::{NonZeroIsize, NonZeroU32};
use std::sync::Arc;

/// A host window and optional host-decided scale. Constructors only store raw
/// values and never dereference them. `App::embed` validates the parent.
/// If the parent dies first, the child stops safely without undefined behavior.
#[derive(Clone)]
pub struct Parent {
    // xtask-allow(state-fields): option: an optional constructor input; the default applies when absent
    pub(super) window: Option<NativeWindow>,
    // xtask-allow(state-fields): option: an optional host scale; the default applies when absent
    pub(super) scale: Option<f64>,
    pub(super) visibility: Visibility,
    pub(super) unhandled_keys: UnhandledKeys,
    pub(super) on_resize_request: Option<Arc<dyn Fn(u32, u32) -> ResizeAnswer + Send + Sync>>,
    pub(super) on_notice: Option<Arc<dyn Fn(EmbedNotice) + Send + Sync>>,
}
impl std::fmt::Debug for Parent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parent")
            .field("window", &self.window)
            .field("scale", &self.scale)
            .field("visibility", &self.visibility)
            .field("unhandled_keys", &self.unhandled_keys)
            .field("on_resize_request", &self.on_resize_request.is_some())
            .field("on_notice", &self.on_notice.is_some())
            .finish()
    }
}
impl Parent {
    /// A Win32 `HWND`: CLAP `clap_window.win32`, VST3 `"HWND"`,
    /// or nih-plug `ParentWindowHandle::Win32Hwnd`. Null is refused by `embed`.
    pub fn win32(hwnd: *mut c_void) -> Parent {
        Self {
            window: NonZeroIsize::new(hwnd as isize).map(NativeWindow::Win32),
            scale: None,
            visibility: Visibility::default(),
            unhandled_keys: UnhandledKeys::default(),
            on_resize_request: None,
            on_notice: None,
        }
    }
    /// An X11 id: CLAP `clap_window.x11`, VST3 `"X11EmbedWindowID"`,
    /// or nih-plug `ParentWindowHandle::X11Window`.
    pub fn x11(window: u32) -> Parent {
        Self {
            window: NonZeroU32::new(window).map(NativeWindow::X11),
            scale: None,
            visibility: Visibility::default(),
            unhandled_keys: UnhandledKeys::default(),
            on_resize_request: None,
            on_notice: None,
        }
    }
    /// Create the editor's window hidden (CLAP: the host calls `show` after
    /// `set_parent`). A hidden editor runs its App (tasks, timers, signals) but
    /// draws nothing, its frame loops do not tick and
    /// `platform::is_surface_active()` is false, until
    /// [`Embedded::show`](super::Embedded::show). Without it the window is
    /// shown when `App::embed` returns.
    pub fn hidden(mut self) -> Parent {
        self.visibility = Visibility::Hidden;
        self
    }
    /// Give key presses the editor does not handle, and their releases, back
    /// to the parent window: Windows posts the original `WM_KEYDOWN`/`WM_KEYUP`
    /// to the parent (as JUCE editors do); X11 sends a synthetic
    /// `KeyPress`/`KeyRelease` to the parent (`XSendEvent`; some toolkits ignore
    /// synthetic events). A key is unhandled when no focused text input, focus
    /// movement, `on_key` handler or default action (Enter/Space activation,
    /// Escape dismissal) used it. Off by default.
    pub fn forward_unhandled_keys(mut self) -> Parent {
        self.unhandled_keys = UnhandledKeys::ToParent;
        self
    }
    /// Forward `platform::request_window_size` to the host in physical pixels
    /// and say what the host did. Runs on the App UI thread (the host GUI
    /// thread on Windows, the editor's own thread on X11) and must not wait
    /// for the host GUI thread; `Embedded::set_size` may be called from inside
    /// it. Without this callback, requests fail as `Unsupported`.
    pub fn on_resize_request(
        mut self,
        f: impl Fn(u32, u32) -> ResizeAnswer + Send + Sync + 'static,
    ) -> Parent {
        self.on_resize_request = Some(Arc::new(f));
        self
    }
    /// Receive an [`EmbedNotice`] when the operating system changes the
    /// applied scale or when the App stops.
    ///
    /// The callback runs on the thread that observes the change: normally the
    /// App's UI thread; on X11 the editor thread while it exits; a close that
    /// starts by dropping the handle can report `Stopped` on the dropping
    /// thread. It must not wait for the host's GUI thread and must not panic:
    /// post the work to the host's main thread, then read
    /// [`Embedded::status`](super::Embedded::status) or
    /// [`Embedded::scale`](super::Embedded::scale) there. A notice is a
    /// wake-up, not a state; it arrives once per change. A failure or panic
    /// while `App::embed` opens the editor can send `Stopped` before `embed`
    /// returns.
    pub fn on_notice(mut self, f: impl Fn(EmbedNotice) + Send + Sync + 'static) -> Parent {
        self.on_notice = Some(Arc::new(f));
        self
    }
    /// Host-decided scale (CLAP `set_scale`, VST3 `setContentScaleFactor`).
    /// Without it the OS scale is used. Nonfinite or nonpositive values,
    /// including values outside positive finite f64 range, are refused by `embed`.
    pub fn scale(mut self, scale: f64) -> Parent {
        self.scale = Some(scale);
        self
    }
}
