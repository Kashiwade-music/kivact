//! Role: Shared node paint, image payloads, effects and state-dependent styling.
mod prop_interp;

pub use prop_interp::{PropBasis, PropInterp, apply_components, prop_interp};

use crate::color_space::SpaceColor;
use crate::effect::FilterGraph;
use crate::geometry3d::Matrix4;
use crate::layout::IntrinsicSize;
use crate::motion::{Transition, TransitionProp};
use crate::pixel::BlendMode;
use crate::{Asset, BorderStyle, Edges, Point, Rgba, Size};

/// Image content, fitting policy and optional intrinsic sizing.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ImageData {
    /// What the node shows (`None`: nothing); compared by identity.
    pub image: Option<Asset>,
    /// How it fills the box; `None` is `ImageFit::Contain`.
    pub fit: Option<ImageFit>,
    /// `true`: only the image's alpha is used, painted in the node's
    /// effective text color (inherited, with its state styles).
    pub monochrome: Option<bool>,
    /// Optional natural dimensions used to size the image.
    pub intrinsic: Option<IntrinsicSize>,
    /// Animated images: `Some(true)` plays, `Some(false)` holds the frame.
    pub playing: Option<bool>,
}

/// How a gradient continues beyond its first and last stop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Extend {
    /// The end colors continue.
    #[default]
    Pad,
    /// The stops repeat (CSS `repeating-*-gradient`).
    Repeat,
    /// The stops repeat, every other time backwards.
    Reflect,
}

/// A length resolved against a reference length at paint time:
/// `px + pct / 100 × reference` (CSS `calc(pct% + px)`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Len {
    /// The absolute part in logical px.
    pub px: f64,
    /// The percentage part.
    pub pct: f64,
}

impl Len {
    /// Zero length.
    pub const ZERO: Len = Len { px: 0.0, pct: 0.0 };

    /// A length of `px` logical pixels.
    pub const fn px(px: f64) -> Len {
        Len { px, pct: 0.0 }
    }

    /// A length of `pct` percent of the reference.
    pub const fn pct(pct: f64) -> Len {
        Len { px: 0.0, pct }
    }

    /// The length against `reference`.
    pub fn of(self, reference: f64) -> f64 {
        self.px + self.pct / 100.0 * reference
    }
}

/// What a box (or a scene rect) is filled with.
#[derive(Clone, Debug, PartialEq)]
pub enum Fill {
    /// One solid color.
    Solid(Rgba),
    /// CSS background layers, the first on top.
    Layers(Vec<FillLayer>),
}

/// The clip of a background or mask layer (CSS `background-clip`, `mask-clip`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BackgroundClip {
    /// Clipped to the border box.
    #[default]
    BorderBox,
    /// Clipped to the padding box.
    PaddingBox,
    /// Clipped to the content box.
    ContentBox,
    /// Painted only through the text glyphs.
    Text,
}

/// The origin a background or mask layer is positioned from (CSS `background-origin`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BackgroundOrigin {
    /// The border box.
    BorderBox,
    /// The padding box (the default for backgrounds).
    #[default]
    PaddingBox,
    /// The content box.
    ContentBox,
}

/// One CSS background layer: an image (a color, a gradient or a picture)
/// sized, positioned and repeated in the box it fills.
#[derive(Clone, Debug, PartialEq)]
pub struct FillLayer {
    /// What the layer paints.
    pub image: FillImage,
    /// The size of one tile.
    pub size: FillSize,
    /// Where the first tile is: x against `box − tile` width, y against
    /// `box − tile` height (CSS `background-position`).
    pub position: [Len; 2],
    /// Repeat horizontally.
    pub repeat_x: bool,
    /// Repeat vertically.
    pub repeat_y: bool,
    /// CSS background-clip / mask-clip.
    pub clip: BackgroundClip,
    /// CSS background-origin / mask-origin; None: the context's default
    /// (backgrounds: PaddingBox, masks: BorderBox).
    pub origin: Option<BackgroundOrigin>,
    /// CSS background-blend-mode (masks ignore it).
    pub blend: BlendMode,
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
            clip: BackgroundClip::BorderBox,
            origin: None,
            blend: BlendMode::Normal,
        }
    }
}

