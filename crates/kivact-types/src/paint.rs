//! Role: Shared node paint, image payloads, effects and state-dependent styling.
use crate::layout::IntrinsicSize;
use crate::{Asset, BorderStyle, Edges, Point, Rgba};

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
    pub extend: Extend,
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

/// How a layer's pixels combine with what is beneath them (CSS
/// `mix-blend-mode`; [`BlendMode::PlusLighter`] is canvas `lighter`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    /// Source over.
    #[default]
    Normal,
    /// Multiply.
    Multiply,
    /// Screen.
    Screen,
    /// Overlay.
    Overlay,
    /// Darken.
    Darken,
    /// Lighten.
    Lighten,
    /// Color dodge.
    ColorDodge,
    /// Color burn.
    ColorBurn,
    /// Hard light.
    HardLight,
    /// Soft light.
    SoftLight,
    /// Difference.
    Difference,
    /// Exclusion.
    Exclusion,
    /// Hue.
    Hue,
    /// Saturation.
    Saturation,
    /// Color.
    Color,
    /// Luminosity.
    Luminosity,
    /// Additive (CSS `plus-lighter`, canvas `lighter`).
    PlusLighter,
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
    pub width: f32,
    pub offset: f32,
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

/// A CSS basic shape in the border box (`clip-path`): x against the box
/// width, y against its height.
#[derive(Clone, Debug, PartialEq)]
pub enum ClipShape {
    Path {
        cmds: Vec<crate::scene::PathCmd>,
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

/// Shadows animated (more change at once).
const SHADOWS: usize = 2;

const BG: usize = 1;

const BORDER: usize = BG + 4;

const TRANSFORM: usize = BORDER + 16;

/// The transform's parts (`Transform::components`), then its origin's.
const SHADOW: usize = TRANSFORM + 14;

const FILTER: usize = SHADOW + 8 * SHADOWS;

/// brightness, contrast, saturate, grayscale, sepia, hue rotation, invert,
/// opacity, blur, drop shadow (x, y, blur, r, g, b, a).
const FILTER_LEN: usize = 16;

const LEN: usize = FILTER + FILTER_LEN;

/// The filter amounts that change nothing.
const IDENTITY: [f32; FILTER_LEN] = [
    1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintVec([f32; LEN]);

fn rgba(c: Rgba) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

fn color(v: &[f32]) -> Rgba {
    Rgba {
        r: v[0],
        g: v[1],
        b: v[2],
        a: v[3].clamp(0.0, 1.0),
    }
}

/// The filter of `p` as amounts in the canonical order (a function given
/// twice composes: amounts multiply, angles and blurs add).
fn filter_amounts(p: &PaintProps) -> [f32; FILTER_LEN] {
    let mut v = IDENTITY;
    for op in p.effects.iter().flat_map(|e| &e.filter) {
        match *op {
            FilterOp::Brightness(a) => v[0] *= a,
            FilterOp::Contrast(a) => v[1] *= a,
            FilterOp::Saturate(a) => v[2] *= a,
            FilterOp::Grayscale(a) => v[3] = a,
            FilterOp::Sepia(a) => v[4] = a,
            FilterOp::HueRotate(a) => v[5] += a,
            FilterOp::Invert(a) => v[6] = a,
            FilterOp::Opacity(a) => v[7] *= a,
            FilterOp::Blur(a) => v[8] += a,
            FilterOp::DropShadow {
                offset,
                blur,
                color,
            } => {
                v[9..12].copy_from_slice(&[offset.x, offset.y, blur]);
                v[12..16].copy_from_slice(&rgba(color));
            }
        }
    }
    v
}

/// The filter of the amounts `v`, in the canonical order, leaving out the
/// functions that change nothing.
fn filter_of(v: &[f32]) -> Vec<FilterOp> {
    let mut out = Vec::new();
    let mut push = |i: usize, op: fn(f32) -> FilterOp| {
        if (v[i] - IDENTITY[i]).abs() > 1e-4 {
            out.push(op(v[i]));
        }
    };
    push(0, FilterOp::Brightness);
    push(1, FilterOp::Contrast);
    push(2, FilterOp::Saturate);
    push(3, FilterOp::Grayscale);
    push(4, FilterOp::Sepia);
    push(5, FilterOp::HueRotate);
    push(6, FilterOp::Invert);
    push(7, FilterOp::Opacity);
    push(8, FilterOp::Blur);
    if v[15] > 1e-3 {
        out.push(FilterOp::DropShadow {
            offset: Point { x: v[9], y: v[10] },
            blur: v[11].max(0.0),
            color: color(&v[12..16]),
        });
    }
    out
}

impl PaintVec {
    pub fn of(p: &PaintProps) -> PaintVec {
        let mut v = [0.0; LEN];
        v[0] = p.opacity;
        if let Some(Fill::Solid(c)) = &p.background {
            v[BG..BORDER].copy_from_slice(&rgba(*c));
        }
        if let Some(b) = &p.border {
            let c = b.colors;
            for (i, side) in [c.top, c.right, c.bottom, c.left].into_iter().enumerate() {
                v[BORDER + 4 * i..BORDER + 4 * i + 4].copy_from_slice(&rgba(side));
            }
        }
        let t = p.transform;
        v[TRANSFORM..TRANSFORM + 10].copy_from_slice(&[
            t.translate.x,
            t.translate.y,
            t.per_size[0],
            t.per_size[1],
            t.per_size[2],
            t.per_size[3],
            t.rotate,
            t.skew_x,
            t.scale.x,
            t.scale.y,
        ]);
        let [x, y] = t.origin;
        v[TRANSFORM + 10..SHADOW].copy_from_slice(&[x.px, x.pct, y.px, y.pct]);
        for (i, s) in p.shadows.iter().take(SHADOWS).enumerate() {
            let at = SHADOW + 8 * i;
            v[at..at + 4].copy_from_slice(&[s.offset.x, s.offset.y, s.blur, s.spread]);
            v[at + 4..at + 8].copy_from_slice(&rgba(s.color));
        }
        v[FILTER..].copy_from_slice(&filter_amounts(p));
        PaintVec(v)
    }

    /// Put the animated components into `p` (the target paint).
    pub fn apply(&self, p: &mut PaintProps) {
        let v = &self.0;
        p.opacity = v[0].clamp(0.0, 1.0);
        let bg = color(&v[BG..BORDER]);
        match &mut p.background {
            Some(Fill::Solid(c)) => *c = bg,
            None if bg.a > 0.001 => p.background = Some(Fill::Solid(bg)),
            _ => {}
        }
        if let Some(b) = &mut p.border {
            let side = |i: usize| color(&v[BORDER + 4 * i..BORDER + 4 * i + 4]);
            b.colors = Edges {
                top: side(0),
                right: side(1),
                bottom: side(2),
                left: side(3),
            };
        }
        let parts: [f32; 10] = std::array::from_fn(|i| v[TRANSFORM + i]);
        let o = &v[TRANSFORM + 10..SHADOW];
        p.transform = crate::ElementTransform {
            translate: Point {
                x: parts[0],
                y: parts[1],
            },
            per_size: [parts[2], parts[3], parts[4], parts[5]],
            rotate: parts[6],
            skew_x: parts[7],
            scale: Point {
                x: parts[8],
                y: parts[9],
            },
            origin: [
                Len {
                    px: o[0],
                    pct: o[1],
                },
                Len {
                    px: o[2],
                    pct: o[3],
                },
            ],
        };
        for i in 0..SHADOWS {
            let at = SHADOW + 8 * i;
            let c = color(&v[at + 4..at + 8]);
            if p.shadows.len() == i && c.a > 0.001 {
                p.shadows.push(Shadow::default());
            }
            if let Some(s) = p.shadows.get_mut(i) {
                s.offset = Point {
                    x: v[at],
                    y: v[at + 1],
                };
                s.blur = v[at + 2].max(0.0);
                s.spread = v[at + 3];
                s.color = c;
            }
        }
        // The target's own filter, unless the amounts are on their way.
        let amounts = &v[FILTER..];
        let target = filter_amounts(p);
        if amounts
            .iter()
            .zip(target)
            .any(|(a, b)| (a - b).abs() > 1e-4)
        {
            p.effects.get_or_insert_with(Box::<Effects>::default).filter = filter_of(amounts);
        }
    }
}

impl PaintVec {
    pub const COMPONENTS: usize = LEN;
    pub fn components(&self) -> &[f32] {
        &self.0
    }
    pub fn from_components(c: &[f32]) -> Self {
        let mut v = [0.0; LEN];
        for (d, s) in v.iter_mut().zip(c) {
            *d = *s;
        }
        PaintVec(v)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Shadow {
    pub offset: Point,
    pub blur: f32,
    pub spread: f32,
    pub color: Rgba,
    /// Inside the box (CSS `inset`).
    pub inset: bool,
}

/// Element transform: CSS `translate`, `rotate`, `scale` and `transform`
/// with `transform-origin`, in one canonical form. About `origin`, the
/// box is scaled, skewed along x, rotated and moved (CSS
/// `translate(..) rotate(..) skewX(..) scale(..)`); it maps the node's box
/// and its whole subtree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementTransform {
    /// The move in px ...
    pub translate: Point,
    /// ... plus fractions of the box's own size: x moves by
    /// `per_size[0] × width + per_size[1] × height`, y by
    /// `per_size[2] × width + per_size[3] × height` (a percentage
    /// translation, mapped by what precedes it in the CSS list).
    pub per_size: [f32; 4],
    /// Clockwise, in degrees.
    pub rotate: f32,
    /// Along x, in degrees.
    pub skew_x: f32,
    pub scale: Point,
    /// Against the box's width and height (CSS `transform-origin`,
    /// default `50% 50%`).
    pub origin: [Len; 2],
}

impl ElementTransform {
    /// No transform (origin at the centre).
    pub const NONE: ElementTransform = ElementTransform {
        translate: Point { x: 0.0, y: 0.0 },
        per_size: [0.0; 4],
        rotate: 0.0,
        skew_x: 0.0,
        scale: Point { x: 1.0, y: 1.0 },
        origin: [Len::pct(50.0), Len::pct(50.0)],
    };

    /// Whether it moves nothing (whatever its origin).
    pub fn is_none(&self) -> bool {
        self.translate == Point::default()
            && self.per_size == [0.0; 4]
            && self.rotate == 0.0
            && self.skew_x == 0.0
            && self.scale == Point { x: 1.0, y: 1.0 }
    }
}

impl Default for ElementTransform {
    fn default() -> Self {
        ElementTransform::NONE
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintProps {
    /// Optional host-driven paint transition, absent on ordinary nodes.
    pub transition: Option<std::sync::Arc<super::motion::Transition>>,
    pub background: Option<Fill>,
    pub border: Option<Border>,
    /// Focus ring drawn outside the border box.
    pub ring: Option<Ring>,
    pub radius: Corners,
    /// CSS order: the first is on top.
    pub shadows: Vec<Shadow>,
    /// Group opacity of the node and its subtree.
    pub opacity: f32,
    pub transform: ElementTransform,
    /// The paint and hit order among its siblings (CSS `z-index`; ties
    /// keep the tree order).
    pub z_index: i32,
    pub cursor: Cursor,
    pub scrollbar: Option<Box<ScrollbarProps>>,
    /// Clip children to the padding box (rounded by the inner radii).
    pub clip: bool,
    /// Filters, backdrop, blending, mask and clip path (`None`: none).
    pub effects: Option<Box<Effects>>,
}

impl Default for PaintProps {
    fn default() -> Self {
        PaintProps {
            transition: None,
            background: None,
            border: None,
            ring: None,
            radius: Corners::default(),
            shadows: Vec::new(),
            opacity: 1.0,
            transform: ElementTransform::NONE,
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
    pub background: Option<Fill>,
    pub border: Option<Border>,
    pub ring: Option<Ring>,
    pub radius: Option<Corners>,
    pub opacity: Option<f32>,
    pub shadows: Option<Vec<Shadow>>,
    pub transform: Option<ElementTransform>,
    pub z_index: Option<i32>,
    pub cursor: Option<Cursor>,
    pub scrollbar: Option<Box<ScrollbarDelta>>,
    /// Replaces the node's effects.
    pub effects: Option<Option<Box<Effects>>>,
    pub text_color: Option<Rgba>,
}

/// Host-evaluated state styles. `disabled` and `selected` are resolved by the
/// core because the core owns those states.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StateStyles {
    pub hover: Option<PaintDelta>,
    pub pressed: Option<PaintDelta>,
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
    pub width: f32,
    pub thumb: Rgba,
    pub track: Rgba,
}

/// Sparse scrollbar state style; unset fields preserve the base style.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollbarDelta {
    /// Preferred border-box width.
    pub width: Option<f32>,
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
