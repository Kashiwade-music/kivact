//! Role: The `Path` value and the commands that build it: lines, curves, arcs and shapes.

use crate::geometry::{CornerRadius, Point, Rect};
use kivact_runtime::UiError;
use kivact_types as p;
use kivact_types::FillRule;
use svgtypes::{SimplePathSegment, SimplifyingPathParser};

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

    /// A path of these commands, not even-odd (for outlines the shaper gives).
    pub(crate) fn from_cmds(cmds: Vec<p::PathCmd>) -> Self {
        Path {
            cmds,
            even_odd: false,
        }
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
    /// Parse SVG path data, including relative, repeated and smooth commands.
    /// Invalid data returns an error without a partial path.
    pub fn parse_svg(d: &str) -> Result<Path, UiError> {
        let mut path = Self::new();
        for segment in SimplifyingPathParser::from(d) {
            path = match segment.map_err(UiError::from)? {
                SimplePathSegment::MoveTo { x, y } => path.move_to(Point::new(x, y)),
                SimplePathSegment::LineTo { x, y } => path.line_to(Point::new(x, y)),
                SimplePathSegment::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                } => path.cubic_to(Point::new(x1, y1), Point::new(x2, y2), Point::new(x, y)),
                SimplePathSegment::Quadratic { x1, y1, x, y } => {
                    path.quad_to(Point::new(x1, y1), Point::new(x, y))
                }
                SimplePathSegment::ClosePath => path.close(),
            };
        }
        if !path.cmds.is_empty() && path.bez().is_none() {
            return Err(UiError::msg("SVG path contains non-finite coordinates"));
        }
        Ok(path)
    }
    /// Round the corner between the current point, `p1` and `p2`, as canvas arcTo.
    /// An empty path moves to `p1`; zero radius or collinear points draw a line.
    /// Negative or non-finite radii and non-finite points leave the path unchanged.
    pub fn arc_to(self, p1: Point, p2: Point, radius: impl Into<f64>) -> Self {
        let radius = radius.into();
        if radius < 0.0
            || ![radius, p1.x, p1.y, p2.x, p2.y]
                .iter()
                .all(|v| v.is_finite())
        {
            return self;
        }
        let mut current = None;
        let mut start = Point::default();
        for command in self.lowered() {
            match command {
                p::PathCmd::MoveTo(point) => {
                    start = point;
                    current = Some(point);
                }
                p::PathCmd::LineTo(point)
                | p::PathCmd::QuadTo(_, point)
                | p::PathCmd::CubicTo(_, _, point) => current = Some(point),
                p::PathCmd::Close => current = Some(start),
                p::PathCmd::Arc { .. } | p::PathCmd::ArcTo { .. } => {}
            }
        }
        match current {
            Some(p0) => self.corner_arc(p0, p1, p2, radius),
            None => self.move_to(p1),
        }
    }
    fn corner_arc(self, p0: Point, p1: Point, p2: Point, radius: f64) -> Self {
        let (ax, ay, bx, by) = (p0.x - p1.x, p0.y - p1.y, p2.x - p1.x, p2.y - p1.y);
        let (a, b) = (ax.hypot(ay), bx.hypot(by));
        let cross = ax * by - ay * bx;
        if radius == 0.0 || a == 0.0 || b == 0.0 || cross == 0.0 {
            return self.line_to(p1);
        }
        let (ax, ay, bx, by) = (ax / a, ay / a, bx / b, by / b);
        let dot = (ax * bx + ay * by).clamp(-1.0, 1.0);
        let distance = radius * ((1.0 + dot) / (1.0 - dot)).sqrt();
        let start = Point::new(p1.x + ax * distance, p1.y + ay * distance);
        let end = Point::new(p1.x + bx * distance, p1.y + by * distance);
        self.line_to(start)
            .svg_arc_to((radius, radius), 0, false, cross < 0.0, end)
    }
    /// SVG's elliptical `A` segment from the current point to `to`.
    pub fn svg_arc_to(
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
            .svg_arc_to(r[1], 0, false, true, Point::new(x + w, y + r[1].1));
        self = self.line_to(Point::new(x + w, y + h - r[2].1)).svg_arc_to(
            r[2],
            0,
            false,
            true,
            Point::new(x + w - r[2].0, y + h),
        );
        self = self.line_to(Point::new(x + r[3].0, y + h)).svg_arc_to(
            r[3],
            0,
            false,
            true,
            Point::new(x, y + h - r[3].1),
        );
        self.line_to(Point::new(x, y + r[0].1))
            .svg_arc_to(r[0], 0, false, true, Point::new(x + r[0].0, y))
            .close()
    }
    pub(crate) fn wire(&self) -> p::DrawingShape {
        p::DrawingShape::Path {
            path: p::PathData {
                cmds: self.cmds.clone(),
            },
            rule: if self.even_odd {
                p::FillRule::EvenOdd
            } else {
                p::FillRule::NonZero
            },
        }
    }
}
