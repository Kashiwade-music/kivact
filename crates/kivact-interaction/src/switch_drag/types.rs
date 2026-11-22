//! Role: The thumb geometry, drag states and lift of a switch drag.

/// The thumb geometry a switch drag follows, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchTrack {
    pub width: f64,
    pub height: f64,
    /// The thumb's size while the switch is off.
    pub off_side: f64,
    /// The thumb's size while the switch is on.
    pub on_side: f64,
    /// The inset of the off thumb from the track's start.
    pub off_inset: f64,
}
impl SwitchTrack {
    pub(crate) fn travel(self) -> f64 {
        self.width - 2.0 - self.on_side / 2.0 - self.off_inset - self.off_side / 2.0
    }
    pub(crate) fn band(self, raw: f64, on: bool) -> f64 {
        let end = self.travel();
        let inset = if on { 2.0 } else { self.off_inset };
        let dimension = (12.0 * self.height / 28.0).min(inset / 2.0);
        let clamped = raw.clamp(0.0, end);
        let excess = raw - clamped;
        clamped
            + excess.signum() * (1.0 - 1.0 / (excess.abs() * 0.55 / dimension + 1.0)) * dimension
    }
}

/// The visual position of a switch thumb during an opt-in gesture.
///
/// Only the combinations the gesture machine reaches exist: a touch press, a
/// tracked contact (touch with its offset, or a pointer), and a released touch
/// thumb which is still raised or has settled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThumbDrag {
    /// A touch contact pressed the switch before moving it.
    TouchPressed,
    /// The contact moves the thumb.
    Tracking(TrackInput),
    /// A touch contact was released.
    TouchReleased(Lift),
}

impl ThumbDrag {
    /// Whether the gesture follows a touch contact.
    pub fn is_touch(self) -> bool {
        match self {
            Self::TouchPressed | Self::TouchReleased(_) => true,
            Self::Tracking(TrackInput::Touch { .. }) => true,
            Self::Tracking(TrackInput::Pointer { .. }) => false,
        }
    }
}

/// What a tracked switch gesture follows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackInput {
    /// A touch contact, at a physical offset along the writing direction.
    Touch {
        /// Physical translation along the writing direction, in logical pixels.
        offset: f64,
        /// The side the thumb shows before the binding commits.
        on: bool,
    },
    /// A mouse or pen pointer.
    Pointer {
        /// The side the thumb shows before the binding commits.
        on: bool,
    },
}

/// Whether a released touch thumb is still raised or has settled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lift {
    /// The thumb is still raised above the track.
    Raised,
    /// The thumb has settled back onto the track.
    Settled,
}
