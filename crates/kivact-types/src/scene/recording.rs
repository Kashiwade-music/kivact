//! Role: Authored draw records and their geometry and text payloads.
use crate::scene::primitives::Stroke;
use crate::scene::{FillRule, PathData};
use crate::{
    BlendMode, ClipShape, Corners, Fill, FillLayer, FilterOp, FontFeature, LineHeight, Point, Rect,
    RegionId, Rgba, TextDecoration,
};

/// Resolved geometry for clipping or painting in logical coordinates.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawingShape {
    Basic { shape: ClipShape, rect: Rect },
    Path { path: PathData, rule: FillRule },
    Rounded { rect: Rect, corners: Corners },
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
    /// Compositing operation against the backdrop.
    pub blend: BlendMode,
    /// Filters applied in order to the layer.
    pub filter: Vec<FilterOp>,
    /// Optional paint layer used as an alpha mask.
    pub mask: Option<FillLayer>,
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

/// Straight-alpha RGBA8 pixels a scene draws as they are (no decode, no
/// asset store). Compared by size and `Arc` identity, never by content, so
/// a new `Arc` every frame costs one comparison.
#[derive(Clone, Debug)]
pub struct Pixels {
    /// Pixel width of the buffer.
    pub width: u32,
    /// Pixel height of the buffer.
    pub height: u32,
    /// Tightly packed straight-alpha RGBA8, row-major from the top left,
    /// width times height times four bytes with no row padding.
    /// Keep the same allocation for unchanged pixels; renderers cache by Arc identity.
    pub rgba: std::sync::Arc<[u8]>,
}

impl Pixels {
    /// Construct an image without copying its shared pixel allocation.
    pub fn new(width: u32, height: u32, rgba: std::sync::Arc<[u8]>) -> Self {
        Self {
            width,
            height,
            rgba,
        }
    }
}

impl PartialEq for Pixels {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && std::sync::Arc::ptr_eq(&self.rgba, &other.rgba)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextRun {
    pub text: String,
    pub origin: Point,
    pub font_size: f64,
    pub weight: u16,
    pub color: Rgba,
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
    /// Lines drawn with it (`em` lengths are of `font_size`).
    pub decoration: TextDecoration,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DrawCmd {
    Pixels {
        image: crate::Pixels,
        rect: Rect,
        source: Option<Rect>,
        opacity: f64,
        pixelated: bool,
    },
    /// A frame of an image an external render engine shares, drawn like `Pixels` on the GPU of the
    /// window it was imported into; nothing elsewhere (Arvane's `RenderCmd::External`).
    External {
        frame: crate::ExternalFrame,
        rect: Rect,
        source: Option<Rect>,
        opacity: f64,
        pixelated: bool,
    },
    ImagePart {
        source: crate::Asset,
        rect: Rect,
        crop: Option<Rect>,
        opacity: f64,
        pixelated: bool,
    },
    Shape(Box<crate::ShapePaint>),
    PushLayer(Box<crate::DrawingLayer>),
    PopLayer,
    PushTransform(crate::geometry::Transform),
    PopTransform,
    Rect {
        rect: Rect,
        fill: Fill,
    },
    RoundedRect {
        rect: Rect,
        radius: f64,
        fill: Fill,
    },
    StrokeRect {
        rect: Rect,
        width: f64,
        color: Rgba,
        radius: f64,
    },
    Line {
        from: Point,
        to: Point,
        width: f64,
        color: Rgba,
    },
    Text(TextRun),
    /// A paragraph the client shaped through the host (`TextShaper::shape`),
    /// drawn exactly as shaped: its glyph runs and decorations at `origin`
    /// in `color`, with no shaping again.
    Paragraph {
        shaped: std::sync::Arc<crate::render::ShapedText>,
        origin: Point,
        color: Rgba,
    },
    Image {
        source: crate::Asset,
        rect: Rect,
    },
    PushClip(Rect),
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
    pub shape: Option<crate::DrawingShape>,
    pub transform: crate::geometry::Transform,
    pub local_rect: Rect,
    pub id: RegionId,
    pub rect: Rect,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DrawList {
    pub cmds: Vec<DrawCmd>,
    pub regions: Vec<Region>,
}

impl DrawList {
    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty() && self.regions.is_empty()
    }
}
