//! Role: Share editor marquee geometry and pointer-anchored time-axis zoom.

use kivact::geometry::Point;
use kivact::view::WheelEvent;

const WHEEL_ZOOM: f64 = 1.1;

pub(crate) fn wheel_zoom(
    scale: f64,
    offset: Point,
    e: WheelEvent,
    bounds: (f64, f64),
    anchor_x: f64,
) -> (f64, Point) {
    let scale = scale.clamp(bounds.0, bounds.1);
    if !e.delta().y.is_finite() || e.delta().y == 0.0 || !e.pos().x.is_finite() {
        return (scale, offset);
    }
    let next = (scale
        * if e.delta().y < 0.0 {
            WHEEL_ZOOM
        } else {
            WHEEL_ZOOM.recip()
        })
    .clamp(bounds.0, bounds.1);
    if next == scale {
        return (scale, offset);
    }
    let x = e.pos().x - anchor_x;
    (
        next,
        Point::new(x - (x - offset.x) * (next / scale), offset.y),
    )
}
