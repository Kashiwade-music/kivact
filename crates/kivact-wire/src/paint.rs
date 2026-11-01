//! Role: Paint and fill protocol types.
//!
//! Resolved element paint beyond plain colors: fill layers (gradients,
//! images, tiles), borders, corner radii, filters, blending, masks and clip
//! shapes. Lengths that depend on the box are [`Len`]s the engine resolves
//! at paint time, so the core never needs the layout.

use super::Point;
use super::values::{Edges, Rgba};
use kivact_types::Asset;

/// A length resolved against a reference length at paint time:
/// `px + pct / 100 × reference` (CSS `calc(pct% + px)`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Len {
    pub px: f32,
    pub pct: f32,
}

impl Len {
    pub const ZERO: Len = Len { px: 0.0, pct: 0.0 };

    pub const fn px(px: f32) -> Len {
        Len { px, pct: 0.0 }
    }

    pub const fn pct(pct: f32) -> Len {
        Len { px: 0.0, pct }
    }

    /// The length against `reference`.
    pub fn of(self, reference: f32) -> f32 {
        self.px + self.pct / 100.0 * reference
    }
}

/// What a box (or a scene rect) is filled with.
#[derive(Clone, Debug, PartialEq)]
pub enum Fill {
    Solid(Rgba),
    /// CSS background layers, the first on top.
    Layers(Vec<FillLayer>),
}

/// One CSS background layer: an image (a color, a gradient or a picture)
/// sized, positioned and repeated in the box it fills.
#[derive(Clone, Debug, PartialEq)]
pub struct FillLayer {
    pub image: FillImage,
    pub size: FillSize,
    /// Where the first tile is: x against `box − tile` width, y against
    /// `box − tile` height (CSS `background-position`).
    pub position: [Len; 2],
    pub repeat_x: bool,
    pub repeat_y: bool,
}

impl FillLayer {
    /// A layer covering the box with `image`.
    pub fn cover(image: FillImage) -> FillLayer {
        FillLayer {
            image,
            size: FillSize::Auto,
            position: [Len::ZERO; 2],
            repeat_x: true,
            repeat_y: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FillImage {
    Solid(Rgba),
    Gradient(Gradient),
    Image(Asset),
    /// A gradient placed by coordinates, not by the area it fills (the
    /// layer's size, position and repeat do not apply to it).
    Coordinate(CoordinateGradient),
}

/// A gradient placed by coordinates, as canvas `createLinearGradient`,
/// `createRadialGradient` and `createConicGradient` do. The points are in
/// the local coordinates of what it is drawn in: the scene of a node (its
/// canvas origin) or, as a background or mask, the box (its border box's
/// top left corner).
#[derive(Clone, Debug, PartialEq)]
pub struct CoordinateGradient {
    pub shape: CoordinateShape,
    /// Offsets and colors, in order of offset (the engine draws nothing for
    /// offsets outside `0..=1`, not finite or out of order, or for no stop);
    /// two stops at one offset make a hard edge.
    pub stops: Vec<(f32, Rgba)>,
    pub extend: GradientExtend,
}

/// Where a [`CoordinateGradient`]'s offsets lie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoordinateShape {
    /// Offset 0 at `start`, offset 1 at `end`.
    Linear { start: Point, end: Point },
    /// Offset 0 on the circle of `start_radius` around `start_center`,
    /// offset 1 on the circle of `end_radius` around `end_center`.
    RadialTwoCircle {
        start_center: Point,
        start_radius: f32,
        end_center: Point,
        end_radius: f32,
    },
    /// Offsets by angle around `center`, one turn clockwise from
    /// `start_deg` (0 along the positive x axis).
    Conic { center: Point, start_deg: f32 },
}

/// What a [`CoordinateGradient`] paints beyond its offsets 0 and 1.
pub use kivact_types::Extend as GradientExtend;

/// The size of one tile (CSS `background-size`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum FillSize {
    /// A gradient or color: the box; an image: its natural size.
    #[default]
    Auto,
    Cover,
    Contain,
    /// Width against the box width, height against the box height; `None`
    /// is auto (the box for a gradient, the image's aspect for an image).
    Explicit(Option<Len>, Option<Len>),
}

/// A CSS gradient image.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    pub kind: GradientKind,
    /// Stops in order; a stop without a position is placed as CSS places
    /// it (evenly between its positioned neighbours).
    pub stops: Vec<GradientStop>,
    /// CSS `repeating-*-gradient`.
    pub repeating: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub color: Rgba,
    /// Against the gradient line's length (linear, radial); a conic
    /// gradient's `px` are degrees and `pct` a percentage of the turn.
    pub at: Option<Len>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientKind {
    /// CSS `linear-gradient(<angle>deg, ..)`: 0 = to top, 90 = to right.
    Linear { angle_deg: f32 },
    /// CSS `radial-gradient(<shape> <size> at <center>, ..)`; the center's
    /// x is against the box width, y against its height.
    Radial {
        circle: bool,
        extent: RadialExtent,
        center: [Len; 2],
    },
    /// CSS `conic-gradient(from <deg> at <center>, ..)`.
    Conic { from_deg: f32, center: [Len; 2] },
}

/// The size of a radial gradient's ending shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RadialExtent {
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    FarthestCorner,
    /// Explicit radii, x against the box width and y against its height (a
    /// circle uses x).
    Radii(Len, Len),
}

