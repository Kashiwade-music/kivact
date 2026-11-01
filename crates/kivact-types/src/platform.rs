//! Role: Platform abstraction and services.
//!
//! The platform vocabulary of a window: its pointers (mouse, touch, pen),
//! its environment (scale, refresh period, preferences, safe area, mouse
//! settings) and what the backend it runs on can do (capabilities).
//!
//! Invariants:
//! - Positions and insets are logical window pixels, times nanoseconds.
//! - A capability that is `false` means the backend never delivers that
//!   kind of event or never acts on that kind of request; the values that
//!   depend on it keep their neutral value (zero insets, no soft keyboard).

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

/// The phase of a pointer event (Pointer Events' `pointerdown` ..
/// `pointerleave`), shared by the window's input and the events a node's
/// listener receives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerPhase {
    /// A button was pressed, or a touch or pen contact began.
    Down,
    /// The pointer moved, with or without buttons held.
    Move,
    /// A button was released, or a contact ended normally (lifted).
    Up,
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

/// The set of mouse buttons held (Pointer Events' `buttons`). A touch or
/// pen contact holds [`MouseButton::Primary`](crate::MouseButton::Primary)
/// while in contact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PointerButtons(u8);

impl PointerButtons {
    /// No button is held.
    pub const NONE: Self = Self(0);

    /// Whether `button` is held.
    pub const fn contains(self, button: crate::MouseButton) -> bool {
        self.0 & Self::bit(button) != 0
    }

    /// This set with `button` held (`held`) or released.
    #[must_use]
    pub const fn with(self, button: crate::MouseButton, held: bool) -> Self {
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

    const fn bit(button: crate::MouseButton) -> u8 {
        match button {
            crate::MouseButton::Primary => 1,
            crate::MouseButton::Secondary => 2,
            crate::MouseButton::Middle => 4,
            crate::MouseButton::Back => 8,
            crate::MouseButton::Forward => 16,
        }
    }
}

/// One event of the mouse, a touch or a pen, as a window delivers it
/// (Pointer Events' `PointerEvent`). The wheel is reported apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerInput {
    /// [`PointerId::MOUSE`] for the mouse; a contact's own id for as long
    /// as it lasts.
    pub id: PointerId,
    /// Mouse, touch or pen.
    pub kind: PointerKind,
    /// What happened.
    pub phase: PointerPhase,
    /// The buttons held after this event: a `Down` adds its button, an `Up`
    /// removes it, a `Cancel` holds none.
    pub buttons: PointerButtons,
    /// The button a `Down` pressed or an `Up` released; `None` for every
    /// other phase.
    pub button: Option<crate::MouseButton>,
    /// Where the pointer is, in logical window pixels. A mouse event whose
    /// platform message has no position carries the last one.
    pub position: crate::Point,
    /// Pressure from 0 to 1, when the device reports it.
    pub pressure: Option<f32>,
    /// The pen's tilt from the vertical (degrees, along x and y), when the
    /// device reports it.
    pub tilt: Option<(f32, f32)>,
    /// Which click of a multi-click a `Down` is (1 single, 2 double ..),
    /// counted by the window from the system's double-click time and
    /// distance (per axis) from the last press of the same button; an `Up`
    /// carries the count of the press it ends, each held button its own.
    /// A contact's `Down` counts up from the last contact's `Down` the same
    /// way and its other events carry that count. What is given to a test
    /// window is replaced by its count.
    pub clicks: u8,
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

/// The system's mouse settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouseSettings {
    /// A second press within this time (ns) of the first counts as a
    /// double click ...
    pub double_click_ns: u64,
    /// ... when it is within this distance (logical pixels, per axis).
    pub double_click_distance: (f32, f32),
    /// Lines one notch of the wheel scrolls.
    pub wheel_lines: u32,
    /// Characters one notch of the wheel scrolls sideways (Shift).
    pub wheel_chars: u32,
}

impl Default for MouseSettings {
    /// Windows' defaults: 500 ms, 4 pixels, 3 lines, 3 characters.
    fn default() -> Self {
        Self {
            double_click_ns: 500_000_000,
            double_click_distance: (4.0, 4.0),
            wheel_lines: 3,
            wheel_chars: 3,
        }
    }
}

/// What a window's surroundings are like, in one snapshot. A change (but
/// of the scale, which arrives with the resize it causes) is announced by
/// one change event; the snapshot then holds the new values.
#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    /// Physical pixels per logical pixel.
    pub scale: f32,
    /// The refresh period (ns) of the display the window is on.
    pub frame_period_ns: u64,
    /// The system asks to reduce motion.
    pub reduced_motion: bool,
    /// The system prefers a dark theme (`false` where it cannot tell).
    pub dark: bool,
    /// The part of each edge of the window that system UI (a notch, a
    /// status bar, rounded corners, a home indicator) may cover: content
    /// meant to be seen and touched stays inside. Zero on desktops.
    pub safe_area: Insets,
    /// The system's mouse settings (the defaults where it cannot tell).
    pub mouse: MouseSettings,
    /// The system's language and region as a BCP 47 tag (`"ja-JP"`), or
    /// `None` where it cannot tell.
    pub locale: Option<String>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            scale: 1.0,
            frame_period_ns: 16_666_667,
            reduced_motion: false,
            dark: false,
            safe_area: Insets::default(),
            mouse: MouseSettings::default(),
            locale: None,
        }
    }
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

/// A host window's raw handle value; this type never dereferences it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParentWindow {
    /// A Win32 `HWND`.
    Win32(std::num::NonZeroIsize),
    /// An X11 window id.
    X11(std::num::NonZeroU32),
}
