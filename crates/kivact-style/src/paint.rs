//! Role: Fills: colors, gradients and images, and the engine's fill layers.
//!
//! What boxes and drawings are filled with: colors, gradients and images,
//! sized, placed and repeated like CSS background layers. The same
//! [`Paint`] fills a box (`style().background(..)`, `mask(..)`) and, in
//! custom drawing, a shape (`Draw`); both resolve it here into the engine's
//! fill layer.
//!
//! The element paint vocabulary is CSS's property names with typed values:
//! `background` ([`Background`]: a stack of [`Paint`]s written as a tuple,
//! the first on top), `border`, `border_top` .. `border_left`,
//! `border_width`, `border_color` and `border_style` (solid, dashed,
//! dotted: width and color per side, one style for all sides), `radius`,
//! `radius_corners`, `radius_top_left` .. (`CornerRadius`: px, percent or
//! an `(x, y)` ellipse), `shadow` (`Shadow::new(x, y, blur,
//! color).spread(s).inset()`, the first on top), `opacity` (a group),
//! `filter`, `backdrop_filter` and `backdrop_blur` (`Filter`: a list of CSS
//! filter functions), `mix_blend_mode` (`BlendMode`), `mask` (a `Paint`)
//! and `clip_path` (`Shape`: polygon, inset, circle, ellipse). These values
//! are `Draw`'s too and live in `kivact::draw`, except `Background` and
//! `BorderStyle`, which only a style takes (`kivact::style`); the prelude
//! keeps the everyday names only.
//!
//! A `Paint` is one CSS background layer: a color, a [`Gradient`] or an
//! image, with its tile size, position and repeat; `Draw::fill_rect` fills
//! with the same type (percentages are of the filled rectangle). A gradient
//! is `linear_gradient(angle)` (CSS angles: 0 is up, 90 right),
//! `radial_gradient()` (circle or ellipse, extent, radius, center) or
//! `conic_gradient(start)`, optionally `repeating()`, with `stop(color,
//! position)` (px or percent, degrees for a conic one) or `colors([..])`
//! without positions. A calc `Length` (`pct(100) - 20`) is resolved when
//! drawn wherever a length is drawn (background size and position, gradient
//! centers and stops, corner radii, clip paths, masks).

use kivact_wire as p;
use kivact_wire::{FillImage, FillLayer, FillSize, GradientKind as WireKind, GradientStop};

use super::asset::Asset;
use super::color::Color;
use super::length::Length;
use super::theme::value::{Theme, ThemeColor};
use canvas_gradient::CanvasShape;
pub(super) mod canvas_gradient;
pub use canvas_gradient::{CanvasGradient, GradientExtend};

