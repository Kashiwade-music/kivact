//! Role: The `Path` value and the commands that build it: lines, curves, arcs and shapes.

use crate::geometry::{CornerRadius, Point, Rect};
use kivact_types as p;
use kivact_types::FillRule;

/// A reusable path, filled with `Draw::fill_path`, stroked with
/// `Draw::stroke_path`, and used as a clip, a hit region or a style clip
/// shape. Angles are degrees, clockwise on the screen.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    pub(crate) cmds: Vec<p::PathCmd>,
    pub(crate) even_odd: bool,
}
impl Path {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn move_to(mut self, point: Point) -> Self {
        self.cmds.push(p::PathCmd::MoveTo(point));
        self
    }
    pub fn line_to(mut self, point: Point) -> Self {
        self.cmds.push(p::PathCmd::LineTo(point));
        self
    }
    pub fn quad_to(mut self, control: Point, point: Point) -> Self {
        self.cmds.push(p::PathCmd::QuadTo(control, point));
        self
    }
    pub fn cubic_to(mut self, first: Point, second: Point, point: Point) -> Self {
        self.cmds.push(p::PathCmd::CubicTo(first, second, point));
        self
    }
    pub fn close(mut self) -> Self {
        self.cmds.push(p::PathCmd::Close);
        self
    }
    /// The rule used when this path is filled.
    pub fn fill_rule(&self) -> FillRule {
        if self.even_odd {
            FillRule::EvenOdd
        } else {
            FillRule::NonZero
        }
    }
    /// Set the rule used when this path is filled.
    pub fn with_fill_rule(mut self, rule: FillRule) -> Self {
        self.even_odd = rule == FillRule::EvenOdd;
        self
    }
    /// A circular arc from `start` to `end`. Zero degrees is to the right.
    pub fn arc(
        self,
        center: Point,
        radius: impl Into<f64>,
        start: impl Into<f64>,
        end: impl Into<f64>,
        counterclockwise: bool,
    ) -> Self {
        let r = radius.into();
        self.ellipse(
            center,
            (r, r),
            0,
            (start.into(), end.into()),
            counterclockwise,
        )
    }
    /// An elliptical arc: radii, rotation, start/end angles and direction.
    pub fn ellipse(
        mut self,
        center: Point,
        radii: (impl Into<f64>, impl Into<f64>),
        rotation: impl Into<f64>,
        angles: (impl Into<f64>, impl Into<f64>),
        counterclockwise: bool,
    ) -> Self {
        let (start, end) = (angles.0.into(), angles.1.into());
        let delta = end - start;
        let sweep =
            if (!counterclockwise && delta >= 360.0) || (counterclockwise && delta <= -360.0) {
                if counterclockwise { -360.0 } else { 360.0 }
            } else if counterclockwise {
                -(-delta).rem_euclid(360.0)
            } else {
                delta.rem_euclid(360.0)
            };
        self.cmds.push(p::PathCmd::Arc {
            center,
            rx: radii.0.into(),
            ry: radii.1.into(),
            x_rotation_deg: rotation.into(),
            start_deg: start,
            sweep_deg: sweep,
        });
        self
    }
    /// SVG's elliptical `A` segment from the current point to `to`.
    pub fn arc_to(
        mut self,
        radii: (impl Into<f64>, impl Into<f64>),
        rotation: impl Into<f64>,
        large_arc: bool,
        sweep: bool,
        to: Point,
    ) -> Self {
        self.cmds.push(p::PathCmd::ArcTo {
            rx: radii.0.into(),
            ry: radii.1.into(),
            x_rotation_deg: rotation.into(),
            large_arc,
            sweep,
            to,
        });
        self
    }
    pub fn rect(self, rect: Rect) -> Self {
        self.move_to(Point::new(rect.x, rect.y))
            .line_to(Point::new(rect.x + rect.width, rect.y))
            .line_to(Point::new(rect.x + rect.width, rect.y + rect.height))
            .line_to(Point::new(rect.x, rect.y + rect.height))
            .close()
    }
    /// Clockwise corners, starting at the top left; elliptical and percentage radii match styles.
    pub fn rounded_rect(mut self, rect: Rect, corners: [CornerRadius; 4]) -> Self {
        let mut r = corners.map(|r| {
            (
                r.x.to_len().of(rect.width).max(0.0),
                r.y.to_len().of(rect.height).max(0.0),
            )
        });
        let ratio = |length: f64, sum: f64| if sum > length { length / sum } else { 1.0 };
        let scale = ratio(rect.width, r[0].0 + r[1].0)
            .min(ratio(rect.width, r[3].0 + r[2].0))
            .min(ratio(rect.height, r[0].1 + r[3].1))
            .min(ratio(rect.height, r[1].1 + r[2].1));
        for (x, y) in &mut r {
            *x *= scale;
            *y *= scale;
        }
        let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
        self = self
            .move_to(Point::new(x + r[0].0, y))
            .line_to(Point::new(x + w - r[1].0, y))
            .arc_to(r[1], 0, false, true, Point::new(x + w, y + r[1].1));
        self = self.line_to(Point::new(x + w, y + h - r[2].1)).arc_to(
            r[2],
            0,
            false,
            true,
            Point::new(x + w - r[2].0, y + h),
        );
        self = self.line_to(Point::new(x + r[3].0, y + h)).arc_to(
            r[3],
            0,
            false,
            true,
            Point::new(x, y + h - r[3].1),
        );
        self.line_to(Point::new(x, y + r[0].1))
            .arc_to(r[0], 0, false, true, Point::new(x + r[0].0, y))
            .close()
    }
    pub(crate) fn wire(&self) -> p::DrawingShape {
        p::DrawingShape::Path {
            cmds: self.cmds.clone(),
            even_odd: self.even_odd,
        }
    }
}
