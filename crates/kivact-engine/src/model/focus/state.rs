//! Role: Keyboard focus, held-key targets, modality and pending focus reveal.

use crate::tree::NodeTree;
use kivact_types::{FocusTarget, KeyCode, NodeId};
use std::collections::HashMap;

/// Targets that received a physical key down, until its release.
#[derive(Default)]
pub(crate) struct KeyDownTargets(pub(crate) HashMap<KeyCode, FocusTarget>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FocusVisibility {
    Hidden,
    Visible,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Reveal {
    #[default]
    No,
    AtCommit,
}

/// The keyboard focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Focused {
    pub(crate) target: FocusTarget,
    /// Its indicator shows (`:focus-visible`).
    pub(crate) visibility: FocusVisibility,
    /// The host moved it on its own: it is scrolled into view at the commit.
    pub(super) reveal: Reveal,
}

/// The focus group: the keyboard focus, and the nodes to scroll into view
/// after the next layout pass.
#[derive(Default)]
pub(crate) struct Focus {
    pub(super) focused: Option<Focused>,
    pub(super) reveal: Vec<NodeId>,
    pub(super) roving: HashMap<NodeId, FocusTarget>,
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
        self.focused
            .is_some_and(|f| f.visibility == FocusVisibility::Visible)
    }

    /// Drop removed and leaving nodes.
    pub(crate) fn forget_missing(&mut self, tree: &NodeTree) {
        self.reveal.retain(|r| tree.live(*r));
        self.roving
            .retain(|root, target| tree.live(*root) && tree.live(target.node));
    }
}
