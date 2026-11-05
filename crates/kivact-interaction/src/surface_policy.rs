//! Role: Configure delayed preview opening and drawer dismissal gestures.
use std::time::Duration;

/// Delays for pointer entry and departure from a hover preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoverCardTiming {
    pub open_delay: Duration,
    pub close_delay: Duration,
}
impl Default for HoverCardTiming {
    fn default() -> Self {
        Self {
            open_delay: Duration::from_millis(600),
            close_delay: Duration::from_millis(300),
        }
    }
}
/// Distance and velocity thresholds for dismissing a drawer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawerSwipe {
    pub close_fraction: f64,
    /// Logical pixels per millisecond.
    pub flick_speed: f64,
    /// Minimum logical pixel travel for a flick.
    pub flick_distance: f64,
}
impl Default for DrawerSwipe {
    fn default() -> Self {
        Self {
            close_fraction: 0.3,
            flick_speed: 0.6,
            flick_distance: 4.0,
        }
    }
}
/// Drawer gesture policy.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrawerPolicy {
    pub swipe: DrawerSwipe,
}
