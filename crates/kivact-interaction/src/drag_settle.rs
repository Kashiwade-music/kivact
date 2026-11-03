//! Role: Shared one-axis drag and detent calculations.

use std::time::Duration;

use kivact_core::draw::Point;

/// Physical axis retained by a constrained drag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragAxis {
    Horizontal,
    Vertical,
}

impl DragAxis {
    /// The signed component on this axis, ignoring cross-axis movement.
    pub fn component(self, delta: Point) -> f32 {
        match self {
            Self::Horizontal => delta.x,
            Self::Vertical => delta.y,
        }
    }

    /// The full axis displacement once it strictly exceeds the slop in pixels.
    pub fn past_slop(self, delta: Point, slop: f32) -> Option<f32> {
        let displacement = self.component(delta);
        (displacement.abs() > slop).then_some(displacement)
    }
}

/// Project a release using its velocity in pixels per millisecond and a decay
/// time constant. This predicts the destination without running an animation.
pub fn project_release(position: f32, velocity: f32, decay: Duration) -> f32 {
    position + velocity * (decay.as_secs_f64() * 1000.0) as f32
}

/// Index of the closest stop. Ties select the first stop in the slice.
pub fn nearest_snap(position: f32, stops: &[f32]) -> Option<usize> {
    stops
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (*a - position).abs().total_cmp(&(*b - position).abs()))
        .map(|(index, _)| index)
}

/// Ease toward the closest stop inside a radius using quadratic falloff.
pub fn pull_to_snap(position: f32, stops: &[f32], radius: f32) -> f32 {
    if radius <= 0.0 {
        return position;
    }
    let Some(index) = nearest_snap(position, stops) else {
        return position;
    };
    let distance = stops[index] - position;
    let pull = (1.0 - (distance.abs() / radius).powi(2)).max(0.0);
    position + distance * pull
}

/// Apply linear resistance to signed movement beyond an edge, with a hard cap.
pub fn resist_overshoot(overshoot: f32, resistance: f32, limit: f32) -> f32 {
    let limit = limit.max(0.0);
    (overshoot * resistance).clamp(-limit, limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slop_ignores_cross_axis_and_retains_signed_displacement() {
        assert_eq!(
            DragAxis::Vertical.past_slop(Point::new(40.0, -8.0), 8.0),
            None
        );
        assert_eq!(
            DragAxis::Vertical.past_slop(Point::new(40.0, -9.0), 8.0),
            Some(-9.0)
        );
        assert_eq!(
            DragAxis::Horizontal.past_slop(Point::new(40.0, -9.0), 8.0),
            Some(40.0)
        );
    }

    #[test]
    fn release_projects_signed_velocity_and_selects_first_tied_stop() {
        assert_eq!(project_release(50.0, -0.5, Duration::from_millis(100)), 0.0);
        assert_eq!(project_release(1.0, 1.0, Duration::from_millis(499)), 500.0);
        assert_eq!(nearest_snap(10.0, &[0.0, 20.0]), Some(0));
        assert_eq!(nearest_snap(15.0, &[0.0, 20.0]), Some(1));
        assert_eq!(nearest_snap(15.0, &[]), None);
    }

    #[test]
    fn snap_pull_and_overshoot_preserve_edge_behavior() {
        assert_eq!(pull_to_snap(20.0, &[0.0], 40.0), 5.0);
        assert_eq!(pull_to_snap(40.0, &[0.0], 40.0), 40.0);
        assert_eq!(pull_to_snap(40.0, &[], 40.0), 40.0);
        assert_eq!(resist_overshoot(-20.0, 0.35, 48.0), -7.0);
        assert_eq!(resist_overshoot(-200.0, 0.35, 48.0), -48.0);
        assert_eq!(resist_overshoot(200.0, 0.35, 48.0), 48.0);
    }
}
