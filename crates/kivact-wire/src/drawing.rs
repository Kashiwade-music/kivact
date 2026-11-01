//! Role: Resolved canvas geometry and layers; no backend types cross the wire.
use super::{BlendMode, ClipShape, Corners, Fill, FillLayer, FilterOp, PathCmd, Rect};

/// Resolved geometry for clipping or painting in logical coordinates.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawingShape {
    Basic { shape: ClipShape, rect: Rect },
    Path { cmds: Vec<PathCmd>, even_odd: bool },
    Rounded { rect: Rect, corners: Corners },
}

pub use ui_types::LineCap;
pub use ui_types::LineJoin;
pub use ui_types::Stroke;
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
