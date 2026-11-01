//! Role: Modal scope queries, input admission and focus confinement.

use crate::model::Model;
use crate::tree::node::HostNode;
use kivact_wire::{A11yAction, FocusTarget, HostEvent, NodeId, PointerId, PointerPhase};

impl Model {
    /// The topmost shown modal overlay, if one is shown.
    pub(crate) fn modal(&self) -> Option<NodeId> {
        self.top_modal(None)
    }

    /// The topmost shown modal overlay outside the subtree of `removed`
    /// (the modal that stays once `removed` is gone).
    pub(crate) fn top_modal(&self, removed: Option<NodeId>) -> Option<NodeId> {
        let shown: Vec<NodeId> = self
            .tree
            .overlays
            .iter()
            .copied()
            .filter(|id| {
                self.tree.get(*id).is_some_and(HostNode::is_modal)
                    && self.tree.shown(*id)
                    && removed.is_none_or(|r| !self.tree.is_in_subtree(*id, r))
            })
            .collect();
        match shown.as_slice() {
            [] => None,
            [only] => Some(*only),
            // Several: the last one in tree order is on top.
            _ => self
                .tree
                .preorder()
                .into_iter()
                .rev()
                .find(|id| shown.contains(id)),
        }
    }

    /// The active modal when `id` lies in it: listener searches and scroll
    /// chains that start at `id` stop there.
    pub(crate) fn modal_boundary(&self, id: NodeId) -> Option<NodeId> {
        self.modal().filter(|m| self.tree.is_in_subtree(id, *m))
    }

    /// Whether input may reach `id` (no modal shown, or inside the active
    /// one).
    pub(crate) fn in_scope(&self, id: NodeId) -> bool {
        self.modal().is_none_or(|m| self.tree.is_in_subtree(id, m))
    }

    /// The topmost shown overlay that offers `Dismiss`, inside the active
    /// modal's scope when one is shown: where an Escape no key handler took
    /// goes. The last, in tree order, of the overlays
    /// [`Model::dismiss_outside`] would dismiss.
    pub(crate) fn dismiss_target(&self) -> Option<NodeId> {
        let mut candidates = self.dismissible_overlays();
        match candidates.len() {
            0 => None,
            1 => candidates.pop(),
            _ => {
                let order = self.tree.preorder();
                candidates
                    .into_iter()
                    .max_by_key(|c| order.iter().position(|x| x == c))
            }
        }
    }

    /// Whether a focus request for `target` may land now: the node is shown
    /// (attached, no hidden or leaving ancestor), focusable by its own props
    /// (a text input by default) and not disabled, the region of `target`
    /// exists in a draw layer and is focusable, enabled and visible, and the
    /// node lies in the active modal. The one rule a focus restore is
    /// checked by before it is sent.
    pub(crate) fn can_take_focus(&self, target: FocusTarget) -> bool {
        self.is_focus_target(target) && self.in_scope(target.node)
    }

    /// Whether the event may be dispatched (see the module's invariants).
    /// Records the drag a pointer `Down` admits and ends the one a release
    /// or a `Cancel` finishes, so it is asked once per event, in event
    /// order.
    pub(crate) fn admit_input(&self, event: &HostEvent) -> bool {
        match event {
            HostEvent::Pointer { node, event, .. } => {
                self.admit_pointer(*node, event.id, event.phase, event.button, event.buttons)
            }
            HostEvent::Wheel { node, .. }
            | HostEvent::File { node, .. }
            | HostEvent::Activate { node, .. }
            | HostEvent::TextChanged { node, .. }
            | HostEvent::SelectionChanged { node, .. }
            | HostEvent::Ime { node, .. }
            | HostEvent::A11y { node, .. }
            | HostEvent::OverlayDismiss { node } => self.in_scope(*node),
            _ => true,
        }
    }

    /// Admit or drop a pointer event of `pointer` and `phase` aimed at
    /// `node`, updating that pointer's admitted drag. `button` is the
    /// changed button of a `Down` or `Up` and `buttons` the buttons held
    /// after the event: a `Move` with none held is a hover, which a modal
    /// judges like a new press.
    fn admit_pointer(
        &self,
        node: NodeId,
        pointer: PointerId,
        phase: PointerPhase,
        button: Option<kivact_wire::MouseButton>,
        buttons: kivact_wire::PointerButtons,
    ) -> bool {
        let admission = &self.admission;
        admission.forget_missing(|n| self.tree.live(n));
        let continuing = admission.drag(pointer) == Some(node);
        let starts = match phase {
            PointerPhase::Down | PointerPhase::Enter => true,
            PointerPhase::Move => buttons.is_empty(),
            PointerPhase::Up | PointerPhase::Cancel | PointerPhase::Leave => false,
        };
        if starts && !(continuing && phase == PointerPhase::Down) && !self.in_scope(node) {
            return false;
        }
        if !starts && phase != PointerPhase::Leave && !continuing && !self.in_scope(node) {
            // A drag outside the modal that did not start before it.
            // Rejected input for another node must not end the admitted drag.
            return false;
        }
        match phase {
            PointerPhase::Down => {
                if let Some(button) = button {
                    admission.press(pointer, node, button);
                    // A new primary press ends the pointer's earlier gesture
                    // whose release never came.
                    if button == kivact_wire::MouseButton::Primary {
                        admission.forget_adoption(pointer);
                    }
                }
            }
            PointerPhase::Up => admission.release(pointer, node, button),
            PointerPhase::Cancel => {
                admission.cancel(pointer, node);
                admission.forget_adoption(pointer);
            }
            _ => {}
        }
        true
    }

    /// The shown overlays that offer `Dismiss` and lie in the active
    /// modal's scope (in overlay-list order, not tree order).
    pub(in crate::model) fn dismissible_overlays(&self) -> Vec<NodeId> {
        let modal = self.modal();
        let tree = &self.tree;
        tree.overlays
            .iter()
            .copied()
            .filter(|o| {
                tree.get(*o)
                    .is_some_and(|n| n.actions().contains(&A11yAction::Dismiss))
                    && tree.shown(*o)
                    && modal.is_none_or(|m| tree.is_in_subtree(*o, m))
            })
            .collect()
    }

    /// Nearest ancestor-or-self of `id` matching `f`, not looking past the
    /// active modal when `id` lies in it.
    pub(crate) fn ancestor_where(
        &self,
        mut id: NodeId,
        f: impl Fn(&HostNode) -> bool,
    ) -> Option<NodeId> {
        let stop = self.modal_boundary(id);
        loop {
            let n = self.tree.get(id)?;
            if f(n) {
                return Some(id);
            }
            if Some(id) == stop {
                return None;
            }
            id = self.tree.parent_id(n)?;
        }
    }

    /// Move a focus left outside the active modal into it (at a commit).
    pub(crate) fn confine_focus(&mut self) {
        let (Some(modal), Some(focus)) = (self.modal(), self.focused()) else {
            return;
        };
        if self.tree.is_in_subtree(focus.node, modal) {
            return;
        }
        let order = self.tree.preorder();
        let start = order.iter().position(|x| *x == focus.node).unwrap_or(0);
        let ranks: std::collections::HashMap<_, _> = order[start..]
            .iter()
            .chain(order[..start].iter())
            .enumerate()
            .map(|(index, id)| (*id, index))
            .collect();
        let next = self
            .tab_stops()
            .into_iter()
            .min_by_key(|target| ranks.get(&target.node).copied().unwrap_or(usize::MAX));
        self.move_focus_to(next);
    }
}