/// What one background or mask layer paints.
#[derive(Clone, Debug, PartialEq)]
pub enum FillImage {
    /// A solid color.
    Solid(Rgba),
    /// A CSS gradient image.
    Gradient(Gradient),
    /// A picture.
    Image(Asset),
    /// A gradient placed by coordinates, not by the area it fills (the
    /// layer's size, position and repeat do not apply to it).
    Coordinate(CoordinateGradient),
    /// A paint shader (§11.3).
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; the design keeps one allocation.
    Shader(std::sync::Arc<ShaderSpec>),
}

/// A gradient placed by coordinates, as canvas `createLinearGradient`,
/// `createRadialGradient` and `createConicGradient` do. The points are in
/// the local coordinates of what it is drawn in: the scene of a node (its
/// canvas origin) or, as a background or mask, the box (its border box's
/// top left corner).
#[derive(Clone, Debug, PartialEq)]
pub struct CoordinateGradient {
    /// The geometry of the gradient.
    pub shape: CoordinateShape,
    /// Offsets and authored colors, in order of offset (the engine draws nothing for
    /// offsets outside `0..=1`, not finite or out of order, or for no stop);
    /// two stops at one offset make a hard edge.
    pub stops: Vec<(f64, SpaceColor)>,
    /// What happens beyond the offsets.
    pub extend: Extend,
    /// The color space the stops are interpolated in (canvas: sRGB).
    pub interpolation: crate::color_space::ColorInterpolation,
}

/// Where a [`CoordinateGradient`]'s offsets lie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoordinateShape {
    /// Offset 0 at `start`, offset 1 at `end`.
    Linear {
        /// Where offset 0 lies.
        start: Point,
        /// Where offset 1 lies.
        end: Point,
    },
    /// Offset 0 on the circle of `start_radius` around `start_center`,
    /// offset 1 on the circle of `end_radius` around `end_center`.
    RadialTwoCircle {
        /// Center of the offset-0 circle.
        start_center: Point,
        /// Radius of the offset-0 circle.
        start_radius: f64,
        /// Center of the offset-1 circle.
        end_center: Point,
        /// Radius of the offset-1 circle.
        end_radius: f64,
    },
    /// Offsets by angle around `center`, one turn clockwise from
    /// `start_deg` (0 along the positive x axis).
    Conic {
        /// The center of the turn.
        center: Point,
        /// Where the turn starts, in degrees.
        start_deg: f64,
    },
}

/// The size of one tile (CSS `background-size`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum FillSize {
    /// A gradient or color: the box; an image: its natural size.
    #[default]
    Auto,
    /// Covers the box, keeping the aspect ratio.
    Cover,
    /// Fits inside the box, keeping the aspect ratio.
    Contain,
    /// Width against the box width, height against the box height; `None`
    /// is auto (the box for a gradient, the image's aspect for an image).
    Explicit(Option<Len>, Option<Len>),
}

/// A CSS gradient image.
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    /// Linear, radial or conic.
    pub kind: GradientKind,
    /// Stops in order; a stop without a position is placed as CSS places
    /// it (evenly between its positioned neighbours).
    pub stops: Vec<GradientStop>,
    /// CSS color hint between stop i and stop i + 1; len = stops.len() − 1.
    pub hints: Vec<Option<Len>>,
    /// CSS `repeating-*-gradient`.
    pub repeating: bool,
    /// The color space of the interpolation (CSS default: Oklab).
    pub interpolation: crate::color_space::ColorInterpolation,
}

/// One color stop of a [`Gradient`], with the authored color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    /// The color, in the space it was authored in.
    pub color: SpaceColor,
    /// Against the gradient line's length (linear, radial); a conic
    /// gradient's `px` are degrees and `pct` a percentage of the turn.
    pub at: Option<Len>,
}

/// The shape of a CSS gradient.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientKind {
    /// CSS `linear-gradient(<angle>deg, ..)`: 0 = to top, 90 = to right.
    Linear {
        /// The angle in degrees.
        angle_deg: f64,
    },
    /// CSS `radial-gradient(<shape> <size> at <center>, ..)`; the center's
    /// x is against the box width, y against its height.
    Radial {
        /// A circle (true) or an ellipse.
        circle: bool,
        /// The size of the ending shape.
        extent: RadialExtent,
        /// The center.
        center: [Len; 2],
    },
    /// CSS `conic-gradient(from <deg> at <center>, ..)`.
    Conic {
        /// Where the turn starts, in degrees.
        from_deg: f64,
        /// The center.
        center: [Len; 2],
    },
}

