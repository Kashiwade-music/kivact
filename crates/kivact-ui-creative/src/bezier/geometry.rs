//! Role: Finite plot coordinates shared by curve painting and hit regions.
use kivact::draw::{Point, Rect, Size};
#[derive(Clone, Copy)]
pub(crate) struct Plane {
    pub(crate) rect: Rect,
    low: f64,
    span: f64,
    weight: f64,
}
impl Plane {
    pub(crate) fn new(size: Size, low: f64, high: f64) -> Self {
        let clean = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
        let (w, h) = (clean(size.width), clean(size.height));
        let pad = (w.min(h) / 4.0).min(18.0);
        let weight = low.abs().max(high.abs()).max(1.0);
        let lo = low / weight;
        let span = (high / weight - lo).max(f64::EPSILON);
        Self {
            rect: Rect::new(pad, pad, (w - 2.0 * pad).max(0.0), (h - 2.0 * pad).max(0.0)),
            low: lo,
            span,
            weight,
        }
    }
    pub(crate) fn point(self, x: f64, y: f64) -> Point {
        Point::new(
            self.rect.x + (x.clamp(0.0, 1.0) as f32) * self.rect.width,
            self.rect.y + self.rect.height
                - (((y / self.weight - self.low) / self.span) as f32) * self.rect.height,
        )
    }
    pub(crate) fn moved(self, point: (f64, f64), delta: Point) -> (f64, f64) {
        (
            point.0 + f64::from(delta.x / self.rect.width.max(1.0)),
            (point.1 / self.weight - f64::from(delta.y / self.rect.height.max(1.0)) * self.span)
                * self.weight,
        )
    }
    pub(crate) fn hit(self, point: Point) -> Rect {
        let radius = (self.rect.width.min(self.rect.height) / 4.0).clamp(1.0, 8.0);
        Rect::new(
            point.x - radius,
            point.y - radius,
            2.0 * radius,
            2.0 * radius,
        )
    }
}
