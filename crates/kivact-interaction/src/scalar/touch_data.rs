//! Role: Passive configuration, contact samples and release-coast data.
use super::track_data::Grip;
use crate::DragVelocity;
use kivact_core::geometry::Point;
use kivact_core::view::PointerId;
use std::rc::Rc;
use std::time::Duration;
/// Geometry and release timing for a scalar's opt-in touch path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TouchSliderGesture {
    /// Axis displacement required before a thumb begins tracking.
    pub slop_px: f64,
    /// Velocity multiplier for the coast destination.
    pub projection: Duration,
    /// Natural response time of the critically damped release spring.
    pub response: Duration,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Travel {
    Still,
    Moved,
}
#[derive(Clone)]
pub(super) struct Contact {
    pub(super) id: PointerId,
    pub(super) grip: Grip,
    pub(super) origin: Point,
    pub(super) initial: f64,
    pub(super) travel: Travel,
    pub(super) velocity: Rc<DragVelocity>,
}
impl PartialEq for Contact {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.grip == other.grip
            && self.origin == other.origin
            && self.initial == other.initial
            && self.travel == other.travel
            && Rc::ptr_eq(&self.velocity, &other.velocity)
    }
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Coast {
    pub(super) index: usize,
    pub(super) start: u64,
    pub(super) from: f64,
    pub(super) target: f64,
    pub(super) velocity: f64,
    pub(super) units_per_pixel: f64,
}
