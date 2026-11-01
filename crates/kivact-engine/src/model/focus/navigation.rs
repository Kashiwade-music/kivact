//! Role: Focus processing: tab stops, Tab order, focus requests and automatic moves.

use crate::model::Model;
use kivact_wire::{FocusTarget, HostEvent, NodeId};

impl Model {
    /// Scroll every scrollable ancestor of `id` so that it is visible
    /// (after the next layout pass).
    pub(crate) fn reveal(&mut self, id: NodeId) {
        self.focus.reveal.push(id);
        self.changes.needs_layout = true;
    }

    /// At a commit: a focus the host moved on its own is revealed, like a
    /// Tab move.
    pub(crate) fn reveal_moved_focus(&mut self) {
        if std::mem::take(&mut self.focus.reveal_focus)
            && let Some(focus) = self.focus.target
        {
            self.reveal(focus.node);
        }
    }

    /// The exact native keyboard focus target.
    pub(crate) fn focused(&self) -> Option<FocusTarget> {
        self.focus.target
    }

    /// Whether a node itself is an enabled, shown tab stop.
    pub(crate) fn is_tab_stop(&self, id: NodeId) -> bool {
        self.tree
            .get(id)
            .is_some_and(|n| n.focusable(self.editors[n.index.0].is_some()))
            && self.tree.shown(id)
    }

    /// Whether the exact target may take focus in the current tree.
    pub(crate) fn is_focus_target(&self, target: FocusTarget) -> bool {
        if !self.tree.shown(target.node) {
            return false;
        }
        let Some(node) = self.tree.get(target.node) else {
            return false;
        };
        match target.region {
            None => node.focusable(self.editors[node.index.0].is_some()),
            Some(id) => {
                !node.disabled()
                    && node
                        .region(id)
                        .is_some_and(|r| r.focusable && !r.disabled && !r.hidden)
            }
        }
    }

    /// All declared targets in node, region, then descendant order.
    pub(in crate::model) fn focus_order(&self) -> Vec<FocusTarget> {
        let mut targets = Vec::new();
        for id in self.tree.preorder() {
            targets.push(FocusTarget {
                node: id,
                region: None,
            });
            let Some(node) = self.tree.get(id) else {
                continue;
            };
            let mut seen = std::collections::HashSet::new();
            let mut regions: Vec<_> = node
                .scenes()
                .iter()
                .flatten()
                .flat_map(|s| &s.regions)
                .collect();
            regions.sort_by_key(|r| r.focus_order);
            for region in regions {
                if seen.insert(region.id) {
                    targets.push(FocusTarget {
                        node: id,
                        region: Some(region.id),
                    });
                }
            }
        }
        targets
    }

    /// The eligible targets in tree and scene declaration order.
    pub(crate) fn tab_stops(&self) -> Vec<FocusTarget> {
        self.focus_order()
            .into_iter()
            .filter(|t| self.is_focus_target(*t) && self.in_scope(t.node))
            .collect()
    }

    /// Move through native tab stops, wrapping within the modal scope.
    pub(crate) fn move_focus(&mut self, backward: bool) -> bool {
        let stops = self.tab_stops();
        if stops.is_empty() {
            return false;
        }
        let cur = self
            .focus
            .target
            .and_then(|f| stops.iter().position(|s| *s == f));
        let next = match (cur, backward) {
            (None, false) => 0,
            (None, true) => stops.len() - 1,
            (Some(i), false) => (i + 1) % stops.len(),
            (Some(i), true) => (i + stops.len() - 1) % stops.len(),
        };
        self.set_focus_visible(Some(stops[next]), true);
        self.focus.reveal.push(stops[next].node);
        self.changes.needs_layout = true;
        true
    }

