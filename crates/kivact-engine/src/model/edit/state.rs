//! Role: The edit group's state: the text input index and the IME's last text.

use std::collections::BTreeSet;

use kivact_editing::TextSession;
use kivact_wire::{NodeId, Rect};

/// What the IME text of a text input is made from (`Model::ime_changed`):
/// the input, its text's revision and its session, its text box, its text
/// scroll and its transforms.
pub(super) type ImeSource = (
    NodeId,
    (u64, TextSession),
    Rect,
    f32,
    Option<kivact_types::Transform>,
    (bool, bool),
);

/// What the IME text of a node with an IME session is made from
/// (`Model::session_ime_changed`): the node, its session, its box and its
/// transforms.
pub(super) type SessionSource = (
    NodeId,
    kivact_wire::ImeSession,
    Option<Rect>,
    Option<kivact_types::Transform>,
);

/// The text inputs with an editing state, and the IME's last text.
#[derive(Default)]
pub(crate) struct Edits {
    pub(super) inputs: BTreeSet<NodeId>,
    pub(super) ime_from: Option<Option<ImeSource>>,
    /// Where the text the IME was last given starts in the value (bytes).
    pub(super) ime_start: usize,
    /// The session node whose text the IME was last given.
    pub(super) session_from: Option<SessionSource>,
}

impl Edits {
    /// The text inputs with an editing state, in id order.
    pub(crate) fn inputs(&self) -> Vec<NodeId> {
        self.inputs.iter().copied().collect()
    }
}
