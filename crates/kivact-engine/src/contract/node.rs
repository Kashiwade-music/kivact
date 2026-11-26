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
/// rest boxed, or shared (`Rc`) when many nodes write equal values
/// (`Model::set_shared_prop`), so only the nodes that have them allocate
/// them (a setter compares before it allocates a box).
#[derive(Clone, Debug, PartialEq)]
pub struct NodeProperties {
    /// Node type and its corresponding payload.
    pub kind: NodeKind,
    /// Optional native editing contract.
    pub editing: Option<Box<EditingData>>,
    /// Optional IME session of a focusable node that is not a native editor.
    pub ime: Option<Box<ImeSessionProps>>,
    /// Optional layout properties; absent uses host defaults.
    pub layout: Option<std::rc::Rc<LayoutProps>>,
    /// Optional resolved paint properties.
    pub paint: Option<std::rc::Rc<PaintProps>>,
    /// Optional inherited text properties.
    pub text: Option<std::rc::Rc<TextStyle>>,
    /// Optional state-dependent style overrides.
    pub states: Option<std::rc::Rc<StateStyles>>,
    /// Input participation and focus flags.
    pub input: NodeInput,
    /// Optional accessible node properties.
    pub semantics: Option<std::rc::Rc<kivact_types::Semantics<FocusTarget>>>,
    /// The drawable this node is the root of, if any (its id and mode).
    pub drawable: Option<DrawableRole>,
    /// The shared-layout id: a node that gets its first box with the id of
    /// a departed node animates from that node's last box (`layout_id`).
    pub layout_id: Option<std::sync::Arc<str>>,
}

const _: () = assert!(
    std::mem::size_of::<NodeProperties>() <= 232,
    "NodeProperties grew: box the new rare field"
);

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
    /// The yes/no/unset input flags ([`InputFlag`]).
    pub flags: InputFlags,
    /// Group descendants into one stop in the native Tab order.
    pub roving_focus: Option<RovingFocus>,
    /// Where a press moves the focus instead of this node; `None`: the host
    /// default.
    pub focus_on_press: Option<FocusTarget>,
}

/// One input flag of a node (see [`InputFlags`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputFlag {
    /// Disable interactive behavior when true.
    Disabled,
    /// Exclude the node and subtree from display and interaction when true.
    Hidden,
    /// Allow the node to receive keyboard focus.
    Focusable,
    /// Keep an otherwise focusable node in focus order while disabled.
    FocusableWhenDisabled,
    /// Request routed pointer events.
    ListenPointer,
    /// Request wheel events for the node.
    ListenWheel,
    /// Request native file drag-and-drop events.
    ListenFile,
    /// `false`: the pointer passes through the node and its subtree; unset is `true`.
    HitTestable,
    /// While the node has keyboard focus, Tab and Shift+Tab reach its key handlers instead of moving focus.
    KeepsTab,
}

/// Each [`InputFlag`] unset, false or true, in two bits. The bits live in one
/// `u32`, so at most 16 flags fit. A new flag goes at the end of
/// [`InputFlag`]; the check below must stay true, and if it does not, the
/// type must be widened.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputFlags(u32);

/// Every flag has two bits in [`InputFlags`]: the last flag must fit in 32 bits.
const _: () = assert!((InputFlag::KeepsTab as usize + 1) * 2 <= u32::BITS as usize);

impl InputFlags {
    /// The value of `flag` (`None`: unset).
    pub fn get(self, flag: InputFlag) -> Option<bool> {
        match (self.0 >> (2 * flag as u32)) & 0b11 {
            0 => None,
            1 => Some(false),
            _ => Some(true),
        }
    }

    /// Set `flag` to `value`.
    pub fn set(&mut self, flag: InputFlag, value: Option<bool>) {
        let shift = 2 * flag as u32;
        let bits = match value {
            None => 0,
            Some(false) => 1,
            Some(true) => 2,
        };
        self.0 = (self.0 & !(0b11 << shift)) | (bits << shift);
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the input flag bits, alone and in a mixed pattern.

    use super::{InputFlag, InputFlags};

    /// Every flag, in declaration order.
    const ALL: [InputFlag; 9] = [
        InputFlag::Disabled,
        InputFlag::Hidden,
        InputFlag::Focusable,
        InputFlag::FocusableWhenDisabled,
        InputFlag::ListenPointer,
        InputFlag::ListenWheel,
        InputFlag::ListenFile,
        InputFlag::HitTestable,
        InputFlag::KeepsTab,
    ];

    /// The value for the `i`th flag in the mixed pattern.
    fn pattern(i: usize) -> Option<bool> {
        match i % 3 {
            0 => None,
            1 => Some(false),
            _ => Some(true),
        }
    }

    #[test]
    fn every_flag_sets_and_reads_without_disturbing_the_others() {
        assert_eq!(ALL.len(), InputFlag::KeepsTab as usize + 1);
        for flag in ALL {
            let mut flags = InputFlags::default();
            flags.set(flag, Some(true));
            assert_eq!(flags.get(flag), Some(true), "{flag:?}");
            for other in ALL.into_iter().filter(|other| *other != flag) {
                assert_eq!(flags.get(other), None, "{other:?} after {flag:?}");
            }
        }

        let mut flags = InputFlags::default();
        for (i, flag) in ALL.into_iter().enumerate() {
            flags.set(flag, pattern(i));
        }
        for (i, flag) in ALL.into_iter().enumerate() {
            assert_eq!(flags.get(flag), pattern(i), "{flag:?}");
        }

        flags.set(InputFlag::KeepsTab, Some(false));
        assert_eq!(flags.get(InputFlag::KeepsTab), Some(false));
        for (i, flag) in ALL.into_iter().enumerate().take(8) {
            assert_eq!(flags.get(flag), pattern(i), "{flag:?} after KeepsTab");
        }
    }
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
    /// The node's role: the one its semantics state, else `Role::Text` for a
    /// text node that is not a native editor (its kind's role, which the
    /// platform tree also gives it, `model::semantics::semantic_role`).
    pub fn role(&self) -> Option<kivact_types::Role> {
        self.semantics.as_deref().and_then(|s| s.role).or_else(|| {
            (matches!(self.kind, NodeKind::Text(_)) && self.editing.is_none())
                .then_some(kivact_types::Role::Text)
        })
    }

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