/// One fill: a color, a gradient or an image, with the size, position and
/// repetition of its tiles in the area it fills (one CSS background layer).
///
/// Sizes and positions are against the area filled (a node's border box,
/// or the rect a `Draw` command fills): `pct(50)` is half of it, and
/// `pct(100) - 20` 20 px less than all of it. By default a tile covers the
/// whole area (an image: its natural size) at its top left corner and
/// repeats along both axes, as in CSS.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{pct};
/// use kivact::draw::{Paint, linear_gradient};
/// // CSS `background: linear-gradient(90deg, #2563eb, #22d3ee) 0 0 / 40% 100% no-repeat`
/// let fill: Paint = linear_gradient(90)
///     .colors([rgb(0x2563eb), rgb(0x22d3ee)])
///     .size(pct(40), pct(100))
///     .no_repeat();
/// # let _ = fill;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    pub(crate) image: PaintImage,
    pub(crate) size: PaintSize,
    pub(crate) position: (Length, Length),
    pub(crate) repeat: (bool, bool),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PaintImage {
    Color(Color),
    Gradient(Gradient),
    Canvas(CanvasGradient),
    Image(Asset),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PaintSize {
    Auto,
    Cover,
    Contain,
    Explicit(Length, Length),
}

impl Paint {
    fn of(image: PaintImage) -> Paint {
        Paint {
            image,
            size: PaintSize::Auto,
            position: (Length::Px(0.0), Length::Px(0.0)),
            repeat: (true, true),
        }
    }

    /// An image (PNG, JPEG, WebP, SVG ..) at its natural size, repeated
    /// (CSS `background-image: url(..)`). It shows nothing until it has
    /// loaded; a failure is not reported (only `image(..)` nodes report
    /// `ErrorKind::Asset`).
    pub fn image(source: impl Into<Asset>) -> Paint {
        Paint::of(PaintImage::Image(source.into()))
    }

    /// The size of one tile (CSS `background-size: w h`); `auto()` keeps the
    /// natural size (an image: its aspect ratio from the other side; a
    /// gradient: the area's).
    pub fn size(mut self, width: impl Into<Length>, height: impl Into<Length>) -> Paint {
        self.size = PaintSize::Explicit(width.into(), height.into());
        self
    }

    /// Tiles as small as covers the area, keeping the aspect ratio (CSS
    /// `background-size: cover`).
    pub fn size_cover(mut self) -> Paint {
        self.size = PaintSize::Cover;
        self
    }

    /// Tiles as large as fits in the area, keeping the aspect ratio (CSS
    /// `background-size: contain`).
    pub fn size_contain(mut self) -> Paint {
        self.size = PaintSize::Contain;
        self
    }

    /// Where the first tile lies (CSS `background-position: x y`): a
    /// percentage puts that point of the tile on that point of the area
    /// (`pct(100)` aligns the tile's right or bottom edge with the area's).
    pub fn position(mut self, x: impl Into<Length>, y: impl Into<Length>) -> Paint {
        self.position = (x.into(), y.into());
        self
    }

    /// One tile only (CSS `background-repeat: no-repeat`).
    pub fn no_repeat(mut self) -> Paint {
        self.repeat = (false, false);
        self
    }

    /// Repeat along x only (CSS `repeat-x`).
    pub fn repeat_x(mut self) -> Paint {
        self.repeat = (true, false);
        self
    }

    /// Repeat along y only (CSS `repeat-y`).
    pub fn repeat_y(mut self) -> Paint {
        self.repeat = (false, true);
        self
    }

    /// The color, when this is a color covering the area.
    pub(crate) fn solid(&self) -> Option<Color> {
        match self.image {
            PaintImage::Color(c) if self.size == PaintSize::Auto && self.repeat == (true, true) => {
                Some(c)
            }
            _ => None,
        }
    }
}

impl<C: ThemeColor> From<C> for Paint {
    fn from(c: C) -> Self {
        Paint::of(PaintImage::Color(Color::of(c)))
    }
}

impl From<Gradient> for Paint {
    fn from(g: Gradient) -> Self {
        Paint::of(PaintImage::Gradient(g))
    }
}

/// The shape of a radial gradient's end, when no radius is given (CSS
/// `closest-side` .. `farthest-corner`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadialExtent {
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    /// The default.
    FarthestCorner,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GradientKind {
    Linear {
        angle: f32,
    },
    Radial {
        circle: bool,
        size: RadialSize,
        at: (Length, Length),
    },
    Conic {
        from: f32,
        at: (Length, Length),
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RadialSize {
    Extent(RadialExtent),
    Radii(Length, Length),
}

/// A CSS gradient, made by [`linear_gradient`], [`radial_gradient`] or
/// [`conic_gradient`]. Colors may be theme tokens (resolved again when the
/// theme changes). Use it as a [`Paint`] (`style().background(gradient)`)
/// or give it a size and position first (`.size(..)`, `.position(..)`).
///
/// ```
/// use kivact::prelude::*;
/// use kivact::draw::linear_gradient;
/// // CSS `repeating-linear-gradient(90deg, #2a2a2a 0 3px, #3e3e3e 3px 4px)`
/// let ruler = linear_gradient(90)
///     .repeating()
///     .stop(rgb(0x2a2a2a), 0)
///     .stop(rgb(0x2a2a2a), 3)
///     .stop(rgb(0x3e3e3e), 3)
///     .stop(rgb(0x3e3e3e), 4);
/// let s = style().background(ruler);
/// # let _ = s;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub(crate) kind: GradientKind,
    pub(crate) stops: Vec<(Color, Option<Length>)>,
    pub(crate) repeating: bool,
}

/// A linear gradient along `angle` degrees as CSS measures it: 0 to top,
/// 90 to right, 180 to bottom (CSS `linear-gradient(<angle>deg, ..)`;
/// `to bottom` is 180).
pub fn linear_gradient(angle: impl Into<f64>) -> Gradient {
    Gradient::new(GradientKind::Linear {
        angle: angle.into() as f32,
    })
}

/// An elliptical gradient from the center of the area to its farthest
/// corner (CSS `radial-gradient(..)`); `.circle()`, `.radius(r)`,
/// `.radii(rx, ry)`, `.extent(..)` and `.at(x, y)` change its shape and
/// center.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{Color, rgba};
/// use kivact::draw::radial_gradient;
/// // CSS `radial-gradient(circle at 1px 1px, #ffffff10 1px, transparent 0) 0 0 / 18px 18px`
/// let dots = radial_gradient()
///     .circle()
///     .at(1, 1)
///     .stop(rgba(0xffffff, 0.06), 1)
///     .stop(Color::TRANSPARENT, 0)
///     .size(18, 18);
/// # let _ = dots;
/// ```
pub fn radial_gradient() -> Gradient {
    Gradient::new(GradientKind::Radial {
        circle: false,
        size: RadialSize::Extent(RadialExtent::FarthestCorner),
        at: (Length::Percent(50.0), Length::Percent(50.0)),
    })
}

/// A gradient around the center of the area starting at `from` degrees
/// (0 is up, clockwise; CSS `conic-gradient(from <deg>, ..)`). Its stop
/// positions are degrees (`.stop(c, 90)`) or percentages of the turn.
pub fn conic_gradient(from: impl Into<f64>) -> Gradient {
    Gradient::new(GradientKind::Conic {
        from: from.into() as f32,
        at: (Length::Percent(50.0), Length::Percent(50.0)),
    })
}

impl Gradient {
    fn new(kind: GradientKind) -> Gradient {
        Gradient {
            kind,
            stops: Vec::new(),
            repeating: false,
        }
    }

    /// A color stop at `at` along the gradient (px or a percentage of the
    /// gradient line; a conic gradient's are degrees). Two stops at the
    /// same position make a hard edge; a stop before the previous one is
    /// moved up to it (CSS), so `.stop(c, 0)` after `.stop(d, 10)` is at
    /// 10.
    pub fn stop(mut self, color: impl ThemeColor, at: impl Into<Length>) -> Gradient {
        self.stops.push((Color::of(color), Some(at.into())));
        self
    }

    /// Color stops without positions: spread evenly between their
    /// positioned neighbours (the first at 0%, the last at 100% when they
    /// have no position), as in CSS `linear-gradient(red, blue)`.
    pub fn colors<C: ThemeColor>(mut self, colors: impl IntoIterator<Item = C>) -> Gradient {
        self.stops
            .extend(colors.into_iter().map(|c| (Color::of(c), None)));
        self
    }

    /// Repeat the stops beyond the last one (CSS `repeating-*-gradient`):
    /// the pattern from the first stop to the last repeats.
    pub fn repeating(mut self) -> Gradient {
        self.repeating = true;
        self
    }

    /// A radial gradient: a circle instead of an ellipse.
    pub fn circle(mut self) -> Gradient {
        if let GradientKind::Radial { circle, .. } = &mut self.kind {
            *circle = true;
        }
        self
    }

    /// A radial gradient: a circle of radius `r` (CSS `radial-gradient(40px
    /// at ..)`).
    pub fn radius(mut self, r: impl Into<Length>) -> Gradient {
        if let GradientKind::Radial { circle, size, .. } = &mut self.kind {
            let r = r.into();
            *circle = true;
            *size = RadialSize::Radii(r, r);
        }
        self
    }

    /// A radial gradient: an ellipse of radii `rx` × `ry` (CSS
    /// `radial-gradient(900px 380px at ..)`; percentages of the area's
    /// width and height).
    pub fn radii(mut self, rx: impl Into<Length>, ry: impl Into<Length>) -> Gradient {
        if let GradientKind::Radial { circle, size, .. } = &mut self.kind {
            *circle = false;
            *size = RadialSize::Radii(rx.into(), ry.into());
        }
        self
    }

    /// A radial gradient: how far its end reaches (CSS `closest-side` ..).
    pub fn extent(mut self, extent: RadialExtent) -> Gradient {
        if let GradientKind::Radial { size, .. } = &mut self.kind {
            *size = RadialSize::Extent(extent);
        }
        self
    }

    /// A radial or conic gradient: its center (CSS `at x y`, against the
    /// area: `pct(50)` is the middle).
    pub fn at(mut self, x: impl Into<Length>, y: impl Into<Length>) -> Gradient {
        match &mut self.kind {
            GradientKind::Radial { at, .. } | GradientKind::Conic { at, .. } => {
                *at = (x.into(), y.into());
            }
            GradientKind::Linear { .. } => {}
        }
        self
    }

    /// As a [`Paint`] of tiles of this size ([`Paint::size`]).
    pub fn size(self, width: impl Into<Length>, height: impl Into<Length>) -> Paint {
        Paint::from(self).size(width, height)
    }

    /// As a [`Paint`] whose first tile is here ([`Paint::position`]).
    pub fn position(self, x: impl Into<Length>, y: impl Into<Length>) -> Paint {
        Paint::from(self).position(x, y)
    }

    /// As a [`Paint`] of one tile ([`Paint::no_repeat`]).
    pub fn no_repeat(self) -> Paint {
        Paint::from(self).no_repeat()
    }
}

/// A box's background: fills stacked like CSS background layers, the
/// first on top. A color, a gradient or a [`Paint`] is a background of one
/// layer; a tuple stacks several.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{Color, rgba};
/// use kivact::draw::linear_gradient;
/// // CSS `background: linear-gradient(#0002 1px, transparent 1px) 0 0 / 24px 24px,
/// //      linear-gradient(90deg, #0002 1px, transparent 1px) 0 0 / 24px 24px, white`
/// let line = rgba(0x000000, 0.12);
/// let grid = style().background((
///     linear_gradient(180).stop(line, 1).stop(Color::TRANSPARENT, 1).size(24, 24),
///     linear_gradient(90).stop(line, 1).stop(Color::TRANSPARENT, 1).size(24, 24),
///     Color::WHITE,
/// ));
/// # let _ = grid;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Background(pub(crate) Vec<Paint>);

impl Background {
    /// The color, when this is one color covering the box.
    pub(crate) fn solid(&self) -> Option<Color> {
        match self.0.as_slice() {
            [one] => one.solid(),
            _ => None,
        }
    }
}

impl From<Paint> for Background {
    fn from(p: Paint) -> Self {
        Background(vec![p])
    }
}

impl<C: ThemeColor> From<C> for Background {
    fn from(c: C) -> Self {
        Background::from(Paint::from(c))
    }
}

impl From<Gradient> for Background {
    fn from(g: Gradient) -> Self {
        Background::from(Paint::from(g))
    }
}

impl From<Vec<Paint>> for Background {
    fn from(v: Vec<Paint>) -> Self {
        Background(v)
    }
}

macro_rules! layers {
    ($($t:ident),+) => {
        impl<$($t: Into<Paint>),+> From<($($t,)+)> for Background {
            #[allow(non_snake_case, reason = "the tuple's elements are named by their types")]
            fn from(($($t,)+): ($($t,)+)) -> Self {
                Background(vec![$($t.into()),+])
            }
        }
    };
}

layers!(A, B);
layers!(A, B, C);
layers!(A, B, C, D);
layers!(A, B, C, D, E);
layers!(A, B, C, D, E, F);

impl From<CanvasGradient> for Paint {
    fn from(g: CanvasGradient) -> Self {
        Paint::of(PaintImage::Canvas(g))
    }
}

/// A paint as the engine fills an area with it.
pub(crate) fn fill_layer(l: &Paint, theme: &Theme) -> FillLayer {
    let len = |l: Length| l.to_len();
    let auto = |l: Length| (l != Length::Auto && !l.is_content()).then(|| l.to_len());
    FillLayer {
        image: match &l.image {
            PaintImage::Color(c) => FillImage::Solid(c.resolve(theme)),
            PaintImage::Gradient(g) => FillImage::Gradient(gradient(g, theme)),
            PaintImage::Canvas(g) => FillImage::Coordinate(canvas_gradient(g, theme)),
            PaintImage::Image(a) => FillImage::Image(a.clone()),
        },
        size: match l.size {
            PaintSize::Auto => FillSize::Auto,
            PaintSize::Cover => FillSize::Cover,
            PaintSize::Contain => FillSize::Contain,
            PaintSize::Explicit(w, h) => FillSize::Explicit(auto(w), auto(h)),
        },
        position: [len(l.position.0), len(l.position.1)],
        repeat_x: l.repeat.0,
        repeat_y: l.repeat.1,
    }
}

/// A gradient by coordinates for the engine.
fn canvas_gradient(g: &CanvasGradient, theme: &Theme) -> p::CoordinateGradient {
    p::CoordinateGradient {
        shape: match g.shape {
            CanvasShape::Linear { from, to } => p::CoordinateShape::Linear {
                start: from,
                end: to,
            },
            CanvasShape::Radial { c0, r0, c1, r1 } => p::CoordinateShape::RadialTwoCircle {
                start_center: c0,
                start_radius: r0,
                end_center: c1,
                end_radius: r1,
            },
            CanvasShape::Conic { start_deg, center } => {
                p::CoordinateShape::Conic { center, start_deg }
            }
        },
        stops: g
            .stops
            .iter()
            .map(|(o, c)| (*o, c.resolve(theme)))
            .collect(),
        extend: g.extend,
    }
}

fn gradient(g: &Gradient, theme: &Theme) -> p::Gradient {
    let at = |(x, y): (Length, Length)| [x.to_len(), y.to_len()];
    p::Gradient {
        kind: match g.kind {
            GradientKind::Linear { angle } => WireKind::Linear { angle_deg: angle },
            GradientKind::Radial {
                circle,
                size,
                at: c,
            } => WireKind::Radial {
                circle,
                extent: match size {
                    RadialSize::Extent(e) => match e {
                        RadialExtent::ClosestSide => p::RadialExtent::ClosestSide,
                        RadialExtent::ClosestCorner => p::RadialExtent::ClosestCorner,
                        RadialExtent::FarthestSide => p::RadialExtent::FarthestSide,
                        RadialExtent::FarthestCorner => p::RadialExtent::FarthestCorner,
                    },
                    RadialSize::Radii(x, y) => p::RadialExtent::Radii(x.to_len(), y.to_len()),
                },
                center: at(c),
            },
            GradientKind::Conic { from, at: c } => WireKind::Conic {
                from_deg: from,
                center: at(c),
            },
        },
        stops: g
            .stops
            .iter()
            .map(|(c, at)| GradientStop {
                color: c.resolve(theme),
                at: at.map(Length::to_len),
            })
            .collect(),
        repeating: g.repeating,
    }
}
