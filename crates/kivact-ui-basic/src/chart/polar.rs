//! Role: Angle helpers shared by the pie, radial and radar charts.

use kivact::draw::{Point, Size};

/// The edge kept free around a polar plot.
pub(super) const MARGIN: f32 = 8.0;

/// The middle of the canvas and the largest radius that keeps the margin.
pub(super) fn disc(size: Size) -> (Point, f32) {
    (
        Point::new(size.width / 2.0, size.height / 2.0),
        (size.width.min(size.height) / 2.0 - MARGIN).max(1.0),
    )
}

/// The point `radius` away from `center` at `degrees`, clockwise from the right.
pub(super) fn polar(center: Point, radius: f32, degrees: f32) -> Point {
    let radians = degrees.to_radians();
    Point::new(
        center.x + radius * radians.cos(),
        center.y + radius * radians.sin(),
    )
}

/// The distance of `at` from `center` and its angle in degrees from the top, clockwise, in `0..360`.
pub(super) fn locate(center: Point, at: Point) -> (f32, f32) {
    let (dx, dy) = (at.x - center.x, at.y - center.y);
    (
        dx.hypot(dy),
        (dy.atan2(dx).to_degrees() + 90.0).rem_euclid(360.0),
    )
}
