//! Role: Engine client node contracts.
use super::measure::LeafMeasure;
use kivact_types::{
    EditLineLayout, FocusTarget, ImageData, IntrinsicSize, Key, LayoutProps, Orientation,
    OverlayProps, PaintProps, StateStyles, TextData, TextStyle,
};

/// The kind of a host node: these five only. Native text editing is not a
/// kind but a payload any node can carry ([`NodeProperties::editing`]); a
/// new basic part is a `Box` leaf that kivact measures
/// ([`BoxData::measure`]), never a new kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeType {
    Root,
    Box,
    Text,
    Image,
    Overlay,
}

/// The payload corresponding to one node type.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    Root,
    Box(BoxData),
    Text(TextData),
    Image(ImageData),
    Overlay(Option<OverlayProps>),
}

/// A node's typed props: its kind's payload, the input flags, and the
/// rest boxed, so only the nodes that have them allocate them (a setter
/// compares before it allocates a box).
#[derive(Clone, Debug, PartialEq)]
pub struct NodeProperties {
    /// Node type and its corresponding payload.
    pub kind: NodeKind,
    /// Optional native editing contract.
    pub editing: Option<Box<EditingData>>,
    /// Optional IME session of a focusable node that is not a native editor.
    pub ime: Option<Box<ImeSessionProps>>,
    /// Optional layout properties; absent uses host defaults.
    pub layout: Option<Box<LayoutProps>>,
    /// Optional resolved paint properties.
    pub paint: Option<Box<PaintProps>>,
    /// Optional inherited text properties.
    pub text: Option<Box<TextStyle>>,
    /// Optional state-dependent style overrides.
    pub states: Option<Box<StateStyles>>,
    /// Input participation and focus flags.
    pub input: NodeInput,
    /// Optional accessible node properties.
    pub semantics: Option<Box<kivact_types::Semantics<FocusTarget>>>,
    /// The drawable this node is the root of, if any (its id and mode).
    pub drawable: Option<DrawableRole>,
    /// The shared-layout id: a node that gets its first box with the id of
    /// a departed node animates from that node's last box (`layout_id`).
    pub layout_id: Option<std::sync::Arc<str>>,
}

/// The root node of a drawable: which drawable it is and how it is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawableRole {
    /// The drawable.
    pub id: kivact_types::DrawableId,
    /// Whether the canvas draws it or the app exports it.
    pub mode: DrawableMode,
}

/// How a drawable's content is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawableMode {
    /// Drawn by the host canvas that places it (`Draw::drawable`).
    Canvas,
    /// Never drawn on screen; rendered for the app's export.
    Export,
}

/// What a focusable node that edits its own text (not a native editor) shows
/// the platform's input method: the text around its caret, the selection and
/// where the caret is. The engine allows the IME while the node has the
/// focus, places its candidate window at `caret` and answers the IME's
/// queries from this; the IME's composition, commits and reconversions
/// come back as [`super::HostEvent::Ime`].
#[derive(Clone, Debug, PartialEq)]
pub struct ImeSessionProps {
    /// The node's committed text (never the composition).
    pub text: String,
    /// The selection, `start..end` in UTF-8 bytes of `text` (empty: the caret).
    pub selection: std::ops::Range<usize>,
    /// The caret in the node's own coordinates (from its top left corner).
    pub caret: kivact_types::Rect,
    /// Password text: the IME is given no surrounding text.
    pub purpose: kivact_types::InputPurpose,
}

