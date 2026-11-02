//! Role: Outside-press dismissal of overlays.
//!
//! Invariants:
//! - On every pointer press, each shown overlay whose actions offer
//!   `Dismiss` gets `A11y` `Dismiss(OutsidePress)` when the pressed node lies
//!   outside the overlay's subtree and outside its anchor's subtree. A
//!   press that hits no node (an empty spot, or the backdrop of a modal) is
//!   outside every overlay.
//! - While a modal overlay is shown, only overlays in its scope (the modal
//!   and the overlays nested in it) are candidates
//!   ([`Model::dismissible_overlays`], which an Escape's
//!   [`Model::dismiss_target`] shares).
//! - Dismissals go innermost first (reverse tree order), before the press
//!   is delivered. Only the overlay list is scanned; the tree is walked
//!   only when several overlays are dismissed at once.

use kivact_wire::{A11yRequest, DismissReason, HostEvent, NodeId};

use crate::model::Model;

impl Model {
    /// Send `Dismiss` to every overlay a press on `hit` lands outside of.
    pub(crate) fn dismiss_outside(&mut self, hit: Option<NodeId>) {
        if self.tree.overlays.is_empty() {
            return;
        }
        let tree = &self.tree;
        let mut targets = self.dismissible_overlays();
        targets.retain(|o| {
            let anchor = tree
                .get(*o)
                .and_then(|n| n.overlay())
                .and_then(|p| p.anchor());
            hit.is_none_or(|h| {
                !tree.is_in_subtree(h, *o) && anchor.is_none_or(|a| !tree.is_in_subtree(h, a))
            })
        });
        if targets.len() > 1 {
            let order = tree.preorder();
            targets.sort_by_key(|t| std::cmp::Reverse(order.iter().position(|x| x == t)));
        }
        for node in targets {
            self.push(HostEvent::A11y {
                node,
                region: None,
                request: A11yRequest::Dismiss(DismissReason::OutsidePress),
            });
        }
    }
}