/// How a border's line is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BorderStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

/// A border inside the border box, per side.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Border {
    pub widths: Edges<f32>,
    pub colors: Edges<Rgba>,
    pub style: BorderStyle,
}

impl Border {
    /// The same width and color on every side, solid.
    pub fn uniform(width: f32, color: Rgba) -> Border {
        Border {
            widths: Edges::all(width),
            colors: Edges::all(color),
            style: BorderStyle::Solid,
        }
    }
}

/// The focus ring: a line of `width` outside the border box.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Ring {
    pub width: f32,
    pub color: Rgba,
}

/// An elliptical corner: `x` against the box width, `y` against its
/// height.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CornerRadius {
    pub x: Len,
    pub y: Len,
}

impl CornerRadius {
    /// A circular radius measured in logical pixels.
    pub const fn px(r: f32) -> CornerRadius {
        CornerRadius {
            x: Len::px(r),
            y: Len::px(r),
        }
    }
}

/// Elliptical radii for each corner of a border box.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Corners {
    pub top_left: CornerRadius,
    pub top_right: CornerRadius,
    pub bottom_right: CornerRadius,
    pub bottom_left: CornerRadius,
}

impl Corners {
    /// The same circular radius (px) on every corner.
    pub fn all(r: f32) -> Self {
        let c = CornerRadius::px(r);
        Corners {
            top_left: c,
            top_right: c,
            bottom_right: c,
            bottom_left: c,
        }
    }

    /// Clockwise from the top left.
    pub fn each(&self) -> [CornerRadius; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }

    /// No corner is rounded.
    pub fn is_zero(&self) -> bool {
        self.each()
            .iter()
            .all(|c| c.x == Len::ZERO || c.y == Len::ZERO)
    }
}

/// One CSS filter function.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterOp {
    Brightness(f32),
    Contrast(f32),
    Saturate(f32),
    Grayscale(f32),
    Sepia(f32),
    /// Degrees.
    HueRotate(f32),
    Invert(f32),
    Opacity(f32),
    /// CSS `blur(px)`.
    Blur(f32),
    /// CSS `drop-shadow(x y blur color)`.
    DropShadow {
        offset: Point,
        blur: f32,
        color: Rgba,
    },
}

/// CSS `mix-blend-mode`.
pub use kivact_types::BlendMode;

/// A CSS basic shape in the border box (`clip-path`): x against the box
/// width, y against its height.
#[derive(Clone, Debug, PartialEq)]
pub enum ClipShape {
    Path {
        cmds: Vec<super::PathCmd>,
        even_odd: bool,
    },
    /// CSS `inset(top right bottom left round radii)`.
    Inset { edges: Edges<Len>, radius: Corners },
    /// CSS `circle(r at x y)`; a percentage radius is against
    /// `√(w² + h²) / √2`.
    Circle { radius: Len, center: [Len; 2] },
    /// CSS `ellipse(rx ry at x y)`.
    Ellipse {
        radius_x: Len,
        radius_y: Len,
        center: [Len; 2],
    },
    /// CSS `polygon(x y, ..)`.
    Polygon {
        points: Vec<[Len; 2]>,
        even_odd: bool,
    },
}

/// The paint effects of a node, rare enough to be boxed: none of them costs
/// anything to a node that does not use them.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Effects {
    /// CSS `filter`, in order.
    pub filter: Vec<FilterOp>,
    /// CSS `backdrop-filter`, in order (empty: none).
    pub backdrop: Vec<FilterOp>,
    pub blend: BlendMode,
    /// CSS `mask` (its alpha masks the node and its subtree).
    pub mask: Option<FillLayer>,
    pub clip_path: Option<ClipShape>,
}

impl Effects {
    /// Nothing to apply.
    pub fn is_empty(&self) -> bool {
        self.filter.is_empty()
            && self.backdrop.is_empty()
            && self.blend == BlendMode::Normal
            && self.mask.is_none()
            && self.clip_path.is_none()
    }
}