/// The size of a radial gradient's ending shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RadialExtent {
    /// Closest side.
    ClosestSide,
    /// Closest corner.
    ClosestCorner,
    /// Farthest side.
    FarthestSide,
    /// Farthest corner.
    FarthestCorner,
    /// Explicit radii, x against the box width and y against its height (a
    /// circle uses x).
    Radii(Len, Len),
}

/// A border inside the border box, per side.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Border {
    /// Widths per side.
    pub widths: Edges<f64>,
    /// Colors per side.
    pub colors: Edges<Rgba>,
    /// Line styles per side.
    pub styles: Edges<BorderStyle>,
}

impl Border {
    /// The same width and color on every side, solid.
    pub fn uniform(width: f64, color: Rgba) -> Border {
        Border {
            widths: Edges::all(width),
            colors: Edges::all(color),
            styles: Edges::all(BorderStyle::Solid),
        }
    }
}

/// Mouse cursor appearance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Cursor {
    #[default]
    /// Auto cursor.
    Auto,
    /// Default cursor.
    Default,
    /// Text cursor.
    Text,
    /// Pointer cursor.
    Pointer,
    /// ColResize cursor.
    ColResize,
    /// RowResize cursor.
    RowResize,
    /// EwResize cursor.
    EwResize,
    /// NsResize cursor.
    NsResize,
    /// Grab cursor.
    Grab,
    /// Grabbing cursor.
    Grabbing,
    /// Crosshair cursor.
    Crosshair,
    /// Move cursor.
    Move,
    /// NotAllowed cursor.
    NotAllowed,
    /// None cursor.
    None,
}

/// How an image fills its box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImageFit {
    /// Scale to fit inside the box, keeping the aspect ratio.
    #[default]
    Contain,
    /// Scale to cover the box, keeping the aspect ratio.
    Cover,
    /// Stretch to the box.
    Fill,
    /// Natural size.
    None,
    /// `None` when the natural size fits the box, else `Contain`.
    ScaleDown,
}

/// A focus ring of `width` separated from the border box by `offset`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Ring {
    /// Width of the ring in logical px.
    pub width: f64,
    /// Gap between the border box and the ring.
    pub offset: f64,
    /// Color of the ring.
    pub color: Rgba,
}

/// An elliptical corner: `x` against the box width, `y` against its
/// height.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CornerRadius {
    /// The horizontal radius.
    pub x: Len,
    /// The vertical radius.
    pub y: Len,
}

impl CornerRadius {
    /// A circular radius measured in logical pixels.
    pub const fn px(r: f64) -> CornerRadius {
        CornerRadius {
            x: Len::px(r),
            y: Len::px(r),
        }
    }
}

/// Elliptical radii for each corner of a border box.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Corners {
    /// Top left corner.
    pub top_left: CornerRadius,
    /// Top right corner.
    pub top_right: CornerRadius,
    /// Bottom right corner.
    pub bottom_right: CornerRadius,
    /// Bottom left corner.
    pub bottom_left: CornerRadius,
}

