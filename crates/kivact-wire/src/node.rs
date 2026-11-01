//! Role: Node properties and structure.
//!
//! What kivact writes into the engine's tree: node kinds, props, scene
//! layers and platform requests (`Writer`).

use std::path::PathBuf;

use super::ids::{FocusTarget, NodeId};
use super::measure::LeafMeasure;
use super::semantics::{A11yAction, Role, SemanticsProperties};
use super::style::{LayoutProps, PaintProps, StateStyles};
use super::text::{TextProps, TextSpan};
use super::{Point, Size};
use kivact_types::Key;
use kivact_types::{Asset, ImageFit};

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
    pub ime: Option<Box<ImeSession>>,
    /// Optional layout properties; absent uses host defaults.
    pub layout: Option<Box<LayoutProps>>,
    /// Optional resolved paint properties.
    pub paint: Option<Box<PaintProps>>,
    /// Optional inherited text properties.
    pub text: Option<Box<TextProps>>,
    /// Optional state-dependent style overrides.
    pub states: Option<Box<StateStyles>>,
    /// Input participation and focus flags.
    pub input: NodeInput,
    /// Optional accessible node properties.
    pub semantics: Option<Box<NodeSemantics>>,
}
/// What a focusable node that edits its own text (not a native editor) shows
/// the platform's input method: the text around its caret, the selection and
/// where the caret is. The engine allows the IME while the node has the
/// focus, places its candidate window at `caret` and answers the IME's
/// queries from this; the IME's composition, commits and reconversions
/// come back as [`crate::HostEvent::Ime`].
#[derive(Clone, Debug, PartialEq)]
pub struct ImeSession {
    /// The node's committed text (never the composition).
    pub text: String,
    /// The selection, `start..end` in UTF-8 bytes of `text` (empty: the caret).
    pub selection: std::ops::Range<usize>,
    /// The caret in the node's own coordinates (from its top left corner).
    pub caret: crate::Rect,
    /// Password text: the IME is given no surrounding text.
    pub password: bool,
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
    /// Request routed pointer events.
    pub listen_pointer: Option<bool>,
    /// Override the press focus target; an inner None suppresses focusing.
    pub focus_on_press: Option<Option<FocusTarget>>,
    /// Request wheel events for the node.
    pub listen_wheel: Option<bool>,
    /// Request native file drag-and-drop events.
    pub listen_file: Option<bool>,
    /// `false`: the pointer passes through the node and its subtree to what
    /// lies behind (CSS `pointer-events: none`); `None` is `true`.
    pub hit_testable: Option<bool>,
}
/// Accessible identity, state and actions attached to a node.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NodeSemantics {
    /// Accessible role of the node.
    pub role: Option<Role>,
    /// Accessible name.
    pub name: Option<String>,
    /// Additional accessible description.
    pub description: Option<String>,
    /// Accessible text value.
    pub value: Option<String>,
    /// Accessible selection state.
    pub selected: Option<bool>,
    /// Checked state, with an inner None representing mixed.
    pub checked: Option<Option<bool>>,
    /// Accessible expansion state.
    pub expanded: Option<Option<bool>>,
    /// Whether the accessible value is invalid.
    pub invalid: Option<bool>,
    /// Additional structured accessibility properties.
    pub accessibility: Option<Box<SemanticsProperties>>,
    /// Actions the node offers to assistive technology.
    pub actions: Option<Vec<A11yAction>>,
    /// Stable identifier used by public screen queries.
    pub test_id: Option<String>,
}
/// Measurement callback for a childless box; absent means ordinary layout.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BoxData {
    /// Optional natural dimensions used to size the leaf box.
    pub intrinsic: Option<IntrinsicSize>,
    /// Pure measurement callback for a leaf box.
    pub measure: Option<LeafMeasure>,
}
/// Owned text content and optional per-span styling.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextData {
    /// Owned text displayed by this node.
    pub content: Option<String>,
    /// Per-range overrides over the node text.
    pub spans: Option<Vec<TextSpan>>,
}
/// Native text-editing behavior, coloring and constraints for a node.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EditingData {
    /// Synchronous native editing behavior.
    pub behavior: Option<kivact_editing::EditBehavior>,
    /// Optional line coloring callback.
    pub colors: Option<kivact_editing::LineColors>,
    /// Keys bypassing native editing and its behavior, forwarded to client handlers.
    pub forward_keys: Option<Vec<Key>>,
    /// Controlled document value; identity and revision govern host adoption.
    pub value: Option<std::rc::Rc<str>>,
    /// Hide clipboard content and request protected native editing.
    pub password: Option<bool>,
    /// Prevent mutations while retaining selection and copy.
    pub read_only: Option<bool>,
    /// Maximum Unicode scalar count accepted by editing mutations.
    pub max_length: Option<usize>,
    /// Text displayed while the controlled document is empty.
    pub placeholder: Option<String>,
    /// Allow line breaks in the editable value.
    pub multiline: Option<bool>,
    /// Request document-change events.
    pub report_text: Option<bool>,
    /// Request selection and composition-session events.
    pub report_session: Option<bool>,
}
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
        }
    }
}
/// Side of the anchor an overlay asks for. The engine flips it to the
/// opposite side when it does not fit and clamps it into the window
/// according to the host's placement policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Below,
    Above,
    Start,
    End,
    Center,
}

