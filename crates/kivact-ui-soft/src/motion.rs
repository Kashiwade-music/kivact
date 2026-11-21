//! Role: The transition durations (ms) and easing curves of Astryx Neutral (crate-private).
use kivact::motion::Easing;

pub(crate) const FAST_MIN: u64 = 95;
pub(crate) const FAST: u64 = 125;
pub(crate) const FAST_MAX: u64 = 165;
pub(crate) const MEDIUM_MIN: u64 = 225;
pub(crate) const MEDIUM: u64 = 300;
pub(crate) const MEDIUM_MAX: u64 = 400;
pub(crate) const SLOW_MIN: u64 = 525;
pub(crate) const SLOW: u64 = 700;
pub(crate) const SLOW_MAX: u64 = 935;
pub(crate) const EASE_STANDARD: Easing = Easing::CubicBezier(0.24, 1.0, 0.4, 1.0);
pub(crate) const EASE_ENTRANCE: Easing = Easing::CubicBezier(0.24, 1.0, 0.4, 1.0);
pub(crate) const EASE_MOVE: Easing = Easing::CubicBezier(0.24, 1.0, 0.4, 1.0);
pub(crate) const EASE_EXIT: Easing = Easing::CubicBezier(0.3, 0.0, 0.6, 0.6);
pub(crate) const PRESS_ONSET: u64 = 150;
pub(crate) const PRESS_RELEASE: u64 = 200;
pub(crate) const EASE_PRESS_RELEASE: Easing = Easing::CubicBezier(0.0, 0.0, 0.2, 1.0);