impl Corners {
    /// The same circular radius (px) on every corner.
    pub fn all(r: f64) -> Self {
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
#[derive(Clone, Debug, PartialEq)]
pub enum FilterOp {
    /// `brightness()`.
    Brightness(f64),
    /// `contrast()`.
    Contrast(f64),
    /// `saturate()`.
    Saturate(f64),
    /// `grayscale()`.
    Grayscale(f64),
    /// `sepia()`.
    Sepia(f64),
    /// Degrees.
    HueRotate(f64),
    /// `invert()`.
    Invert(f64),
    /// `opacity()`.
    Opacity(f64),
    /// CSS `blur(px)`.
    Blur(f64),
    /// CSS `drop-shadow(x y blur color)`.
    DropShadow {
        /// Offset of the shadow.
        offset: Point,
        /// Blur radius.
        blur: f64,
        /// Color of the shadow.
        color: Rgba,
    },
    /// CSS `url(#filter)`: an SVG filter graph (§5.9), images as assets.
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; the design keeps one allocation.
    Graph(std::sync::Arc<FilterGraph<Asset>>),
    /// A filter shader (§11.3).
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; the design keeps one allocation.
    Shader(std::sync::Arc<ShaderSpec>),
}

/// A CSS basic shape in the border box (`clip-path`): x against the box
/// width, y against its height.
#[derive(Clone, Debug, PartialEq)]
pub enum ClipShape {
    /// An explicit path.
    Path {
        /// The path commands.
        cmds: Vec<crate::scene::PathCmd>,
        /// Even-odd fill rule (false: non-zero).
        even_odd: bool,
    },
    /// CSS `inset(top right bottom left round radii)`.
    Inset {
        /// The insets from each edge.
        edges: Edges<Len>,
        /// The corner radii.
        radius: Corners,
    },
    /// CSS `circle(r at x y)`; a percentage radius is against
    /// `√(w² + h²) / √2`.
    Circle {
        /// The radius.
        radius: Len,
        /// The center.
        center: [Len; 2],
    },
    /// CSS `ellipse(rx ry at x y)`.
    Ellipse {
        /// Horizontal radius.
        radius_x: Len,
        /// Vertical radius.
        radius_y: Len,
        /// The center.
        center: [Len; 2],
    },
    /// CSS `polygon(x y, ..)`.
    Polygon {
        /// The vertices.
        points: Vec<[Len; 2]>,
        /// Even-odd fill rule (false: non-zero).
        even_odd: bool,
    },
}

/// The mode of one CSS mask layer (`mask-mode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MaskMode {
    /// The alpha of the mask masks.
    #[default]
    Alpha,
    /// The luminance times the alpha of the mask masks.
    Luminance,
}

/// How a mask layer combines with the layers below it (`mask-composite`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MaskComposite {
    /// Add.
    #[default]
    Add,
    /// Subtract.
    Subtract,
    /// Intersect.
    Intersect,
    /// Exclude.
    Exclude,
}

/// One CSS mask layer.
#[derive(Clone, Debug, PartialEq)]
pub struct MaskLayer {
    /// The image of the mask.
    pub fill: FillLayer,
    /// Alpha or luminance.
    pub mode: MaskMode,
    /// How it combines with the layers below.
    pub composite: MaskComposite,
}

/// How a border image tiles its edges (`border-image-repeat`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BorderImageRepeat {
    /// Stretch the slices to fill.
    #[default]
    Stretch,
    /// Repeat the slices.
    Repeat,
    /// Repeat with a whole number of slices.
    Round,
    /// Repeat with spacing.
    Space,
}

/// CSS border-image (also the nine-slice of `Draw::border_image`).
#[derive(Clone, Debug, PartialEq)]
pub struct BorderImage {
    /// Image, gradient or solid color (§9.2).
    pub source: FillImage,
    /// Insets into the source: px are source pixels (for gradients and
    /// solid colors, px of the border image area), pct of the source size.
    pub slice: Edges<Len>,
    /// Paint the middle of the slices.
    pub fill: bool,
    /// None: the border widths. pct is of the border image area (§9.2).
    pub width: Option<Edges<Len>>,
    /// How far the border image extends beyond the border box.
    pub outset: Edges<f64>,
    /// Horizontal, vertical.
    pub repeat: [BorderImageRepeat; 2],
}

/// The paint effects of a node, rare enough to be boxed: none of them costs
/// anything to a node that does not use them.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Effects {
    /// CSS `filter`, in order.
    pub filter: Vec<FilterOp>,
    /// CSS `backdrop-filter`, in order (empty: none).
    pub backdrop: Vec<FilterOp>,
    /// Blend mode with what is beneath.
    pub blend: BlendMode,
    /// CSS `mask` layers (their alpha masks the node and its subtree).
    pub masks: Vec<MaskLayer>,
    /// CSS `clip-path`.
    pub clip_path: Option<ClipShape>,
}

impl Effects {
    /// Nothing to apply.
    pub fn is_empty(&self) -> bool {
        self.filter.is_empty()
            && self.backdrop.is_empty()
            && self.blend == BlendMode::Normal
            && self.masks.is_empty()
            && self.clip_path.is_none()
    }
}

