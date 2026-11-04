//! Role: The edit group's state: the text input index and the IME's last text.

use std::collections::BTreeSet;

use kivact_editing::TextSession;
use kivact_types::{NodeId, Rect};

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
    pub(super) transform: Option<kivact_types::Transform>,
    pub(super) password: bool,
    pub(super) read_only: bool,
}

/// What the IME text of a node with an IME session is made from
/// (`Model::session_ime_changed`): the node, its session, its box and its
/// transform.
#[derive(Clone, PartialEq)]
pub(super) struct SessionSource {
    pub(super) node: NodeId,
    pub(super) session: crate::contract::ImeSessionProps,
    pub(super) area: Option<Rect>,
    pub(super) transform: Option<kivact_types::Transform>,
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
    /// starts in the input's value (bytes), `None` when the input gave no text.
    Input {
        source: InputSource,
        start: Option<usize>,
    },
    /// The text of the focused node with an IME session.
    Session(SessionSource),
}

/// The text inputs with an editing state, and what the IME was last given.
#[derive(Default)]
pub(crate) struct Edits {
    pub(super) inputs: BTreeSet<NodeId>,
    pub(super) given: ImeGiven,
}

/// What `Model::ime_changed` asks of the window.
pub(crate) enum ImeUpdate {
    /// The window's IME text is still right.
    Unchanged,
    /// Give the window this text (`None`: no text input has the focus).
    Set(Option<arvane::ShellImeText>),
}

impl Edits {
    /// The text inputs with an editing state, in id order.
    pub(crate) fn inputs(&self) -> Vec<NodeId> {
        self.inputs.iter().copied().collect()
    }
}