/// Where an overlay sits along its anchor's side: its start, centre or
/// end (`Below` with `End`: right edges aligned).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlacementAlign {
    #[default]
    Start,
    Center,
    End,
}

/// Props of an `Overlay` node. While it is shown, a host sends
/// `HostEvent::OverlayDismiss` to an overlay whose `Actions`
/// offer `Dismiss` when a pointer press lands outside its subtree and its
/// anchor's subtree. A `modal` overlay confines input to its subtree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayProps {
    /// Node whose bounds anchor the overlay.
    pub anchor: Option<NodeId>,
    /// A window point to place it at instead of a node (wins over
    /// `anchor`).
    pub point: Option<Point>,
    /// Requested side of the anchor.
    pub placement: Placement,
    /// Alignment along the requested anchor side.
    pub align: PlacementAlign,
    /// The gap between the anchor and the overlay (logical px).
    pub offset: f32,
    /// Confine interaction to this overlay while shown.
    pub modal: bool,
    /// Lay the overlay out at least as wide as its anchor node's layout
    /// box (not a point's): its minimum width is raised to that width.
    pub at_least_anchor_width: bool,
}

pub use kivact_types::IntrinsicSize;

/// Whether a node scene is painted below or above its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SceneLayer {
    /// Drawn behind children (canvas content, `draw`).
    Draw,
    /// Drawn over children (`draw_over`).
    DrawOver,
}

/// Initial location, suggested name and filters for a file dialog.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FileDialogRequest {
    /// Dialog title.
    pub title: String,
    /// Initial directory, when the platform supports it.
    pub directory: Option<PathBuf>,
    /// Suggested file name.
    pub file_name: Option<String>,
    /// (label, extensions without dot)
    pub filters: Vec<(String, Vec<String>)>,
}

/// A requested native platform action, completed through host events.
#[derive(Clone, Debug, PartialEq)]
pub enum PlatformRequest {
    /// Request logical content dimensions.
    RequestWindowSize(Size),
    Fullscreen(bool),
    CopyText(String),
    ReadText,
    PickFile(FileDialogRequest),
    PickFiles(FileDialogRequest),
    SaveFile(FileDialogRequest),
    /// Ask for a folder; the result is `Path` (`None` when cancelled).
    /// `file_name` and `filters` do not apply to a folder.
    PickFolder(FileDialogRequest),
    /// Open a file or folder with the platform's default handler. The
    /// result is `Done` once the handler was started, `Failed` naming the
    /// path and the reason, or `Unsupported` on a platform without it.
    OpenPath(PathBuf),
    /// Show a file or folder selected in the platform's file manager;
    /// results as for `OpenPath`.
    RevealPath(PathBuf),
    /// Register the fonts of a TTF, OTF or TTC file with the host's text
    /// system (always available). The result is
    /// `PlatformResult::Fonts` with the family names in the file, or
    /// `Failed` with a message naming the source and the reason. The
    /// engine handles it when it is written: the next layout already
    /// measures with the font, and text measured before is measured and
    /// drawn again.
    RegisterFont(Asset),
}