/// How a shader's `time` advances (§11.3.5).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ShaderClock {
    /// Time does not advance.
    #[default]
    Still,
    /// Time advances at this rate in hertz.
    Hz(f64),
    /// Time advances with the display.
    Display,
}

/// A shader as authored on a node (paint wire): the engine turns it into a
/// `ShaderUse` for each frame (time, decoded images).
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderSpec {
    /// The validated program.
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; the design keeps one allocation.
    pub program: std::sync::Arc<crate::effect::ShaderProgram>,
    /// The parameter bytes, laid out by the program's parameter slots.
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; the design keeps one allocation.
    pub params: std::sync::Arc<[u8]>,
    /// Slots 0..=3 of the WGSL `image(slot, uv)` helper.
    pub images: [Option<Asset>; 4],
    /// How the shader's time advances.
    pub clock: ShaderClock,
    /// Keeps the clock running under reduced motion.
    pub essential: bool,
    /// Paint shaders: what the CPU (and a failed pipeline) draws instead; None: nothing.
    /// Never `FillImage::Shader` (resolve reports and drops it).
    pub fallback: Option<FillImage>,
    /// Filter shaders: px the output may extend beyond the layer bounds.
    pub margin: f64,
}

/// Element transform: CSS individual transforms and `transform`, in one
/// canonical form. About its origin the box is scaled, skewed, rotated
/// (x, then y, then z), moved, then seen through `perspective()`:
/// T(o) · P · T(t) · Rz · Ry · Rx · K · S · T(−o).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementTransform {
    /// The move in px (x, y, z) ...
    pub translate: [f64; 3],
    /// ... plus fractions of the box size, as before:
    /// x += per_size[0]·w + per_size[1]·h, y += per_size[2]·w + per_size[3]·h.
    pub per_size: [f64; 4],
    /// Degrees about x, y and z.
    pub rotate: [f64; 3],
    /// Degrees: xy (skewX), xz, yz.
    pub skew: [f64; 3],
    /// Scale factors along x, y and z.
    pub scale: [f64; 3],
    /// 1/d of the transform's `perspective(d)` function; 0 is none.
    pub perspective_inverse: f64,
    /// Against the box's width and height (CSS default 50% 50%).
    pub origin: [Len; 2],
    /// z of the origin in px (CSS transform-origin's third value).
    pub origin_z: f64,
}

impl ElementTransform {
    /// No transform (origin at the centre).
    pub const NONE: ElementTransform = ElementTransform {
        translate: [0.0; 3],
        per_size: [0.0; 4],
        rotate: [0.0; 3],
        skew: [0.0; 3],
        scale: [1.0; 3],
        perspective_inverse: 0.0,
        origin: [Len::pct(50.0), Len::pct(50.0)],
        origin_z: 0.0,
    };

    /// Whether it moves nothing (whatever its origin).
    pub fn is_none(&self) -> bool {
        self.translate == [0.0; 3]
            && self.per_size == [0.0; 4]
            && self.rotate == [0.0; 3]
            && self.skew == [0.0; 3]
            && self.scale == [1.0; 3]
            && self.perspective_inverse == 0.0
    }

    /// The 4×4 matrix of this transform for a box of `size`.
    pub fn matrix(&self, size: Size) -> Matrix4 {
        let o = [
            self.origin[0].of(size.width),
            self.origin[1].of(size.height),
            self.origin_z,
        ];
        let t = [
            self.translate[0] + self.per_size[0] * size.width + self.per_size[1] * size.height,
            self.translate[1] + self.per_size[2] * size.width + self.per_size[3] * size.height,
            self.translate[2],
        ];
        let p = if self.perspective_inverse > 0.0 {
            Matrix4::perspective(1.0 / self.perspective_inverse)
        } else {
            Matrix4::IDENTITY
        };
        Matrix4::translate(-o[0], -o[1], -o[2])
            .then(Matrix4::scale(self.scale[0], self.scale[1], self.scale[2]))
            .then(Matrix4::skew(self.skew[0], self.skew[1], self.skew[2]))
            .then(Matrix4::rotate_x(self.rotate[0]))
            .then(Matrix4::rotate_y(self.rotate[1]))
            .then(Matrix4::rotate_z(self.rotate[2]))
            .then(Matrix4::translate(t[0], t[1], t[2]))
            .then(p)
            .then(Matrix4::translate(o[0], o[1], o[2]))
    }

