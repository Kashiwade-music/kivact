//! Role: Authored draw records and their geometry and text payloads.
use crate::protocol::shape::PathCmd;
use crate::scene::primitives::Stroke;
use crate::{
    BlendMode, ClipShape, Corners, Fill, FillLayer, FilterOp, FontFeature, LineHeight, Point, Rect,
    RegionId, Rgba, TextDecoration,
};

/// Resolved geometry for clipping or painting in logical coordinates.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawingShape {
    Basic { shape: ClipShape, rect: Rect },
    Path { cmds: Vec<PathCmd>, even_odd: bool },
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
    pub opacity: f32,
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
    /// Tightly packed RGBA8 bytes, width times height times four.
    pub rgba: std::sync::Arc<[u8]>,
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
    pub font_size: f32,
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
    pub letter_spacing: f32,
    /// Extra space after every space, in px.
    pub word_spacing: f32,
    /// Pair kerning.
    pub kerning: crate::FontKerning,
    /// OpenType features.
    pub features: Vec<FontFeature>,
    /// Lines drawn with it (`em` lengths are of `font_size`).
    pub decoration: TextDecoration,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SceneCmd {
    Pixels {
        image: crate::Pixels,
        rect: Rect,
        source: Option<Rect>,
        opacity: f32,
        pixelated: bool,
    },
    /// A frame of an image an external render engine shares, drawn like `Pixels` on the GPU of the
    /// window it was imported into; nothing elsewhere (Arvane's `DrawCommand::External`).
    External {
        frame: crate::ExternalFrame,
        rect: Rect,
        source: Option<Rect>,
        opacity: f32,
        pixelated: bool,
    },
    ImagePart {
        source: crate::Asset,
        rect: Rect,
        crop: Option<Rect>,
        opacity: f32,
        pixelated: bool,
    },
    Shape(Box<crate::ShapePaint>),
    PushLayer(Box<crate::DrawingLayer>),
    PopLayer,
    PushTransform([f32; 6]),
    PopTransform,
    Rect {
        rect: Rect,
        fill: Fill,
    },
    RoundedRect {
        rect: Rect,
        radius: f32,
        fill: Fill,
    },
    StrokeRect {
        rect: Rect,
        width: f32,
        color: Rgba,
        radius: f32,
    },
    Line {
        from: Point,
        to: Point,
        width: f32,
        color: Rgba,
    },
    Text(TextRun),
    /// A paragraph the client shaped through the host (`TextShaper::shape`),
    /// drawn exactly as shaped: its glyph runs and decorations at `origin`
    /// in `color`, with no shaping again.
    Paragraph {
        shaped: std::sync::Arc<crate::protocol::ShapedText>,
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
    pub transform: [f32; 6],
    pub local_rect: Rect,
    pub id: RegionId,
    pub rect: Rect,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Scene {
    pub cmds: Vec<SceneCmd>,
    pub regions: Vec<Region>,
}

impl Scene {
    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty() && self.regions.is_empty()
    }
}
