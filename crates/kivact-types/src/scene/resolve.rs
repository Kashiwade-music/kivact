//! Role: Resolve recorded draw geometry into outlines and paint areas, shared by the engine and the SVG writer.

use crate::geometry::{Point, Rect};
use crate::paint::{ClipShape, Corners, Len};
use crate::scene::{DrawingShape, FillRule, PathCmd, Radius, RoundedRect, Shape, ShapePaint};

fn positive(v: f64) -> f64 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

fn rule(even_odd: bool) -> FillRule {
    if even_odd {
        FillRule::EvenOdd
    } else {
        FillRule::NonZero
    }
}

impl Corners {
    /// The four radii in `rect` (top left, top right, bottom right, bottom
    /// left): percentages of its width (horizontal) and height (vertical),
    /// never negative.
    pub fn radii_in(&self, rect: Rect) -> [Radius; 4] {
        self.each().map(|cr| Radius {
            x: positive(cr.x.of(rect.width)),
            y: positive(cr.y.of(rect.height)),
        })
    }

    /// The box `rect` with these corners; a plain rect when none is rounded.
    pub fn shape_in(&self, rect: Rect) -> Shape {
        if self.is_zero() {
            return Shape::Rect(rect);
        }
        let radii = self.radii_in(rect);
        if radii.iter().all(|r| r.x == r.y) {
            Shape::RoundedRect(RoundedRect::with_corners(rect, radii.map(|r| r.x)))
        } else {
            Shape::RoundedRect(RoundedRect { rect, radii })
        }
    }
}

impl ClipShape {
    /// The CSS basic shape in the box `rect` (`clip-path` geometry).
    pub fn in_rect(&self, rect: Rect) -> Shape {
        let at = |[x, y]: [Len; 2]| Point {
            x: rect.x + x.of(rect.width),
            y: rect.y + y.of(rect.height),
        };
        let ellipse = |c: Point, rx: f64, ry: f64| {
            Shape::RoundedRect(RoundedRect::ellipse(Rect {
                x: c.x - rx,
                y: c.y - ry,
                width: 2.0 * rx,
                height: 2.0 * ry,
            }))
        };
        match self {
            ClipShape::Path { cmds, even_odd } => Shape::Path {
                cmds: cmds.clone(),
                rule: rule(*even_odd),
            }
            .translated(Point {
                x: rect.x,
                y: rect.y,
            }),
            ClipShape::Inset { edges, radius } => {
                let (left, top) = (edges.left.of(rect.width), edges.top.of(rect.height));
                let inner = Rect {
                    x: rect.x + left,
                    y: rect.y + top,
                    width: (rect.width - left - edges.right.of(rect.width)).max(0.0),
                    height: (rect.height - top - edges.bottom.of(rect.height)).max(0.0),
                };
                radius.shape_in(inner)
            }
            ClipShape::Circle { radius, center } => {
                let reference = rect.width.hypot(rect.height) / std::f64::consts::SQRT_2;
                let r = positive(radius.of(reference));
                ellipse(at(*center), r, r)
            }
            ClipShape::Ellipse {
                radius_x,
                radius_y,
                center,
            } => ellipse(
                at(*center),
                positive(radius_x.of(rect.width)),
                positive(radius_y.of(rect.height)),
            ),
            ClipShape::Polygon { points, even_odd } => Shape::Polygon {
                points: points.iter().map(|p| at(*p)).collect(),
                rule: rule(*even_odd),
            },
        }
    }
}

impl Shape {
    /// The outline moved by `by`, as a path (arcs become curves).
    pub fn translated(self, by: Point) -> Shape {
        let at = |p: Point| Point {
            x: p.x + by.x,
            y: p.y + by.y,
        };
        let rule = self.rule();
        Shape::Path {
            rule,
            cmds: self
                .outline()
                .into_iter()
                .map(|cmd| match cmd {
                    PathCmd::MoveTo(p) => PathCmd::MoveTo(at(p)),
                    PathCmd::LineTo(p) => PathCmd::LineTo(at(p)),
                    PathCmd::QuadTo(p, q) => PathCmd::QuadTo(at(p), at(q)),
                    PathCmd::CubicTo(p, q, r) => PathCmd::CubicTo(at(p), at(q), at(r)),
                    other => other,
                })
                .collect(),
        }
    }
}

impl DrawingShape {
    /// The shape as one outline in the recording's coordinates.
    pub fn to_shape(&self) -> Shape {
        match self {
            DrawingShape::Basic { shape, rect } => shape.in_rect(*rect),
            DrawingShape::Rounded { rect, corners } => corners.shape_in(*rect),
            DrawingShape::Path { path, rule } => Shape::Path {
                cmds: path.cmds.clone(),
                rule: *rule,
            },
        }
    }
}

impl ShapePaint {
    /// The rect a gradient or image paint of this shape spans, in the
    /// recording's coordinates: a CSS shape's reference box or a path's
    /// bounds, padded by half the stroke width where a stroked outline has
    /// no width or no height (a straight line).
    pub fn paint_area(&self) -> Rect {
        let mut area = match &self.shape {
            DrawingShape::Basic { rect, .. } | DrawingShape::Rounded { rect, .. } => *rect,
            DrawingShape::Path { .. } => self.shape.to_shape().bounds(),
        };
        if let Some(stroke) = &self.stroke {
            let pad = stroke.width.max(0.0) / 2.0;
            if area.width == 0.0 {
                area.x -= pad;
                area.width = 2.0 * pad;
            }
            if area.height == 0.0 {
                area.y -= pad;
                area.height = 2.0 * pad;
            }
        }
        area
    }
}