/// Optional input flags; absent values leave the host defaults in force.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NodeInput {
    /// Disable interactive behavior when true.
    pub disabled: Option<bool>,
    /// Exclude the node and subtree from display and interaction when true.
    pub hidden: Option<bool>,
    /// Allow the node to receive keyboard focus.
    pub focusable: Option<bool>,
    /// Keep an otherwise focusable node in focus order while disabled.
    pub focusable_when_disabled: Option<bool>,
    /// Group descendants into one stop in the native Tab order.
    pub roving_focus: Option<RovingFocus>,
    /// Request routed pointer events.
    pub listen_pointer: Option<bool>,
    /// Where a press moves the focus instead of this node; `None`: the host
    /// default.
    pub focus_on_press: Option<FocusTarget>,
    /// Request wheel events for the node.
    pub listen_wheel: Option<bool>,
    /// Request native file drag-and-drop events.
    pub listen_file: Option<bool>,
    /// `false`: the pointer passes through the node and its subtree to what
    /// lies behind (CSS `pointer-events: none`); `None` is `true`.
    pub hit_testable: Option<bool>,
}

/// Direction of a group whose active focus target moves with arrow keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RovingFocus {
    pub orientation: Orientation,
    pub rtl: bool,
}

/// Measurement callback for a childless box; absent means ordinary layout.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BoxData {
    /// Optional natural dimensions used to size the leaf box.
    pub intrinsic: Option<IntrinsicSize>,
    /// Pure measurement callback for a leaf box.
    pub measure: Option<LeafMeasure>,
}

/// Native text-editing behavior, coloring and constraints for a node.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EditingData {
    /// Synchronous native editing behavior.
    pub behavior: Option<kivact_editing::EditBehavior>,
    /// Optional line coloring callback.
    pub colors: Option<kivact_editing::LineColors>,
    /// Resolved document decorations.
    pub decorations: Option<Vec<kivact_editing::RangeDecoration>>,
    /// Keys bypassing native editing and its behavior, forwarded to client handlers.
    pub forward_keys: Option<Vec<Key>>,
    /// Controlled document value; identity and revision govern host adoption.
    pub value: Option<std::rc::Rc<str>>,
    /// Hide clipboard content and request protected native editing.
    pub purpose: Option<kivact_types::InputPurpose>,
    /// Prevent mutations while retaining selection and copy.
    pub read_only: Option<bool>,
    /// Maximum Unicode scalar count accepted by editing mutations.
    pub max_length: Option<usize>,
    /// Text displayed while the controlled document is empty.
    pub placeholder: Option<String>,
    /// Allow line breaks in the editable value.
    pub multiline: Option<bool>,
    /// Visual line layout of a multiline field.
    pub line_layout: Option<EditLineLayout>,
    /// Request document-change events.
    pub report_text: Option<bool>,
    /// Request selection and composition-session events.
    pub report_session: Option<bool>,
}

impl NodeType {
    /// Whether this kind participates as a structural parent.
    pub fn can_have_children(self) -> bool {
        matches!(self, Self::Root | Self::Box | Self::Overlay)
    }
}

impl NodeKind {
    /// Return the tag corresponding to this payload.
    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Root => NodeType::Root,
            Self::Box(_) => NodeType::Box,
            Self::Text(_) => NodeType::Text,
            Self::Image(_) => NodeType::Image,
            Self::Overlay(_) => NodeType::Overlay,
        }
    }
    /// Build the default payload for a node type.
    pub fn new(kind: NodeType) -> Self {
        match kind {
            NodeType::Root => Self::Root,
            NodeType::Box => Self::Box(Default::default()),
            NodeType::Text => Self::Text(Default::default()),
            NodeType::Image => Self::Image(Default::default()),
            NodeType::Overlay => Self::Overlay(None),
        }
    }
}

impl NodeProperties {
    /// Build properties containing this payload and no optional overrides.
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            editing: None,
            ime: None,
            layout: None,
            paint: None,
            text: None,
            states: None,
            input: Default::default(),
            semantics: None,
            drawable: None,
            layout_id: None,
        }
    }
}

/// Whether a node scene is painted below or above its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SceneLayer {
    /// Drawn behind children (canvas content, `draw`).
    Draw,
    /// Drawn over children (`draw_over`).
    DrawOver,
}
