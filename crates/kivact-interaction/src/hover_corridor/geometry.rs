//! Role: Test pointer travel against the facing edge of a hover surface.
use kivact_core::geometry::{Point, Rect};
pub(super) struct Edge {
    first: Point,
    last: Point,
    vertical: bool,
}

impl Edge {
    pub(super) fn facing(origin: Point, rect: Rect) -> Option<Self> {
        const PAD: f64 = 8.0;
        let right = rect.x + rect.width;
        let bottom = rect.y + rect.height;
        let vertical = |x| Self {
            first: Point::new(x, rect.y - PAD),
            last: Point::new(x, bottom + PAD),
            vertical: true,
        };
        let horizontal = |y| Self {
            first: Point::new(rect.x - PAD, y),
            last: Point::new(right + PAD, y),
            vertical: false,
        };
        if rect.width <= 0.0 || rect.height <= 0.0 {
            None
        } else if origin.x < rect.x {
            Some(vertical(rect.x))
        } else if origin.x > right {
            Some(vertical(right))
        } else if origin.y < rect.y {
            Some(horizontal(rect.y))
        } else if origin.y > bottom {
            Some(horizontal(bottom))
        } else {
            None
        }
    }

    pub(super) fn distance(&self, point: Point) -> f64 {
        if self.vertical {
            (self.first.x - point.x).abs()
        } else {
            (self.first.y - point.y).abs()
        }
    }

    pub(super) fn contains(&self, point: Point, origin: Point) -> bool {
        fn cross(a: Point, b: Point, p: Point) -> f64 {
            (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
        }
        let sides = [
            cross(origin, self.first, point),
            cross(self.first, self.last, point),
            cross(self.last, origin, point),
        ];
        sides.iter().all(|&side| side >= 0.0) || sides.iter().all(|&side| side <= 0.0)
    }
}
