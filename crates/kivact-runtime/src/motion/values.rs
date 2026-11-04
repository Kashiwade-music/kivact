//! Role: Frame timing values and shared interpolation vocabulary.

pub use kivact_types::Animatable;
use std::time::Duration;

/// Target rate of a frame loop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameRate {
    /// A fixed rate (finite, positive).
    Hz(f64),
    /// Every display refresh.
    Display,
}

/// Passed to frame callbacks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameInfo {
    /// Host clock time of this tick.
    pub now: Duration,
    /// Time since the previous tick of this loop (zero for the first).
    pub delta: Duration,
    /// Tick counter of this loop since `start`.
    pub frame: u64,
    /// Host estimate of when this frame is presented.
    pub present: Duration,
}
