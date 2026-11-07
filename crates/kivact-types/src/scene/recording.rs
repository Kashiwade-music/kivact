//! Role: Authored draw records and their geometry and text payloads.
use crate::geometry3d::{Matrix4, Projective};
use crate::paint::{BackfaceVisibility, BorderImage, Fill, FilterOp, MaskLayer, TransformStyle};
use crate::pixel::BlendMode;
use crate::render::Composite;
use crate::scene::primitives::Stroke;
use crate::scene::{FillRule, PathData};
use crate::{
    ClipShape, Corners, FontFeature, FontVariation, LineHeight, Point, Rect, RegionId, Rgba,
    TextDecoration,
};

/// Resolved geometry for clipping or painting in logical coordinates.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawingShape {
    /// A basic CSS shape within its rectangle.
    Basic {
        /// The shape.
        shape: ClipShape,
        /// Its box.
        rect: Rect,
    },
    /// A path.
    Path {
        /// The path.
        path: PathData,
        /// Its fill rule.
        rule: FillRule,
    },
    /// A rounded rectangle.
    Rounded {
        /// Its box.
        rect: Rect,
        /// Its corner radii.
        corners: Corners,
    },
}

/// Compositing scope applied to enclosed scene commands.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawingLayer {
    /// Optional clipping geometry for the layer.
    pub clip: Option<DrawingShape>,
    /// Logical bounds of the compositing layer.
    pub area: Rect,
    /// Layer opacity multiplier; one is opaque.
    pub opacity: f64,
    /// Blend with what is beneath.
    pub blend: BlendMode,
    /// Porter–Duff operator with what is beneath.
    pub composite: Composite,
    /// Filters applied in order to the layer.
    pub filter: Vec<FilterOp>,
    /// Alpha mask layers over the layer (CSS `mask`).
    pub masks: Vec<MaskLayer>,
    /// A non-affine (3D) transform of the layer about the recording origin;
    /// affine transforms stay PushTransform.
    pub transform: Option<Matrix4>,
    /// CSS `transform-style` of the layer.
    pub transform_style: TransformStyle,
    /// CSS `backface-visibility` of the layer.
    pub backface: BackfaceVisibility,
}

/// A resolved shape and its fill or optional stroked outline.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapePaint {
    /// Geometry to fill or stroke.
    pub shape: DrawingShape,
    /// Fill used for the shape or its stroke.
    pub paint: Fill,
    /// Stroke parameters; absent paints the filled interior.
    pub stroke: Option<Stroke>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextRun {
    /// The text.
    pub text: String,
    /// The origin of the run.
    pub origin: Point,
    /// The font size in logical px.
    pub font_size: f64,
    /// The font weight.
    pub weight: u16,
    /// The text color, used when `fill` is None.
    pub color: Rgba,
    /// Fill paint; None: `color`.
    pub fill: Option<Fill>,
    /// Outline stroke and its paint.
    pub stroke: Option<(Stroke, Fill)>,
    /// A family name or a generic family (`system-ui`, `sans-serif`,
    /// `serif`, `monospace`); `None`: the family of the node the scene
    /// belongs to (its inherited text style).
    pub family: Option<String>,
    /// Use the italic face of the family when it has one.
    pub italic: bool,
    /// The distance between lines; `None`: the line height of the node the
    /// scene belongs to (its inherited text style).
    pub line_height: Option<LineHeight>,
    /// Extra space after every character, in px.
    pub letter_spacing: f64,
    /// Extra space after every space, in px.
    pub word_spacing: f64,
    /// Pair kerning.
    pub kerning: crate::FontKerning,
    /// OpenType features.
    pub features: Vec<FontFeature>,
    /// Variable font axes.
    pub variations: Vec<FontVariation>,
    /// CSS font-width in percent.
    pub font_width: f64,
    /// Lines drawn with it (`em` lengths are of `font_size`).
    pub decoration: TextDecoration,
}

