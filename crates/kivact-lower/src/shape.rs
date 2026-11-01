//! Role: Box, padding box and clip path shapes shared by painting and hit testing.
//!
//! The shapes of a node's box, resolved from its paint at its rect: the
//! rounded border box, the padding box its children are clipped to, and
//! its clip path. Painting and hit testing both take them from here, so
//! what is drawn and what can be pressed agree (one outline:
//! `kivact_types::Shape::contains` follows `Shape::outline`).
//!
//! Invariants:
//! - Percentages of a corner radius are of the box's width (horizontal)
//!   and height (vertical); radii that do not fit are scaled down together
//!   (CSS, `RoundedRect::fitted_radii`).
//! - The padding box is the border box inset by the border widths, its
//!   radii shrunk by them (CSS inner radii).
//! - Hit testing (G-07, `layout::geometry`): a node is hit only inside its
//!   rounded border box ([`hits`]); outside its clip path neither it nor
//!   its descendants are ([`in_clip_path`]); the children of a clipping
//!   node are hit only inside its rounded padding box ([`in_child_clip`]).
//!   `Draw::region`, hover and the test driver's actionability go through
//!   the same hit test, so they agree with what is drawn. The hit shapes
//!   are those of the node's base paint: a state style that changes its
//!   corners or clip path changes how it is drawn, not where it is hit.

use kivact_wire::{Border, ClipShape, Corners, Edges, Len, PaintProps, Point, Rect};

use kivact_types::{FillRule, Radius, RoundedRect, Shape};

/// A box's outline: a plain rect, circular corners (drawn with Arvane's
/// rounded-rect commands, as before elliptical corners existed) or
/// elliptical ones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoxShape {
    /// An axis-aligned logical rectangle without rounded corners.
    Rect(Rect),
    /// Top-left, top-right, bottom-right, bottom-left radii.
    Circular(Rect, [f32; 4]),
    /// Four elliptical radii with their logical reference rectangle.
    Rounded(RoundedRect),
}

impl BoxShape {
    /// The box `rect` with the corners `radius`.
    pub fn of(rect: Rect, radius: &Corners) -> BoxShape {
        let r = rect;
        if radius.is_zero() {
            return BoxShape::Rect(r);
        }
        let radii = radius.each().map(|cr| Radius {
            x: positive(cr.x.of(rect.width)),
            y: positive(cr.y.of(rect.height)),
        });
        if radii.iter().all(|r| r.x == r.y) {
            BoxShape::Circular(r, radii.map(|r| r.x))
        } else {
            BoxShape::Rounded(RoundedRect { rect: r, radii })
        }
    }

    /// The outer logical bounding rectangle.
    pub fn rect(&self) -> Rect {
        match self {
            BoxShape::Rect(r) | BoxShape::Circular(r, _) => *r,
            BoxShape::Rounded(rr) => rr.rect,
        }
    }

    /// The outline as a rounded rect (CSS-fitted radii).
    pub(crate) fn rounded(&self) -> RoundedRect {
        match *self {
            BoxShape::Rect(r) => RoundedRect::new(r, 0.0),
            BoxShape::Circular(r, radii) => RoundedRect::with_corners(r, radii),
            BoxShape::Rounded(rr) => rr,
        }
    }

    /// The equivalent neutral outline used for drawing and containment.
    pub fn shape(&self) -> Shape {
        match self {
            BoxShape::Rect(r) => Shape::Rect(*r),
            _ => Shape::RoundedRect(self.rounded()),
        }
    }

    /// The box grown (or shrunk, negative) by `by` on every side, its radii
    /// grown alike (a focus ring's outline).
    pub(crate) fn grown(&self, by: f32) -> BoxShape {
        let grow = |r: Rect| Rect {
            x: r.x - by,
            y: r.y - by,
            width: (r.width + 2.0 * by).max(0.0),
            height: (r.height + 2.0 * by).max(0.0),
        };
        match *self {
            BoxShape::Rect(r) => BoxShape::Rect(grow(r)),
            BoxShape::Circular(r, radii) => {
                BoxShape::Circular(grow(r), radii.map(|v| (v + by).max(0.0)))
            }
            BoxShape::Rounded(rr) => BoxShape::Rounded(RoundedRect {
                rect: grow(rr.rect),
                radii: rr.fitted_radii().map(|r| Radius {
                    x: (r.x + by).max(0.0),
                    y: (r.y + by).max(0.0),
                }),
            }),
        }
    }

    /// The box inset by `widths` (top, right, bottom, left), its radii
    /// shrunk by them per axis (the padding box inside a border).
    pub fn inset(&self, widths: Edges<f32>) -> BoxShape {
        if widths == Edges::all(0.0) {
            return *self;
        }
        let r = self.rect();
        let inner = Rect {
            x: r.x + widths.left,
            y: r.y + widths.top,
            width: (r.width - widths.left - widths.right).max(0.0),
            height: (r.height - widths.top - widths.bottom).max(0.0),
        };
        if let BoxShape::Rect(_) = self {
            return BoxShape::Rect(inner);
        }
        let radii = self.rounded().fitted_radii();
        let shrink = |r: Radius, x: f32, y: f32| Radius {
            x: (r.x - x).max(0.0),
            y: (r.y - y).max(0.0),
        };
        BoxShape::Rounded(RoundedRect {
            rect: inner,
            radii: [
                shrink(radii[0], widths.left, widths.top),
                shrink(radii[1], widths.right, widths.top),
                shrink(radii[2], widths.right, widths.bottom),
                shrink(radii[3], widths.left, widths.bottom),
            ],
        })
    }

