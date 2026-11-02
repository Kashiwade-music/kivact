//! Role: What a window reports: environment, mouse, pointer input, parent.
//!
//! Positions are logical window pixels, times nanoseconds.

use crate::geometry::Point;
use crate::platform::{Insets, PointerButtons, PointerId, PointerKind, PointerPhase};

/// One event of the mouse, a touch or a pen, as a window delivers it
/// (Pointer Events' `PointerEvent`). The wheel is reported apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerInput {
    /// [`PointerId::MOUSE`] for the mouse; a contact's own id for as long
    /// as it lasts.
    pub id: PointerId,
    /// Mouse, touch or pen.
    pub kind: PointerKind,
    /// What happened; a `Down`'s or `Up`'s button and click count are in it. A
    /// window creates a `Down` or `Up` with `clicks: 1` and its click counter
    /// (`arvane_shell::ClickCounter`) replaces the count; what is given to a
    /// test window is replaced by its count.
    pub phase: PointerPhase,
    /// The buttons held after this event: a `Down` adds its button, an `Up`
    /// removes it, a `Cancel` holds none.
    pub buttons: PointerButtons,
    /// Where the pointer is, in logical window pixels. A mouse event whose
    /// platform message has no position carries the last one.
    pub position: Point,
    /// Pressure from 0 to 1, when the device reports it.
    pub pressure: Option<f32>,
    /// The pen's tilt from the vertical (degrees, along x and y), when the
    /// device reports it.
    pub tilt: Option<(f32, f32)>,
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

/// A host window's raw handle value; this type never dereferences it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParentWindow {
    /// A Win32 `HWND`.
    Win32(std::num::NonZeroIsize),
    /// An X11 window id.
    X11(std::num::NonZeroU32),
}
