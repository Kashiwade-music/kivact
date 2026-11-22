//! Role: The swipe policy, its phases and the inputs a swipe follows.

use crate::DrawerSwipe;
use kivact_core::geometry::Edge;
use kivact_core::view::PointerKind;

/// Which contacts a swipe follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeInputs {
    All,
    TouchAndPen,
}
impl SwipeInputs {
    pub(super) fn accepts(self, kind: PointerKind) -> bool {
        self == Self::All || matches!(kind, PointerKind::Touch | PointerKind::Pen)
    }
}
/// How a swipe toward `edge` is recognised and when its release dismisses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwipeDismissPolicy {
    /// The edge the surface leaves through.
    pub edge: Edge,
    pub inputs: SwipeInputs,
    /// Logical pixels a contact moves before it is a swipe or not.
    pub slop: f64,
    /// The share of the surface's size along the edge's axis that dismisses.
    pub close_fraction: f64,
    /// The least travel that dismisses by distance.
    pub min_distance: f64,
    /// The least travel of a flick.
    pub flick_distance: f64,
    /// Logical pixels per millisecond toward the edge that make a flick.
    pub flick_speed: f64,
}
impl SwipeDismissPolicy {
    /// A toast card's swipe: touch and pen only, 40 % of its size or 48 px,
    /// or a flick of 48 px faster than 1.2 px/ms.
    pub fn toast(edge: Edge, slop: f64) -> Self {
        Self {
            edge,
            inputs: SwipeInputs::TouchAndPen,
            slop,
            close_fraction: 0.4,
            min_distance: 48.0,
            flick_distance: 48.0,
            flick_speed: 1.2,
        }
    }
    /// A drawer's handle: every pointer, the theme's [`DrawerSwipe`].
    pub fn drawer(edge: Edge, swipe: DrawerSwipe) -> Self {
        Self {
            edge,
            inputs: SwipeInputs::All,
            slop: 0.0,
            close_fraction: swipe.close_fraction,
            min_distance: 0.0,
            flick_distance: swipe.flick_distance,
            flick_speed: swipe.flick_speed,
        }
    }
    pub(crate) fn dismisses(&self, travel: f64, size: f64, speed: f64) -> bool {
        travel >= (size * self.close_fraction).max(self.min_distance)
            || (travel >= self.flick_distance && speed > self.flick_speed)
    }
}
/// Where a swipe is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwipePhase {
    Rest,
    /// A contact is down and has not become a swipe yet.
    Pressed,
    /// The surface follows the contact `offset` px toward its edge; `size` is its extent on that axis.
    Dragging {
        offset: f64,
        size: f64,
    },
    /// A release dismissed the surface `travel` px from its place.
    Dismissed {
        travel: f64,
        size: f64,
    },
}
