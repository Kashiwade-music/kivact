//! Role: Pointer identity and phases, safe-area insets, capabilities, lifecycle.
//!
//! The platform vocabulary of a window: its pointers (mouse, touch, pen),
//! the safe-area insets and what the backend it runs on can do
//! (capabilities). The window's environment, mouse settings and parent
//! handle are the host's protocol (`protocol::window`).
//!
//! Invariants:
//! - Positions and insets are logical window pixels, times nanoseconds.
//! - A capability that is `false` means the backend never delivers that
//!   kind of event or never acts on that kind of request; the values that
//!   depend on it keep their neutral value (zero insets, no soft keyboard).

use crate::input::MouseButton;

/// Identifies one pointer: the mouse ([`PointerId::MOUSE`]), or one touch
/// or pen contact for as long as it lasts (from its `Down` to its `Up` or
/// `Cancel`). A new contact gets a new id; a window never reuses one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerId(pub u64);

impl PointerId {
    /// The mouse (and anything the platform reports as one, such as a
    /// touchpad).
    pub const MOUSE: PointerId = PointerId(1);
}

/// What kind of device a pointer is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PointerKind {
    /// A mouse or touchpad: it hovers, has buttons and a wheel.
    #[default]
    Mouse,
    /// A finger on a touch screen: it exists only while in contact.
    Touch,
    /// A pen (stylus) on a digitizer, while in contact.
    Pen,
}

/// The button a `Down` pressed or an `Up` released, and which click of a
/// multi-click the press is. A touch or pen contact presses
/// [`MouseButton::Primary`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ButtonPress {
    /// The button.
    pub button: MouseButton,
    /// Which click of a multi-click the press is: 1 a single click, 2 a
    /// double click, ..; never 0. An `Up` has the count of the press it
    /// ends. The window counts it from the system's double-click time and
    /// distance (per axis) from the last press of the same button; a
    /// contact's taps count the same way from the last contact's `Down`.
    pub clicks: u8,
}

/// The phase of a pointer event (Pointer Events' `pointerdown` ..
/// `pointerleave`), shared by the window's input and the events a node's
/// listener receives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerPhase {
    /// A button was pressed, or a touch or pen contact began.
    Down(ButtonPress),
    /// The pointer moved, with or without buttons held.
    Move,
    /// A button was released, or a contact ended normally (lifted).
    Up(ButtonPress),
    /// The gesture ended without a release the application should act on
    /// (the window lost focus with buttons held, the platform took a
    /// contact over for a gesture or rejected a palm).
    Cancel,
    /// The pointer entered a listener. A window does not report it; the
    /// host synthesizes it when the pointer moves onto a listener.
    Enter,
    /// The pointer left a listener, or the mouse left the window. Held
    /// buttons keep their capture.
    Leave,
}

impl PointerPhase {
    /// The press of a `Down` or the release of an `Up`; `None` for the other
    /// phases.
    pub fn press(self) -> Option<ButtonPress> {
        match self {
            Self::Down(press) | Self::Up(press) => Some(press),
            Self::Move | Self::Cancel | Self::Enter | Self::Leave => None,
        }
    }
}

/// The set of mouse buttons held (Pointer Events' `buttons`). A touch or
/// pen contact holds [`MouseButton::Primary`]
/// while in contact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PointerButtons(u8);

impl PointerButtons {
    /// No button is held.
    pub const NONE: Self = Self(0);

    /// Whether `button` is held.
    pub const fn contains(self, button: MouseButton) -> bool {
        self.0 & Self::bit(button) != 0
    }

    /// This set with `button` held (`held`) or released.
    #[must_use]
    pub const fn with(self, button: MouseButton, held: bool) -> Self {
        if held {
            Self(self.0 | Self::bit(button))
        } else {
            Self(self.0 & !Self::bit(button))
        }
    }

    /// Whether no button is held.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    const fn bit(button: MouseButton) -> u8 {
        match button {
            MouseButton::Primary => 1,
            MouseButton::Secondary => 2,
            MouseButton::Middle => 4,
            MouseButton::Back => 8,
            MouseButton::Forward => 16,
        }
    }
}

/// Distances from a window's edges, in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    /// From the top edge.
    pub top: f32,
    /// From the right edge.
    pub right: f32,
    /// From the bottom edge.
    pub bottom: f32,
    /// From the left edge.
    pub left: f32,
}

/// What the backend a window runs on can do, reported once, when the
/// window exists: requests it cannot act on and events it never delivers
/// can be diagnosed instead of being ignored.
///
/// Each platform's values are decided by the window that reports them:
/// `arvane-native`'s native window by what winit 0.30 and its Windows
/// subclass deliver on that platform, the `arvane` facade adding the GPU;
/// a test window stands for a platform that can do everything, but the GPU
/// when it is not asked for and native accessibility without a real
/// window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// Touch contacts arrive as touch pointers.
    pub touch: bool,
    /// Pen contacts arrive as pen pointers (not as touch or mouse).
    pub pen: bool,
    /// A request to show or hide the software keyboard is acted on.
    pub soft_keyboard: bool,
    /// The environment's safe area is the platform's (else always zero).
    pub safe_area: bool,
    /// The platform's memory warnings are delivered.
    pub memory_warnings: bool,
    /// The platform IME's reconversion and text queries are answered
    /// from the focused input's text.
    pub ime_reconversion: bool,
    /// The accessibility tree reaches the platform's assistive
    /// technology.
    pub native_accessibility: bool,
    /// Frames can be drawn on the GPU (GPU support is built in; which
    /// renderer draws them is reported as it changes); else on the CPU
    /// only.
    pub gpu: bool,
}

/// An actual application lifecycle transition or platform memory warning.
/// Surface availability and visibility remain current state in the host's environment (`HostEnv`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// Application execution has been suspended by the shell.
    Suspended,
    /// The shell resumed application execution, possibly without a surface yet.
    Resumed,
    /// The platform asks the application to release rebuildable resources.
    MemoryWarning,
}
