//! Role: The edit group's state: the text input index and the IME's last text.

use kivact_editing::TextSession;
use kivact_types::{NodeId, Rect};
use std::collections::BTreeSet;

/// What a text input's IME text is made from (`Model::ime_changed`): the input,
/// its text's revision and its session, its text box, its text scroll, its
/// transform and its password and read-only flags.
#[derive(Clone, PartialEq)]
pub(super) struct InputSource {
    pub(super) node: NodeId,
    pub(super) revision: u64,
    pub(super) session: TextSession,
    pub(super) area: Rect,
    pub(super) scroll: f64,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    pub(super) transform: Option<kivact_types::Projective>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    pub(super) password: bool,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    pub(super) read_only: bool,
}

/// What the IME text of a node with an IME session is made from
/// (`Model::session_ime_changed`): the node, its session, its box and its
/// transform.
#[derive(Clone, PartialEq)]
pub(super) struct SessionSource {
    pub(super) node: NodeId,
    pub(super) session: crate::contract::ImeSessionProps,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    pub(super) area: Option<Rect>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    pub(super) transform: Option<kivact_types::Projective>,
}

/// What the window's IME was last given.
#[derive(Default)]
pub(super) enum ImeGiven {
    /// Nothing yet: the next update gives whatever has the focus.
    #[default]
    Unset,
    /// No text: no text input or session node has the focus.
    Nothing,
    /// The text of the focused text input made of `source`; `start` is where it
    /// starts in the input's value (bytes), or that the input gave no text.
    Input {
        source: InputSource,
        start: InputStart,
    },
    /// The text of the focused node with an IME session.
    Session(SessionSource),
}

/// The text inputs with an editing state, and what the IME was last given.
#[derive(Default)]
pub(crate) struct Edits {
    /// The live nodes with an editing state, in id order (`Model::set_edit`).
    pub(super) inputs: BTreeSet<NodeId>,
    pub(super) given: ImeGiven,
    pub(crate) target: ImeTarget,
}

/// What `Model::ime_changed` asks of the window.
pub(crate) enum ImeUpdate {
    /// The window's IME text is still right.
    Unchanged,
    /// Give the window this text (`None`: no text input has the focus).
    Set(Option<arvane::ShellImeText>),
}

#[derive(Clone, Copy)]
pub(super) enum InputStart {
    At(usize),
    NoText,
}

#[derive(Clone, Copy, Default)]
pub(crate) enum ImeTarget {
    #[default]
    None,
    Native(EditorSlot),
    Session(NodeId),
}
#[derive(Clone, Copy)]
pub(crate) struct EditorSlot(pub(crate) crate::tree::node::NodeIndex);