    /// Test a logical point against the same outline used for drawing.
    pub(crate) fn contains(&self, p: Point) -> bool {
        match self {
            BoxShape::Rect(r) => r.contains(p),
            _ => self.rounded().contains(p),
        }
    }
}

fn positive(v: f32) -> f32 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

/// The widths of the visible border sides (a side without width or color
/// draws nothing and insets nothing).
pub fn border_widths(border: Option<&Border>) -> Edges<f32> {
    let Some(b) = border else {
        return Edges::all(0.0);
    };
    let side = |w: f32, a: f32| if w > 0.0 && a > 0.0 { w } else { 0.0 };
    Edges {
        top: side(b.widths.top, b.colors.top.a),
        right: side(b.widths.right, b.colors.right.a),
        bottom: side(b.widths.bottom, b.colors.bottom.a),
        left: side(b.widths.left, b.colors.left.a),
    }
}

/// The clip path `shape` in the box `rect` (CSS basic shapes).
pub fn clip_path(shape: &ClipShape, rect: Rect) -> Shape {
    let at = |[x, y]: [Len; 2]| Point {
        x: rect.x + x.of(rect.width),
        y: rect.y + y.of(rect.height),
    };
    match shape {
        ClipShape::Path { cmds, even_odd } => {
            let shape = Shape::Path {
                cmds: cmds.clone(),
                rule: if *even_odd {
                    FillRule::EvenOdd
                } else {
                    FillRule::NonZero
                },
            };
            super::path::shifted(
                shape,
                Point {
                    x: rect.x,
                    y: rect.y,
                },
            )
        }
        ClipShape::Inset { edges, radius } => {
            let inner = Rect {
                x: rect.x + edges.left.of(rect.width),
                y: rect.y + edges.top.of(rect.height),
                width: (rect.width - edges.left.of(rect.width) - edges.right.of(rect.width))
                    .max(0.0),
                height: (rect.height - edges.top.of(rect.height) - edges.bottom.of(rect.height))
                    .max(0.0),
            };
            BoxShape::of(inner, radius).shape()
        }
        ClipShape::Circle { radius, center } => {
            let reference = (rect.width.hypot(rect.height)) / std::f32::consts::SQRT_2;
            let r = positive(radius.of(reference));
            let c = at(*center);
            Shape::RoundedRect(RoundedRect::ellipse(Rect {
                x: c.x - r,
                y: c.y - r,
                width: 2.0 * r,
                height: 2.0 * r,
            }))
        }
        ClipShape::Ellipse {
            radius_x,
            radius_y,
            center,
        } => {
            let (rx, ry) = (
                positive(radius_x.of(rect.width)),
                positive(radius_y.of(rect.height)),
            );
            let c = at(*center);
            Shape::RoundedRect(RoundedRect::ellipse(Rect {
                x: c.x - rx,
                y: c.y - ry,
                width: 2.0 * rx,
                height: 2.0 * ry,
            }))
        }
        ClipShape::Polygon { points, even_odd } => Shape::Polygon {
            points: points.iter().map(|p| at(*p)).collect(),
            rule: if *even_odd {
                FillRule::EvenOdd
            } else {
                FillRule::NonZero
            },
        },
    }
}

/// Whether a node painted with `paint` in its box `rect` is hit at `p`
/// (window coordinates): inside its rounded border box and its clip path.
pub fn hits(paint: Option<&PaintProps>, rect: Rect, p: Point) -> bool {
    if !rect.contains(p) {
        return false;
    }
    let Some(paint) = paint else {
        return true;
    };
    in_clip_path(paint, rect, p) && BoxShape::of(rect, &paint.radius).contains(p)
}

/// Whether `p` lies in the clip path of a node painted with `paint` in
/// `rect` (true without one).
pub fn in_clip_path(paint: &PaintProps, rect: Rect, p: Point) -> bool {
    match paint.effects.as_ref().and_then(|e| e.clip_path.as_ref()) {
        Some(shape) => clip_path(shape, rect).contains(p),
        None => true,
    }
}

/// Whether the children of a clipping node painted with `paint` in `rect`
/// can be hit at `p`: inside its padding box (rounded by its inner radii).
pub fn in_child_clip(paint: Option<&PaintProps>, rect: Rect, p: Point) -> bool {
    match paint {
        Some(paint) => BoxShape::of(rect, &paint.radius)
            .inset(border_widths(paint.border.as_ref()))
            .contains(p),
        None => rect.contains(p),
    }
}
