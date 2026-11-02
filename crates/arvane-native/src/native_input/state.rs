//! Role: Stores native input and IME state.

use std::cell::{Cell, RefCell};

use arvane_shell::ShellImeText;

/// The system's mouse settings: how close in time (ms) and space
/// (physical pixels per axis) a second press counts as a double click, and
/// how many lines (characters with Shift) one notch of the wheel scrolls.
/// Windows reads them from the system and again when they change; the
/// defaults are Windows' own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MouseSettings {
    pub(crate) double_click_ms: u32,
    pub(crate) double_click_px: (u32, u32),
    pub(crate) wheel_lines: u32,
    pub(crate) wheel_chars: u32,
}

impl Default for MouseSettings {
    fn default() -> Self {
        MouseSettings {
            double_click_ms: 500,
            double_click_px: (4, 4),
            wheel_lines: 3,
            wheel_chars: 3,
        }
    }
}

/// What the window's subclass shares with the window and the runner.
#[derive(Debug, Default)]
pub(crate) struct NativeInput {
    /// The focused text input's state, as the application last set it.
    pub(crate) text: RefCell<Option<ShellImeText>>,
    /// Physical pixels per logical pixel of the window.
    pub(crate) scale: Cell<f64>,
    /// A range of `text` the IME took for reconversion (UTF-16), not
    /// delivered yet.
    pub(crate) reconvert: Cell<Option<(usize, usize)>>,
    /// The client position (physical pixels) of the last button or wheel
    /// message, not taken yet.
    pub(crate) pointer: Cell<Option<(f64, f64)>>,
    pub(crate) file_pointer: Cell<Option<kivact_types::Point>>,
    /// The subclass has not been removed yet (by `WM_NCDESTROY` or the guard's drop); only the
    /// subclass reads it.
    #[cfg(windows)]
    pub(crate) subclass_installed: Cell<bool>,
    /// The system's reduce-motion preference as last read.
    pub(crate) reduced_motion: Cell<bool>,
    /// The system's mouse settings as last read.
    pub(crate) mouse: Cell<MouseSettings>,
    /// The system's locale as last read (`system_locale`).
    pub(crate) locale: RefCell<Option<String>>,
    /// The reduce-motion preference, the mouse settings or the locale
    /// changed; the change is not announced yet.
    pub(crate) env_changed: Cell<bool>,
    /// Wakes the event loop when the subclass noted something to deliver
    /// outside a window event (a settings change).
    pub(crate) waker: RefCell<Option<arvane_shell::ShellWaker>>,
    /// The touch and pen pointers of the frame of the last pointer message
    /// (Windows' `WM_POINTER*`: winit reports every pointer of the frame
    /// from it), read while winit reports them: winit reports a pen as a
    /// touch and a canceled contact as lifted.
    pub(crate) contacts: RefCell<Vec<ContactNote>>,
}

/// What the platform says about a touch or pen pointer that winit does
/// not report.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ContactNote {
    /// The platform's pointer id (winit's touch id).
    pub(crate) id: u64,
    /// Touch or pen.
    pub(crate) kind: kivact_types::PointerKind,
    /// The contact ended without a release (the system took it over).
    pub(crate) canceled: bool,
    /// The pen's tilt (degrees along x and y), when reported.
    pub(crate) tilt: Option<(f32, f32)>,
}