/// One authored draw command of a recording.
#[derive(Clone, Debug, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "authored once per draw command and moved into the recording; boxing the image payload would allocate per command"
)]
pub enum DrawCmd {
    /// A pixel image drawn into a rectangle.
    Pixels {
        /// The pixels.
        image: crate::Pixels,
        /// Destination.
        rect: Rect,
        /// Source pixels; None draws the whole image.
        source: Option<Rect>,
        /// Group opacity.
        opacity: f64,
        /// Disable smoothing.
        pixelated: bool,
    },
    /// A frame of an image an external render engine shares, drawn like `Pixels` on the GPU of the
    /// window it was imported into; nothing elsewhere (Arvane's `RenderCmd::External`).
    External {
        /// The frame.
        frame: crate::ExternalFrame,
        /// Pixels the CPU draws instead.
        cpu_pixels: Option<crate::Pixels>,
        /// Destination.
        rect: Rect,
        /// Source pixels; None draws the whole frame.
        source: Option<Rect>,
        /// Group opacity.
        opacity: f64,
        /// Disable smoothing.
        pixelated: bool,
    },
    /// A part of an asset image.
    ImagePart {
        /// The asset.
        source: crate::Asset,
        /// Destination.
        rect: Rect,
        /// Crop in the asset.
        crop: Option<Rect>,
        /// Group opacity.
        opacity: f64,
        /// Disable smoothing.
        pixelated: bool,
    },
    /// A shape with its paint.
    Shape(Box<crate::ShapePaint>),
    /// Open a compositing layer.
    PushLayer(Box<DrawingLayer>),
    /// Close the innermost compositing layer.
    PopLayer,
    /// Open a transform.
    PushTransform(crate::geometry::Transform),
    /// Close the innermost transform.
    PopTransform,
    /// A filled rectangle.
    Rect {
        /// Bounds.
        rect: Rect,
        /// Paint.
        fill: Fill,
    },
    /// A filled rounded rectangle.
    RoundedRect {
        /// Bounds.
        rect: Rect,
        /// Radius.
        radius: f64,
        /// Paint.
        fill: Fill,
    },
    /// A stroked rectangle.
    StrokeRect {
        /// Bounds.
        rect: Rect,
        /// Stroke width.
        width: f64,
        /// Color.
        color: Rgba,
        /// Radius.
        radius: f64,
    },
    /// A straight line.
    Line {
        /// Start.
        from: Point,
        /// End.
        to: Point,
        /// Width.
        width: f64,
        /// Color.
        color: Rgba,
    },
    /// A text run, shaped when drawn.
    Text(TextRun),
    /// A paragraph the client shaped through the host (`TextShaper::shape`),
    /// drawn exactly as shaped: its glyph runs and decorations at `origin`
    /// with `fill` and the optional outline `stroke`, with no shaping again.
    Paragraph {
        /// The shaped paragraph.
        shaped: crate::ShapedTextSnapshot,
        /// Origin.
        origin: Point,
        /// Fill paint (`Fill::Solid` for one color).
        fill: Fill,
        /// Outline stroke and its paint.
        stroke: Option<(Stroke, Fill)>,
    },
    /// An image stretched to a rectangle.
    Image {
        /// The asset.
        source: crate::Asset,
        /// Destination.
        rect: Rect,
    },
    /// A drawable's subtree at `at` under the open transforms (§10).
    Drawable {
        /// The drawable.
        id: crate::ids::DrawableId,
        /// Where it is placed.
        at: Point,
    },
    /// A nine-slice image / border image over `rect`.
    BorderImage {
        /// Destination box.
        rect: Rect,
        /// The border image.
        image: Box<BorderImage>,
    },
    /// Intersect the clip with a rectangle.
    PushClip(Rect),
    /// Close the innermost clip.
    PopClip,
}

/// A hit and accessibility region inside a node's scene. Exposed by hosts as
/// a flat a11y leaf directly under the node.
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    /// Stable declaration order for keyboard navigation, independent of paint order.
    pub focus_order: u32,
    /// Native pointer focus destination; never changes activation identity.
    pub focus_on_press: Option<crate::FocusTarget>,
    /// Canonical accessible identity, state, relations and text properties.
    pub semantics: crate::semantics::Semantics<crate::FocusTarget>,
    /// The region's shape, if any.
    pub shape: Option<DrawingShape>,
    /// Region space → recording space.
    pub transform: Projective,
    /// The region's rectangle in region space.
    pub local_rect: Rect,
    /// Identity of the region.
    pub id: RegionId,
    /// The region's rectangle in recording space.
    pub rect: Rect,
}

/// The commands and regions of one recording.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DrawList {
    /// The draw commands in order.
    pub cmds: Vec<DrawCmd>,
    /// The hit and accessibility regions.
    pub regions: Vec<Region>,
}

impl DrawList {
    /// Whether the list has neither commands nor regions.
    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty() && self.regions.is_empty()
    }
}