    /// Whether the transform is two-dimensional (no z component, no perspective).
    pub fn is_2d(&self) -> bool {
        self.translate[2] == 0.0
            && self.rotate[0] == 0.0
            && self.rotate[1] == 0.0
            && self.skew[1] == 0.0
            && self.skew[2] == 0.0
            && self.scale[2] == 1.0
            && self.perspective_inverse == 0.0
            && self.origin_z == 0.0
    }
}

impl Default for ElementTransform {
    fn default() -> Self {
        ElementTransform::NONE
    }
}

/// CSS `perspective` and `perspective-origin`: how a node's children are seen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Perspective {
    /// Distance from the viewer to the plane z = 0, in px.
    pub distance: f64,
    /// Where the viewer looks at, against the box (default 50% 50%).
    pub origin: [Len; 2],
}

/// CSS `transform-style`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TransformStyle {
    /// Children are flattened into the node.
    #[default]
    Flat,
    /// Children share a 3D context with the node.
    Preserve3d,
}

/// CSS `backface-visibility`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BackfaceVisibility {
    /// The back face is shown.
    #[default]
    Visible,
    /// The back face is hidden.
    Hidden,
}

/// CSS `will-change`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WillChange {
    /// No hint.
    #[default]
    Auto,
    /// The transform will change.
    Transform,
    /// The opacity will change.
    Opacity,
}

/// CSS `isolation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Isolation {
    /// Isolated only where the other rules say.
    #[default]
    Auto,
    /// Always an isolated group.
    Isolate,
}

/// Shadows animated (more change at once).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Shadow {
    /// Offset of the shadow.
    pub offset: Point,
    /// Blur radius.
    pub blur: f64,
    /// Growth of the shadow's shape.
    pub spread: f64,
    /// Color.
    pub color: Rgba,
    /// Inside the box (CSS `inset`).
    pub inset: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintProps {
    /// Per-property paint transitions, absent on ordinary nodes.
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; the design keeps one allocation.
    pub transitions: Option<std::sync::Arc<[(TransitionProp, Transition)]>>,
    /// The background fill.
    pub background: Option<Fill>,
    /// The border.
    pub border: Option<Border>,
    /// Focus ring drawn outside the border box.
    pub ring: Option<Ring>,
    /// Corner radii.
    pub radius: Corners,
    /// CSS order: the first is on top.
    pub shadows: Vec<Shadow>,
    /// Group opacity of the node and its subtree.
    pub opacity: f64,
    /// The element transform.
    pub transform: ElementTransform,
    /// CSS `perspective` of this node's children.
    pub perspective: Option<Perspective>,
    /// CSS `transform-style`.
    pub transform_style: TransformStyle,
    /// CSS `backface-visibility`.
    pub backface: BackfaceVisibility,
    /// CSS `will-change`.
    pub will_change: WillChange,
    /// CSS `isolation`.
    pub isolation: Isolation,
    /// CSS `border-image`.
    pub border_image: Option<Box<BorderImage>>,
    /// The paint and hit order among its siblings (CSS `z-index`; ties
    /// keep the tree order).
    pub z_index: i32,
    /// The mouse cursor.
    pub cursor: Cursor,
    /// Overlay scrollbar appearance.
    pub scrollbar: Option<Box<ScrollbarProps>>,
    /// Clip children to the padding box (rounded by the inner radii).
    pub clip: bool,
    /// Filters, backdrop, blending, masks and clip path (`None`: none).
    pub effects: Option<Box<Effects>>,
}

impl Default for PaintProps {
    fn default() -> Self {
        PaintProps {
            transitions: None,
            background: None,
            border: None,
            ring: None,
            radius: Corners::default(),
            shadows: Vec::new(),
            opacity: 1.0,
            transform: ElementTransform::NONE,
            perspective: None,
            transform_style: TransformStyle::Flat,
            backface: BackfaceVisibility::Visible,
            will_change: WillChange::Auto,
            isolation: Isolation::Auto,
            border_image: None,
            z_index: 0,
            cursor: Cursor::Auto,
            scrollbar: None,
            clip: false,
            effects: None,
        }
    }
}

