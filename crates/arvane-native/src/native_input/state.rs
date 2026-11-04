//! Role: Stores native input and IME state.

use std::cell::{Cell, RefCell};

use arvane_shell::{ShellImeText, ShellWaker, WakeReasons};
use kivact_types::cell::LocalMailbox;

use kivact_types::{Environment, PointerSettings};

/// What the window's subclass shares with the window and the runner.
pub(crate) struct NativeInput {
    /// The focused text input's state, as the application last set it.
    pub(crate) text: RefCell<Option<ShellImeText>>,
    /// Physical pixels per logical pixel of the window.
    pub(crate) scale: Cell<f64>,
    /// A range of `text` the IME took for reconversion (snapshot-relative UTF-8 bytes), not
    /// delivered yet.
    pub(crate) reconvert: LocalMailbox<(usize, usize)>,
    /// The client position (physical pixels) of the last button or wheel
    /// message, not taken yet.
    pub(crate) pointer: LocalMailbox<(f64, f64)>,
    pub(crate) file_pointer: LocalMailbox<kivact_types::Point>,
    /// The subclass has not been removed yet (by `WM_NCDESTROY` or the guard's drop); only the
    /// subclass reads it.
    #[cfg(windows)]
    pub(crate) subclass_installed: Cell<Subclass>,
    /// The system's reduce-motion preference as last read.
    pub(crate) reduced_motion: Cell<bool>,
    /// The system's mouse settings as last read.
    pub(crate) mouse: Cell<PointerSettings>,
    /// The system's locale as last read (`system_locale`).
    pub(crate) locale: RefCell<Option<String>>,
    /// The reduce-motion preference, the mouse settings or the locale
    /// changed; the change is not announced yet.
    pub(crate) env_changed: LocalMailbox<()>,
    /// Wakes the event loop when the subclass noted something to deliver
    /// outside a window event (a settings change).
    pub(crate) wake: InputWake,
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
    pub(crate) tilt: Option<(f64, f64)>,
}

/// Delivery route fixed when the input state is constructed.
pub(crate) enum InputWake {
    Direct,
    EventLoop(ShellWaker),
}
impl InputWake {
    pub(crate) fn wake(&self, reasons: WakeReasons) {
        if let Self::EventLoop(waker) = self {
            waker.wake(reasons);
        }
    }
}
#[cfg(windows)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subclass {
    NotInstalled,
    Installed,
}
impl NativeInput {
    pub(crate) fn new(environment: Environment, wake: InputWake) -> Self {
        Self {
            text: RefCell::new(None),
            scale: Cell::new(environment.scale_factor),
            reconvert: LocalMailbox::new(),
            pointer: LocalMailbox::new(),
            file_pointer: LocalMailbox::new(),
            #[cfg(windows)]
            subclass_installed: Cell::new(Subclass::NotInstalled),
            reduced_motion: Cell::new(environment.reduced_motion),
            mouse: Cell::new(environment.pointer),
            locale: RefCell::new(environment.locale),
            env_changed: LocalMailbox::new(),
            wake,
            contacts: RefCell::new(Vec::new()),
        }
    }
}