    /// Validate and focus a node or one of its declared regions.
    pub(crate) fn request_focus(
        &mut self,
        target: FocusTarget,
        visible: bool,
    ) -> Result<(), &'static str> {
        if !self.is_focus_target(target) {
            return Err("target is not focusable");
        }
        if !self.in_scope(target.node) {
            return Err("node is outside the open modal overlay");
        }
        self.set_focus_visible(Some(target), visible);
        self.focus.reveal.push(target.node);
        self.changes.needs_layout = true;
        Ok(())
    }

    /// Whether the focus is on the nearest ancestor of `id` that is a group
    /// or on a node below it.
    pub(crate) fn is_group_focused(&self, id: NodeId) -> bool {
        let Some(focus) = self.focus.target else {
            return false;
        };
        self.nearest_group(id)
            .is_some_and(|g| self.tree.is_in_subtree(focus.node, g))
    }

    /// Change the sole focus target and report its complete identity.
    pub(crate) fn set_focus(&mut self, target: Option<FocusTarget>) {
        self.set_focus_visible(target, self.focus.visible);
    }

    /// Change focus together with the modality used by focus indicators.
    pub(crate) fn set_focus_visible(&mut self, target: Option<FocusTarget>, visible: bool) {
        let left = self.tree.chain(self.focus.target.map(|t| t.node));
        self.change_focus(target, visible, &left);
    }

    /// [`Model::set_focus_visible`] from a focus whose node and its
    /// ancestors, nearest first, were `left` (read before a removal took
    /// them from the tree).
    fn change_focus(&mut self, target: Option<FocusTarget>, visible: bool, left: &[NodeId]) {
        let visible = target.is_some() && visible;
        if self.focus.target == target && self.focus.visible == visible {
            return;
        }
        if self.focus.target != target
            && let Some(old) = self.focus.target.filter(|t| t.region.is_none())
        {
            self.commit_composition(old.node);
        }
        for n in [self.focus.target, target]
            .into_iter()
            .flatten()
            .map(|t| t.node)
        {
            self.prepare_state_motion(n);
            self.repaint(n);
            if self.tree.get(n).and_then(|x| x.states()).is_some_and(|s| {
                (s.focus.is_some() || s.focus_visible.is_some()) && crate::tree::states_place(s)
            }) {
                self.changes.needs_layout = true;
                self.changes.moved = true;
            }
        }
        if self.focus.target != target {
            // The groups the focus enters or leaves restyle the nodes below.
            let new = self.tree.chain(target.map(|t| t.node));
            let touched: Vec<NodeId> = left
                .iter()
                .filter(|x| !new.contains(x))
                .chain(new.iter().filter(|x| !left.contains(x)))
                .copied()
                .collect();
            for id in self.group_dependents(&touched, |s| s.group_focus.is_some()) {
                self.prepare_state_motion(id);
                self.restyled(id);
            }
        }
        self.changes.a11y_focus |= self.focus.target != target;
        self.focus.target = target;
        self.focus.visible = visible;
        self.push(HostEvent::FocusChanged {
            node: target.map(|t| t.node),
            region: target.and_then(|t| t.region),
            visible,
        });
    }

    /// Whether a press outside focusable targets should blur the focus.
    pub(crate) fn press_blurs(&self, hit: Option<NodeId>) -> bool {
        let Some(focus) = self.focus.target else {
            return false;
        };
        let Some(hit) = hit else {
            return self.modal().is_none();
        };
        !self.tree.overlays.iter().any(|o| {
            self.tree.is_in_subtree(hit, *o)
                && self.tree.shown(*o)
                && self
                    .tree
                    .get(*o)
                    .and_then(|n| n.overlay())
                    .and_then(|p| p.anchor)
                    .is_some_and(|a| self.tree.is_in_subtree(focus.node, a))
        })
    }

    /// Reveal focus moved automatically by a tree mutation.
    pub(crate) fn move_focus_to(&mut self, target: Option<FocusTarget>) {
        self.set_focus(target);
        if target.is_some() {
            self.focus.reveal_focus = true;
        }
    }

    /// The focus is in `removed`, which goes now: the nodes styled by the
    /// groups it is in move from what they show; its node and ancestors,
    /// nearest first, for [`Model::move_focus_off_removed`] (`None`: the
    /// focus is elsewhere).
    pub(crate) fn focus_in_removal(&mut self, removed: NodeId) -> Option<Vec<NodeId>> {
        let focus = self
            .focused()
            .filter(|f| self.tree.is_in_subtree(f.node, removed))?;
        let left = self.tree.chain(Some(focus.node));
        for x in self.group_dependents(&left, |s| s.group_focus.is_some()) {
            self.prepare_state_motion(x);
        }
        Some(left)
    }

    /// [`Model::move_focus_to`] from a focus a removal took, whose node
    /// and its ancestors, nearest first, were `left`: the groups it was in
    /// restyle the nodes below them.
    pub(crate) fn move_focus_off_removed(&mut self, left: &[NodeId], target: Option<FocusTarget>) {
        self.change_focus(target, self.focus.visible, left);
        if target.is_some() {
            self.focus.reveal_focus = true;
        }
    }

    /// The next target outside a removed subtree and within the remaining modal.
    pub(crate) fn next_tab_stop_outside(&self, removed: NodeId) -> Option<FocusTarget> {
        let order = self.focus_order();
        let start = order.iter().position(|t| t.node == removed)?;
        let modal = self.top_modal(Some(removed));
        order[start..]
            .iter()
            .chain(order[..start].iter())
            .copied()
            .find(|t| {
                !self.tree.is_in_subtree(t.node, removed)
                    && self.is_focus_target(*t)
                    && modal.is_none_or(|m| self.tree.is_in_subtree(t.node, m))
            })
    }

    /// Revalidate an exact target after props or scenes change.
    pub(crate) fn revalidate_focus(&mut self) {
        let Some(focus) = self.focus.target else {
            return;
        };
        if self.is_focus_target(focus) {
            return;
        }
        let order = self.focus_order();
        let start = order
            .iter()
            .position(|t| *t == focus)
            .or_else(|| order.iter().position(|t| t.node == focus.node))
            .unwrap_or(0);
        let next = order[start..]
            .iter()
            .chain(order[..start].iter())
            .copied()
            .find(|t| self.is_focus_target(*t) && self.in_scope(t.node));
        self.move_focus_to(next);
    }
}