/// A partial paint override applied by the host while a pointer/focus state
/// is active. Layout never changes with these states.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PaintDelta {
    /// Replaces the background.
    pub background: Option<Fill>,
    /// Replaces the border.
    pub border: Option<Border>,
    /// Replaces the focus ring.
    pub ring: Option<Ring>,
    /// Replaces the corner radii.
    pub radius: Option<Corners>,
    /// Replaces the opacity.
    pub opacity: Option<f64>,
    /// Replaces the shadows.
    pub shadows: Option<Vec<Shadow>>,
    /// Replaces the element transform.
    pub transform: Option<ElementTransform>,
    /// Replaces the perspective (the complete value, or none).
    pub perspective: Option<Option<Perspective>>,
    /// Replaces the transform style.
    pub transform_style: Option<TransformStyle>,
    /// Replaces the backface visibility.
    pub backface: Option<BackfaceVisibility>,
    /// Replaces the z-index.
    pub z_index: Option<i32>,
    /// Replaces the cursor.
    pub cursor: Option<Cursor>,
    /// Replaces the scrollbar appearance.
    pub scrollbar: Option<Box<ScrollbarDelta>>,
    /// Replaces the node's effects.
    pub effects: Option<Option<Box<Effects>>>,
    /// Replaces the text color.
    pub text_color: Option<Rgba>,
}

/// Host-evaluated state styles. `disabled` and `selected` are resolved by the
/// core because the core owns those states.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StateStyles {
    /// Applied while hovered.
    pub hover: Option<PaintDelta>,
    /// Applied while pressed.
    pub pressed: Option<PaintDelta>,
    /// Applied while focused.
    pub focus: Option<PaintDelta>,
    /// Applied over `focus` while the focus is visible (the host asks for a
    /// focus indication: keyboard focus, not a pointer press).
    pub focus_visible: Option<PaintDelta>,
    /// Applied while the nearest ancestor that is a `group` is hovered (in
    /// the hover chain: the pointer is over it or a node below it), under
    /// the node's own `hover`.
    pub group_hover: Option<PaintDelta>,
    /// Applied while the focus is on the nearest ancestor that is a `group`
    /// or on a node below it, under the node's own `focus`.
    pub group_focus: Option<PaintDelta>,
    /// The node is a group for the `group_hover` and `group_focus` of the
    /// nodes below it.
    pub group: bool,
    /// The paint the node starts from when it is first attached (CSS
    /// `@starting-style`): its paint transition moves it to its paint. A
    /// node without a paint transition, or under reduced motion with a
    /// decorative one, shows its paint at once.
    pub enter: Option<PaintDelta>,
    /// The paint a removed node moves to by its paint transition before it
    /// goes: until every exit in the removed subtree ends, the subtree is
    /// drawn where it was but takes no hit, focus or accessibility node.
    /// Without a transition (or under reduced motion with a decorative
    /// one) the node goes at once.
    pub exit: Option<PaintDelta>,
}

impl StateStyles {
    /// Every delta it sets.
    pub fn deltas(&self) -> impl Iterator<Item = &PaintDelta> {
        [
            &self.hover,
            &self.pressed,
            &self.focus,
            &self.focus_visible,
            &self.group_hover,
            &self.group_focus,
            &self.enter,
            &self.exit,
        ]
        .into_iter()
        .flatten()
    }
}

/// Overlay scrollbar appearance; layout space is never reserved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarProps {
    /// Preferred border-box width.
    pub width: f64,
    /// Color of the thumb.
    pub thumb: Rgba,
    /// Color of the track.
    pub track: Rgba,
}

/// Sparse scrollbar state style; unset fields preserve the base style.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollbarDelta {
    /// Preferred border-box width.
    pub width: Option<f64>,
    /// Thumb and track colors.
    pub colors: Option<(Rgba, Rgba)>,
}

impl Default for ScrollbarProps {
    fn default() -> Self {
        Self {
            width: 6.0,
            thumb: Rgba {
                r: 0.45,
                g: 0.48,
                b: 0.52,
                a: 0.65,
            },
            track: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
        }
    }
}
