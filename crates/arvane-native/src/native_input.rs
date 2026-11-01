//! Role: Native input handling and translation.
//!
//! What the shell reads from the native window beyond winit: the IME's
//! queries about the focused text (reconversion, the document around the
//! composition, character positions) and the position a button or wheel
//! message carries. winit 0.30 answers neither: it lets `WM_IME_REQUEST`
//! fall to the default procedure (no reconversion) and reports buttons and
//! wheels without a position.
//!
//! On Windows a subclass of the window (`windows.rs`, using Win32 FFI)
//! answers the IME from the [`arvane_shell::ShellImeText`] the
//! application last gave the window and notes what the window learns
//! ([`state::NativeInput`]); the runner turns the notes into events. Elsewhere
//! nothing reads them.
//!
//! The IME: winit 0.30 drives Windows' IME through IMM32
//! (`ImmSetCompositionWindow`, `WM_IME_COMPOSITION`) and keeps no TSF text
//! store. A TSF IME (the Microsoft IME of Windows 10 and later) reaches the
//! window through IMM32's compatibility layer, and asks for the text
//! around the composition and for reconversion with `WM_IME_REQUEST`
//! (`RECONVERTSTRING`: a string and the offsets of the composition and the
//! target in it). So an excerpt of the text answers it, not the whole
//! document: offsets are UTF-16 code units of the excerpt and
//! [`arvane_shell::ShellImeText::text_start`] is where the excerpt starts (the
//! IME's selection, [`arvane_shell::ShellEvent::ImeSelection`], is in the
//! excerpt too). winit offers no way to hand other platforms the
//! surrounding text (macOS's `NSTextInputClient` range queries, Wayland's
//! text-input v3 `set_surrounding_text` are not used), so the excerpt is
//! read on Windows only. text-input v3 takes up to 4000 bytes of
//! surrounding text, and the host's excerpt holds up to 2048 bytes on each
//! side of the caret or selection (`kivact-engine`'s `IME_CONTEXT`), so
//! up to 4096 bytes besides the selection: handing it there once winit
//! offers it would need it cut to the limit.
//!
//! Invariants:
//! - Everything here lives on the window's UI thread; the subclass runs on
//!   it, inside winit's message dispatch, and only reads or replaces
//!   values (no borrow is held across a call out).
//! - Offsets into [`arvane_shell::ShellImeText::text`] are UTF-16 code units (what
//!   platform IMEs count); rectangles are logical window pixels.
//! - A change of the system's reduce-motion preference, mouse settings or
//!   locale (`WM_SETTINGCHANGE`) is noted and the loop woken; the runner
//!   delivers [`arvane_shell::ShellEvent::EnvironmentChanged`] before the wake.
//!   Where no message tells of a locale change, the runner reads it again
//!   when the window regains the focus (`runtime.rs`, at `Focused(true)`;
//!   on every platform, for a change made elsewhere meanwhile).
//! - Touch and pen arrive from winit as touches (`WM_POINTER*`, or
//!   `WM_TOUCH` on systems without pointer messages); the subclass notes
//!   the kind, whether the contact was canceled and a pen's tilt of every
//!   pointer of the message's frame before winit reports them
//!   ([`state::ContactNote`]). Mouse messages Windows
//!   synthesizes from touch or pen (their extra info carries the
//!   `MI_WP_SIGNATURE`) are dropped before winit sees them, so a touch is
//!   never reported as the mouse as well.
//! - A range the IME takes for reconversion is delivered as
//!   [`arvane_shell::ShellEvent::ImeSelection`] before the IME event that follows
//!   it (the reconversion's composition replaces that range).

pub(crate) mod locale;
pub(crate) mod state;
use state::MouseSettings;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::Subclass;
#[cfg(all(windows, feature = "embed"))]
pub(crate) use windows::ime_request;

impl MouseSettings {
    /// The system's settings (the defaults where the shell cannot read
    /// them).
    pub(crate) fn read() -> Self {
        #[cfg(windows)]
        {
            windows::mouse_settings()
        }
        #[cfg(not(windows))]
        {
            MouseSettings::default()
        }
    }
}

/// Whether the system asks to reduce motion (Windows: the "animation
/// effects" setting is off); `false` where the shell cannot tell.
pub(crate) fn reduced_motion() -> bool {
    #[cfg(windows)]
    {
        windows::reduced_motion().unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Current file drag position; Windows queries it even during OLE capture.
pub(crate) fn file_position(window: &winit::window::Window) -> Option<kivact_types::Point> {
    #[cfg(windows)]
    {
        windows::file_position(window)
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        None
    }
}
pub(crate) fn track_files(window: &winit::window::Window, tracking: bool) {
    #[cfg(windows)]
    {
        windows::track_files(window, tracking);
    }
    #[cfg(not(windows))]
    {
        let _ = (window, tracking);
    }
}
