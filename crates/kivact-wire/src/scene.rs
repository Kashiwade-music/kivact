//! Role: Draw scenes (canvas and per-node draw layers) and hit/semantics regions.
//!
//! Coordinates are node-local logical pixels. Push/pop commands preserve
//! transform and compositing scopes until the engine lowers them to Arvane.
//! The rarer commands keep their values in a box (`Shape`, `PushLayer`), so
//! they do not grow the command a scene of plain rects is made of.

use super::RegionId;
use super::paint::Fill;
use super::semantics::{A11yAction, Role};
use super::text::{FontFeature, LineHeight, TextDecoration};
use super::values::Rgba;
use super::{Point, Rect};

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
    pub kerning: super::FontKerning,
    /// OpenType features.
    pub features: Vec<FontFeature>,
    /// Lines drawn with it (`em` lengths are of `font_size`).
    pub decoration: TextDecoration,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SceneCmd {
    Pixels {
        image: super::Pixels,
        rect: Rect,
        source: Option<Rect>,
        opacity: f32,
        pixelated: bool,
    },
    /// A frame of an image an external render engine shares, drawn like `Pixels` on the GPU of the
    /// window it was imported into; nothing elsewhere (Arvane's `DrawCommand::External`).
    External {
        frame: kivact_types::ExternalFrame,
        rect: Rect,
        source: Option<Rect>,
        opacity: f32,
        pixelated: bool,
    },
    ImagePart {
        source: super::Asset,
        rect: Rect,
        crop: Option<Rect>,
        opacity: f32,
        pixelated: bool,
    },
    Shape(Box<super::ShapePaint>),
    PushLayer(Box<super::DrawingLayer>),
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
        shaped: std::sync::Arc<kivact_types::protocol::ShapedText>,
        origin: Point,
        color: Rgba,
    },
    Image {
        source: kivact_types::Asset,
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
    pub focusable: bool,
    /// Native pointer focus destination; never changes activation identity.
    pub focus_on_press: Option<super::FocusTarget>,
    pub disabled: bool,
    pub hidden: bool,
    pub description: Option<String>,
    /// Checked state of the region (`None`: not a toggle).
    pub checked: Option<super::CheckedState>,
    /// Selection state.
    pub selected: bool,
    /// Disclosure state (`None`: not expandable).
    pub expanded: Option<bool>,
    /// The region's value is invalid.
    pub invalid: bool,
    pub properties: Option<Box<super::SemanticsProperties>>,
    pub shape: Option<super::DrawingShape>,
    pub transform: [f32; 6],
    pub local_rect: Rect,
    pub id: RegionId,
    pub rect: Rect,
    pub role: Role,
    pub name: String,
    pub value: Option<String>,
    pub actions: Vec<A11yAction>,
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
