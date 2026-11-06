//! Role: Shapes, shadows, filters and blending, and their engine forms.
//!
//! Shapes, shadows, filters and blending: the effects vocabulary of
//! `Style` (and of custom drawing), and how a shape and a filter become the
//! engine's.

use kivact_types as p;
use kivact_types::{ClipShape, Corners, FilterOp};

use super::color::Color;
use super::geometry::{Point, Rect, corner};
use super::length::Length;
use super::theme::value::{Theme, ThemeColor};

/// A box shadow (CSS `box-shadow`); several stack, the first on top.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{rgba};
/// use kivact::draw::Shadow;
/// // CSS `box-shadow: 0 4px 14px -4px rgba(0,0,0,.5), inset 0 1px 0 rgba(255,255,255,.35)`
/// let s = style()
///     .shadow(Shadow::new(0, 4, 14, rgba(0x000000, 0.5)).spread(-4))
///     .shadow(Shadow::new(0, 1, 0, rgba(0xffffff, 0.35)).inset());
/// # let _ = s;
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub offset: Point,
    /// Blur radius (CSS: a gaussian of standard deviation `blur / 2`).
    pub blur: f64,
    /// Growth of the shadow's shape (negative shrinks it).
    pub spread: f64,
    pub color: Color,
    /// Inside the box (CSS `inset`).
    pub inset: bool,
}

impl Shadow {
    /// A shadow offset by `(x, y)` with blur radius `blur`.
    pub fn new(
        x: impl Into<f64>,
        y: impl Into<f64>,
        blur: impl Into<f64>,
        color: impl ThemeColor,
    ) -> Shadow {
        Shadow {
            offset: Point::new(x.into(), y.into()),
            blur: blur.into(),
            spread: 0.0,
            color: Color::of(color),
            inset: false,
        }
    }

    /// Grow (or, negative, shrink) the shadow's shape (CSS spread radius).
    pub fn spread(mut self, spread: impl Into<f64>) -> Shadow {
        self.spread = spread.into();
        self
    }

    /// Draw it inside the box (CSS `inset`).
    pub fn inset(mut self) -> Shadow {
        self.inset = true;
        self
    }
}

/// How a border's line is drawn (CSS `border-style`).
///
/// `Dashed`: Dashes three times as long as the border is wide.
///
/// `Dotted`: Round dots as wide as the border.
#[doc(inline)]
pub use kivact_types::BorderStyle;

pub use super::geometry::CornerRadius;

/// How a node's pixels mix with what is beneath (CSS `mix-blend-mode`).
#[doc(inline)]
pub use kivact_types::BlendMode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum FilterFn {
    Brightness(f64),
    Contrast(f64),
    Saturate(f64),
    Grayscale(f64),
    Sepia(f64),
    HueRotate(f64),
    Invert(f64),
    Opacity(f64),
    Blur(f64),
    DropShadow {
        offset: Point,
        blur: f64,
        color: Color,
    },
}

/// A list of CSS filter functions, applied in order (CSS `filter`,
/// `backdrop-filter`). Amounts are CSS's: `1` leaves the image as it is
/// for brightness, contrast, saturate and opacity, `0` for grayscale,
/// sepia and invert; angles are degrees, lengths px.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{rgba};
/// use kivact::draw::Filter;
/// // CSS `filter: brightness(1.12) drop-shadow(0 0 4px rgba(90,169,255,.5))`
/// let s = style().filter(
///     Filter::new()
///         .brightness(1.12)
///         .drop_shadow(0, 0, 4, rgba(0x5aa9ff, 0.5)),
/// );
/// # let _ = s;
/// ```
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Filter {
    pub(crate) ops: Vec<FilterFn>,
}

macro_rules! filter_fn {
    ($(#[$m:meta])* $name:ident, $variant:ident) => {
        $(#[$m])*
        pub fn $name(mut self, amount: impl Into<f64>) -> Filter {
            self.ops.push(FilterFn::$variant(amount.into()));
            self
        }
    };
}

impl Filter {
    /// No filter functions (add them with the methods).
    pub fn new() -> Filter {
        Filter::default()
    }

    filter_fn!(
        /// CSS `brightness(amount)`: 0 black, 1 unchanged.
        brightness, Brightness);
    filter_fn!(
        /// CSS `contrast(amount)`: 0 gray, 1 unchanged.
        contrast, Contrast);
    filter_fn!(
        /// CSS `saturate(amount)`: 0 gray, 1 unchanged, more is more vivid.
        saturate, Saturate);
    filter_fn!(
        /// CSS `grayscale(amount)`: 0 unchanged, 1 gray.
        grayscale, Grayscale);
    filter_fn!(
        /// CSS `sepia(amount)`: 0 unchanged, 1 sepia.
        sepia, Sepia);
    filter_fn!(
        /// CSS `hue-rotate(deg)`.
        hue_rotate, HueRotate);
    filter_fn!(
        /// CSS `invert(amount)`: 0 unchanged, 1 inverted.
        invert, Invert);
    filter_fn!(
        /// CSS `opacity(amount)`: 1 unchanged, 0 transparent.
        opacity, Opacity);
    filter_fn!(
        /// CSS `blur(px)`: a gaussian of standard deviation `px`.
        blur, Blur);

    /// CSS `drop-shadow(x y blur color)`: a blurred copy of the
    /// silhouette beneath (it follows the drawn shape, unlike a box
    /// shadow).
    pub fn drop_shadow(
        mut self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        blur: impl Into<f64>,
        color: impl ThemeColor,
    ) -> Filter {
        self.ops.push(FilterFn::DropShadow {
            offset: Point::new(x.into(), y.into()),
            blur: blur.into(),
            color: Color::of(color),
        });
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ShapeKind {
    Path(super::path::Path),
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
    pub fn path(path: super::path::Path) -> Shape {
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
pub(super) fn basic(rect: Rect, shape: &Shape) -> p::DrawingShape {
    p::DrawingShape::Basic {
        rect,
        shape: clip_shape(&shape.0),
    }
}

/// A shape as the engine clips with it.
pub(super) fn clip_shape(shape: &ShapeKind) -> ClipShape {
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

/// A filter resolved against `theme` as the engine applies it.
pub(crate) fn filter_ops(f: &Filter, theme: &Theme) -> Vec<FilterOp> {
    f.ops
        .iter()
        .map(|op| match *op {
            FilterFn::Brightness(a) => FilterOp::Brightness(a),
            FilterFn::Contrast(a) => FilterOp::Contrast(a),
            FilterFn::Saturate(a) => FilterOp::Saturate(a),
            FilterFn::Grayscale(a) => FilterOp::Grayscale(a),
            FilterFn::Sepia(a) => FilterOp::Sepia(a),
            FilterFn::HueRotate(a) => FilterOp::HueRotate(a),
            FilterFn::Invert(a) => FilterOp::Invert(a),
            FilterFn::Opacity(a) => FilterOp::Opacity(a),
            FilterFn::Blur(a) => FilterOp::Blur(a),
            FilterFn::DropShadow {
                offset,
                blur,
                color,
            } => FilterOp::DropShadow {
                offset,
                blur,
                color: color.resolve(theme),
            },
        })
        .collect()
}
