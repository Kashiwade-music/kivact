//! Role: The engine's DevTools contracts: overlays, keys, pick phases and records.
use kivact_types::export::FrameStats;
use kivact_types::{
    InputPurpose, Key, Modifiers, NodeId, Point, Rect, Revision, Rgba, ThemeMode, WindowId,
};

/// What the DevTools draws over one window.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DevtoolsOverlay {
    pub boxes: Vec<OverlayBox>,
    pub outlines: Vec<OverlayOutline>,
    pub fills: Vec<OverlayFill>,
    pub labels: Vec<OverlayLabel>,
}

/// The margin, border, padding and content rectangles of one box.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayBox {
    pub margin: Rect,
    pub border: Rect,
    pub padding: Rect,
    pub content: Rect,
}

/// A stroked rectangle of the overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayOutline {
    pub rect: Rect,
    pub color: Rgba,
    pub width: f64,
}

/// A filled rectangle of the overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayFill {
    pub rect: Rect,
    pub color: Rgba,
}

/// A text label of the overlay at a window point.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayLabel {
    pub at: Point,
    pub text: String,
}

/// What a pick event did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickPhase {
    Hover,
    Select,
    Cancel,
}

/// One key with the modifiers held for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyChord {
    pub key: Key,
    pub modifiers: Modifiers,
}

/// The keys that open the DevTools and those that open it in pick mode.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DevtoolsKeys {
    pub open: Vec<KeyChord>,
    pub pick: Vec<KeyChord>,
}

/// One sync report of a window.
#[derive(Clone, Debug, PartialEq)]
pub struct SyncRecord {
    pub window: WindowId,
    pub revision: Revision,
    pub reasons: u8,
    pub measured: Vec<NodeId>,
    pub laid_out: Vec<NodeId>,
    pub redrawn: Vec<NodeId>,
}

/// One presented frame.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameRecord {
    pub window: WindowId,
    pub revision: Revision,
    pub end_ns: u64,
    pub work_ns: u64,
    pub scene_ns: u64,
    pub raster_ns: u64,
    pub present_ns: u64,
    pub stats: FrameStats,
    pub gpu: bool,
    pub frame_period_ns: u64,
}

/// What the core takes each turn; `dropped` counts the records the ring lost.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EngineRecords {
    pub syncs: Vec<SyncRecord>,
    pub frames: Vec<FrameRecord>,
    pub dropped: u32,
}

/// The environment emulation the DevTools asks of every window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EngineEmulation {
    pub theme_hint: Option<ThemeMode>,
    pub scale: Option<f64>,
    pub touch: Option<bool>,
}

/// The margin, border, padding and content rectangles of one node in window
/// coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct BoxModelRects {
    pub margin: Rect,
    pub border: Rect,
    pub padding: Rect,
    pub content: Rect,
}

/// The text input state of one window for the DevTools.
#[derive(Clone, Debug, PartialEq)]
pub enum ImeStateInfo {
    /// No text input has focus in the window.
    Inactive,
    /// The text input of `node` has focus.
    Focused {
        /// The node that receives text.
        node: NodeId,
        /// The software keyboard hint of the focused input.
        purpose: InputPurpose,
        /// The area of the cursor, when the window has one.
        cursor_area: Option<Rect>,
        /// The text being composed (`composing` in the protocol means
        /// `composition.is_some()`).
        composition: Option<String>,
    },
}

/// A change the host applies to `DevtoolsHost` after the turn.
#[cfg(feature = "devtools")]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DevtoolsChange {
    Overlay(WindowId, DevtoolsOverlay),
    Pick(Option<WindowId>),
    Keys(DevtoolsKeys),
}

/// The nodes the client created or wrote (props, scenes, children) and
/// the nodes destroyed since the last `Reader::take_tree_changes`, while
/// `Reader::record_tree_changes` is on. Each list is in (window, id) order; a node
/// destroyed after a write is only in `removed`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TreeChanges {
    pub upserted: Vec<(WindowId, NodeId)>,
    pub removed: Vec<(WindowId, NodeId)>,
}
