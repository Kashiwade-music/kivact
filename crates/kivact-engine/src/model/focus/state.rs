//! Role: The focus group's state: the focus target, its modality and pending reveals.

use crate::tree::NodeTree;
use kivact_wire::{FocusTarget, NodeId};

/// The focus group: the keyboard focus target, whether its indicator
/// shows, and the nodes to scroll into view after the next layout pass.
#[derive(Default)]
pub(crate) struct Focus {
    pub(super) target: Option<FocusTarget>,
    pub(super) visible: bool,
    pub(super) reveal: Vec<NodeId>,
    /// The host moved the focus on its own: reveal it at the commit.
    pub(super) reveal_focus: bool,
}

impl Focus {
    /// The nodes to scroll into view, taken by the layout pass.
    pub(crate) fn take_reveal(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.reveal)
    }

    /// Whether the focus is shown (`:focus-visible`).
    pub(crate) fn visible(&self) -> bool {
        self.visible
    }

    /// Drop removed and leaving nodes.
    pub(crate) fn forget_missing(&mut self, tree: &NodeTree) {
        self.reveal.retain(|r| tree.live(*r));
    }
}
