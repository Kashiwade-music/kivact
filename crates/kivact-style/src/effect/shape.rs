//! Role: Basic shapes (`Shape`) and the clip and draw forms the engine takes.

use kivact_types as p;
use kivact_types::{ClipShape, Corners};

use crate::geometry::{CornerRadius, Rect, corner};
use crate::length::Length;
use crate::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ShapeKind {
    Path(Path),
    Inset {
        edges: [Length; 4],
        radius: Option<CornerRadius>,
    },
    Circle {
        radius: Length,
        at: (Length, Length),
    },
    Ellipse {
        rx: Length,
        ry: Length,
        at: (Length, Length),
    },
    Polygon {
        points: Vec<(Length, Length)>,
        even_odd: bool,
    },
}

/// A shape in a box (CSS basic shapes, as `clip-path` takes them): x
/// against the box's width, y against its height; `pct(100) - 18` is 18 px
/// from the right or bottom edge.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{pct};
/// use kivact::draw::Shape;
/// // CSS `clip-path: polygon(0 0, calc(100% - 18px) 0, 100% 18px, 100% 100%, 0 100%)`
/// let cut_corner = Shape::polygon()
///     .point(0, 0)
///     .point(pct(100) - 18, 0)
///     .point(pct(100), 18)
///     .point(pct(100), pct(100))
///     .point(0, pct(100));
/// let s = style().clip_path(cut_corner);
/// # let _ = s;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Shape(pub(crate) ShapeKind);

impl Shape {
    /// A reusable canvas path, shared with element clipping.
    pub fn path(path: Path) -> Shape {
        Shape(ShapeKind::Path(path))
    }
    /// A polygon; add its corners with [`Shape::point`] (CSS `polygon()`).
    pub fn polygon() -> Shape {
        Shape(ShapeKind::Polygon {
            points: Vec::new(),
            even_odd: false,
        })
    }

    /// A polygon's next corner.
    pub fn point(mut self, x: impl Into<Length>, y: impl Into<Length>) -> Shape {
        if let ShapeKind::Polygon { points, .. } = &mut self.0 {
            points.push((x.into(), y.into()));
        }
        self
    }

    /// A polygon: fill by the even-odd rule (CSS `polygon(evenodd, ..)`).
    pub fn even_odd(mut self) -> Shape {
        if let ShapeKind::Polygon { even_odd, .. } = &mut self.0 {
            *even_odd = true;
        }
        self
    }

    /// The box shrunk by each side's length (CSS `inset(t r b l)`).
    pub fn inset(
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
    ) -> Shape {
        Shape(ShapeKind::Inset {
            edges: [top.into(), right.into(), bottom.into(), left.into()],
            radius: None,
        })
    }

    /// An inset shape: round its corners (CSS `inset(.. round r)`).
    pub fn round(mut self, r: impl Into<CornerRadius>) -> Shape {
        if let ShapeKind::Inset { radius, .. } = &mut self.0 {
            *radius = Some(r.into());
        }
        self
    }

    /// A circle of radius `r` at the box's center (CSS `circle(r)`; a
    /// percentage is of `√(w² + h²) / √2`).
    pub fn circle(r: impl Into<Length>) -> Shape {
        Shape(ShapeKind::Circle {
            radius: r.into(),
            at: (Length::Percent(50.0), Length::Percent(50.0)),
        })
    }

    /// An ellipse of radii `rx` × `ry` at the box's center (CSS
    /// `ellipse(rx ry)`).
    pub fn ellipse(rx: impl Into<Length>, ry: impl Into<Length>) -> Shape {
        Shape(ShapeKind::Ellipse {
            rx: rx.into(),
            ry: ry.into(),
            at: (Length::Percent(50.0), Length::Percent(50.0)),
        })
    }

    /// A circle or an ellipse: its center (CSS `at x y`).
    pub fn at(mut self, x: impl Into<Length>, y: impl Into<Length>) -> Shape {
        match &mut self.0 {
            ShapeKind::Circle { at, .. } | ShapeKind::Ellipse { at, .. } => {
                *at = (x.into(), y.into());
            }
            _ => {}
        }
        self
    }
}

/// A shape placed in `rect` as the engine draws it.
pub(crate) fn basic(rect: Rect, shape: &Shape) -> p::DrawingShape {
    p::DrawingShape::Basic {
        rect,
        shape: clip_shape(&shape.0),
    }
}

/// A shape as the engine clips with it.
pub(crate) fn clip_shape(shape: &ShapeKind) -> ClipShape {
    let at = |(x, y): (Length, Length)| [x.to_len(), y.to_len()];
    match shape {
        ShapeKind::Path(path) => ClipShape::Path {
            cmds: path.cmds.clone(),
            even_odd: path.even_odd,
        },
        ShapeKind::Inset { edges, radius } => ClipShape::Inset {
            edges: p::Edges {
                top: edges[0].to_len(),
                right: edges[1].to_len(),
                bottom: edges[2].to_len(),
                left: edges[3].to_len(),
            },
            radius: {
                let c = corner(*radius);
                Corners {
                    top_left: c,
                    top_right: c,
                    bottom_right: c,
                    bottom_left: c,
                }
            },
        },
        ShapeKind::Circle { radius, at: c } => ClipShape::Circle {
            radius: radius.to_len(),
            center: at(*c),
        },
        ShapeKind::Ellipse { rx, ry, at: c } => ClipShape::Ellipse {
            radius_x: rx.to_len(),
            radius_y: ry.to_len(),
            center: at(*c),
        },
        ShapeKind::Polygon { points, even_odd } => ClipShape::Polygon {
            points: points.iter().map(|p| at(*p)).collect(),
            even_odd: *even_odd,
        },
    }
}
