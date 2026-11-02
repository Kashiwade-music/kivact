//! Role: The focus group's state: the focus target with its modality and pending reveal.

use crate::tree::NodeTree;
use kivact_wire::{FocusTarget, NodeId};

/// The keyboard focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Focused {
    pub(crate) target: FocusTarget,
    /// Its indicator shows (`:focus-visible`).
    pub(crate) visible: bool,
    /// The host moved it on its own: it is scrolled into view at the commit.
    pub(super) reveal_at_commit: bool,
}

/// The focus group: the keyboard focus, and the nodes to scroll into view
/// after the next layout pass.
#[derive(Default)]
pub(crate) struct Focus {
    pub(super) focused: Option<Focused>,
    pub(super) reveal: Vec<NodeId>,
}

impl Focus {
    /// The nodes to scroll into view, taken by the layout pass.
    pub(crate) fn take_reveal(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.reveal)
    }

    /// The exact keyboard focus target.
    pub(crate) fn target(&self) -> Option<FocusTarget> {
        self.focused.map(|f| f.target)
    }

    /// Whether the focus is shown (`:focus-visible`).
    pub(crate) fn visible(&self) -> bool {
        self.focused.is_some_and(|f| f.visible)
    }

    /// Drop removed and leaving nodes.
    pub(crate) fn forget_missing(&mut self, tree: &NodeTree) {
        self.reveal.retain(|r| tree.live(*r));
    }
}
