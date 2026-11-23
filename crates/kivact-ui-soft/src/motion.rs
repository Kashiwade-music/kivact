//! Role: The transition durations (ms) and easing curves of Astryx Neutral
//! (crate-private).
use kivact::motion::Easing;

pub(crate) const FAST_MIN: u64 = 95;
pub(crate) const FAST: u64 = 125;
pub(crate) const FAST_MAX: u64 = 165;
pub(crate) const MEDIUM_MIN: u64 = 225;
pub(crate) const MEDIUM: u64 = 300;
pub(crate) const MEDIUM_MAX: u64 = 400;
pub(crate) const SLOW_MIN: u64 = 525;
pub(crate) const EASE_STANDARD: Easing = Easing::CubicBezier(0.24, 1.0, 0.4, 1.0);
pub(crate) const EASE_ENTRANCE: Easing = Easing::CubicBezier(0.24, 1.0, 0.4, 1.0);
