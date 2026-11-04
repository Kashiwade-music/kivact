//! Role: Applies client writes and records changed derived consumers.
//!
//! The writes kivact makes during a turn: each one is applied to the
//! node tree at once, recording layout invalidations for Model::sync, or
//! carried out when it is the
//! model's (focus, scroll, layout observation); a commit then checks the
//! focus once and records a private boundary. The writes only a host can carry out
//! (platform effects, frames, the window) are the caller's
//! (`host::write`).
//!
//! Invariants:
//! - Writes are trusted; local invariants use debug assertions.
//!   Writes naming unknown nodes do nothing.
//! - The hover is checked at the commit too: a hit that got hidden or
//!   left the shown modal's scope is hovered no more.
//! - The focus is checked once, at the commit, never on a half-written
//!   tree: a focused node that stopped being a tab stop (disabled, hidden
//!   by itself or an ancestor) gives the focus to the next tab stop, then
//!   a focus outside a shown modal moves into it. The one exception is
//!   the removal of the focused node, which moves the focus at once (to
//!   the next tab stop outside the removed subtree).
//! - A prop that moves boxes asks for a layout pass: layout props and
//!   `Hidden` (computed again), measured content (measured again), an
//!   overlay's `Overlay` and a new transform or z-index (placed again,
//!   reported and redrawn: `moved`).
//! - A private commit boundary follows every event its writes produced;
//!   the host delivers it only after layout and without model borrows.

use std::sync::Arc;

use crate::contract::{HostEvent, NodeKind, NodeProperties, NodeType, SceneLayer};
use crate::mutation::Mutation;
use crate::tree::node::{LeavingRole, NodePresence, Notify};
use kivact_types::{DrawList, ElementTransform, NodeId, Point, RequestId, Revision};

use crate::model::Model;
use crate::model::changes::Dirty;
use crate::model::text::text_reach_of;
use crate::tree::node::layer_index;

impl Model {
    /// End the writes of one revision (the one after the last applied):
    /// check the focus once and record its private post-layout boundary.
    pub(crate) fn commit_revision(&mut self) -> Revision {
        self.tree.pack_child_orders();
        self.start_entrances();
        self.revalidate_focus();
        self.confine_focus();
        self.revalidate_hover();
        self.reveal_moved_focus();
        let revision = Revision(self.applied().0 + 1);
        self.events.commit(revision);
        revision
    }

    /// Move an attached node before a sibling (append if `None`).
    pub(crate) fn move_before(&mut self, id: NodeId, before: Option<NodeId>) {
        self.changes.a11y_structure = true;
        self.changes.restack = true;
        if let Some(parent) = self.tree.detach(id) {
            self.changes.mark(parent, Dirty::LAYOUT_CHAIN);
            self.changes.outdate_viewport();
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::STYLE);
            self.insert(parent, id, before, false);
        }
    }

    /// Set the scene of one draw layer of `id`.
    pub(crate) fn set_scene(&mut self, id: NodeId, layer: SceneLayer, scene: Arc<DrawList>) {
        self.changes.restack = true;
        let old_order = if self.focused().is_some_and(|focus| focus.node == id) {
            self.focus_order()
        } else {
            Vec::new()
        };
        if let Some(n) = self.tree.get_mut(id) {
            n.scenes[layer_index(layer)] = Some(scene);
            self.changes
                .mark(id, Dirty::WRITTEN.union(Dirty::A11Y_METADATA));
        }
        if let Some(focus) = self
            .focused()
            .filter(|f| f.node == id && !self.is_focus_target(*f))
        {
            let start = old_order.iter().position(|t| *t == focus).unwrap_or(0);
            let next = old_order[start..]
                .iter()
                .chain(old_order[..start].iter())
                .copied()
                .find(|t| self.is_focus_target(*t) && self.in_scope(t.node))
                .or_else(|| self.tab_stops().into_iter().next());
            self.move_focus_to(next);
        }
    }

    /// Focus `id` on kivact's request and report the result.
    pub(crate) fn focus_request(
        &mut self,
        id: kivact_types::FocusTarget,
        request: RequestId,
        visible: bool,
    ) {
        let result = self.request_focus(id, visible);
        self.push(HostEvent::FocusResult {
            request,
            result: result.map_err(String::from),
        });
    }

    /// Scroll the container `id` to `offset`.
    pub(crate) fn scroll_to(&mut self, id: NodeId, offset: Point) {
        if let Some(n) = self.tree.get_mut(id) {
            // Like a wheel or a scroll by a delta, a new offset moves the
            // content (the paint list redraws positions on `moved`).
            if n.scroll != offset {
                self.changes.moved();
                self.changes.mark(id, Dirty::SCROLLED);
            }
            n.scroll = offset;
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
        }
    }

    /// Report `LayoutChanged` for `id` while `on`; `false` when it
    /// already did.
    pub(crate) fn observe_layout(&mut self, id: NodeId, on: bool) -> bool {
        let changed = self.tree.set_observed(id, on);
        if changed {
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
        }
        changed
    }

    /// Create a detached node with its static props.
    pub(crate) fn create(&mut self, id: NodeId, kind: NodeKind) {
        self.changes.a11y_structure = true;
        self.create_slot(id, kind);
        self.arrive(id);
        let edit = self
            .node(id)
            .and_then(|n| n.input_data())
            .map(|data| kivact_editing::EditState::new(data.value.as_deref().unwrap_or("")));
        if edit.is_some() {
            let previous = self.set_edit(id, edit);
            debug_assert!(previous.is_none());
        }
        self.changes
            .mark(id, Dirty::LAYOUT_STYLE.union(Dirty::WRITTEN));
        self.changes
            .layout
            .insert(crate::model::changes::LayoutReasons::STYLE);
        self.changes
            .layout
            .insert(crate::model::changes::LayoutReasons::PASS);
    }

    /// Store one typed property of `id` when it differs and record what it
    /// outdates. `field` finds the property (`None`: the node's kind has
    /// none).
    pub(crate) fn set_prop<T: PartialEq>(
        &mut self,
        id: NodeId,
        value: Option<T>,
        field: impl Fn(&mut NodeProperties) -> Option<&mut Option<T>>,
        mut mutation: Mutation,
    ) -> bool {
        let animate_text = mutation.text && self.tree.attached(id);
        let Some(node) = self.tree.get_mut(id) else {
            return false;
        };
        let Some(old) = field(&mut node.properties) else {
            return false;
        };
        if *old == value {
            return false;
        }
        if mutation.read_only {
            // Commit preedit while the previous read-only policy still applies.
            self.edit_input(id, kivact_editing::EditState::unmark);
            let node = self
                .tree
                .get_mut(id)
                .expect("invariant: input edit keeps node");
            *field(&mut node.properties).expect("invariant: input edit keeps its kind") = value;
        } else if mutation.paint || mutation.states {
            let old_place = node
                .paint()
                .map_or((ElementTransform::NONE, 0), |p| (p.transform, p.z_index));
            let old_states = node.states().is_some_and(crate::tree::states_place);
            // A node with state styles is drawn with the deltas of its
            // states: what its list steps are made of is read from the paint
            // they give (the others from the base paint, `Model::derive`).
            let styled = mutation.states || node.states().is_some();
            let before = styled.then(|| self.paint_stacking(id));
            self.prepare_state_motion(id);
            let node = self
                .tree
                .get_mut(id)
                .expect("invariant: state preparation keeps node");
            *field(&mut node.properties).expect("invariant: setter field keeps its kind") = value;
            mutation.placed |= if mutation.paint {
                old_place
                    != node
                        .paint()
                        .map_or((ElementTransform::NONE, 0), |p| (p.transform, p.z_index))
            } else {
                old_states || node.states().is_some_and(crate::tree::states_place)
            };
            mutation.restack |= before.is_some_and(|before| before != self.paint_stacking(id));
        } else if animate_text {
            self.prepare_state_motion(id);
            let node = self
                .tree
                .get_mut(id)
                .expect("invariant: text transition preparation keeps node");
            *field(&mut node.properties).expect("invariant: setter field keeps its kind") = value;
        } else {
            *old = value;
        }
        self.changed(id, mutation);
        true
    }

    /// [`Model::set_prop`] for a boxed property: the borrowed value is
    /// compared before its box is allocated.
    pub(crate) fn set_boxed_prop<T: PartialEq>(
        &mut self,
        id: NodeId,
        value: Option<T>,
        field: impl Fn(&mut NodeProperties) -> Option<&mut Option<Box<T>>>,
        mutation: Mutation,
    ) -> bool {
        let unchanged = self
            .tree
            .get_mut(id)
            .and_then(|node| field(&mut node.properties))
            .is_some_and(|old| old.as_deref() == value.as_ref());
        if unchanged {
            return false;
        }
        self.set_prop(id, value.map(Box::new), field, mutation)
    }

    /// Record the consumers affected by one typed property change.
    pub(super) fn changed(&mut self, id: NodeId, mutation: Mutation) {
        let local = mutation.local_text
            && !(mutation.style
                || mutation.text
                || mutation.placed
                || mutation.subtree
                || mutation.restack
                || mutation.paint
                || mutation.states
                || mutation.read_only);
        self.changes.mark(
            id,
            if local {
                Dirty::A11Y_LOCAL_TEXT
            } else {
                Dirty::A11Y_METADATA
            },
        );
        if mutation.style || mutation.text || mutation.placed || mutation.subtree {
            self.changes.a11y_subtrees.insert(id);
        }
        self.changes.restack |= mutation.restack;
        let Mutation {
            style,
            measure,
            text,
            placed,
            ..
        } = mutation;
        let Some(index) = self.tree.nodes.index_of(id) else {
            return;
        };
        let parent = self.tree.nodes.links(index).parent;
        let old_place = self.tree.update_place(index);
        let n = self.tree.nodes.at(index);
        // Where it lists among its siblings and whether it moves: only a
        // change of these (and of its paint's stacking facts, `derive`)
        // builds the list again.
        self.changes.restack |= n.place != old_place;
        let stacks = parent.filter(|_| n.place.has_z());
        if style {
            let place = n.place;
            let was = (old_place.sticky, old_place.fixed);
            let parent = parent.map(|index| self.tree.nodes.id_at(index));
            self.changes.mark(id, Dirty::LAYOUT_STYLE);
            if place.fixed != was.1 {
                self.changes
                    .layout
                    .insert(crate::model::changes::LayoutReasons::STYLE);
            }
            if let Some(parent) = parent {
                if place.fixed != was.1 {
                    // It moves between its parent's layout and the window's.
                    self.changes.mark(parent, Dirty::LAYOUT_CHAIN);
                    self.changes.outdate_viewport();
                }
                if (place.sticky, place.fixed) != was {
                    // Its parent draws it differently (in flow, stuck, or as a
                    // layer of its own).
                    self.changes.regroup(parent);
                }
            }
            if !place.sticky
                && let Some(n) = self.tree.get_mut(id)
            {
                n.shift = Point::default();
            }
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
        }
        if measure {
            self.changes.mark(id, Dirty::MEASURE);
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
        }
        if placed {
            // A new anchor or placement moves the overlay's subtree, a new
            // transform the node's (no layout computation: origins and
            // transforms are placed live), so the geometry reports run and
            // positions are redrawn.
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
            self.changes.moved();
        }
        if text {
            // An inherited text style: every text below measures again, and
            // every image below draws again (a monochrome one is painted in
            // the text color).
            // The resolved style of every node below is dropped here, in
            // the write: a state-motion capture of this revision reads the
            // new one (`model::text`). A text that now paints farther out
            // (its size or shadows) may show where it was culled, so the
            // list is built again; a color alone does not.
            let mut stack = vec![id];
            while let Some(x) = stack.pop() {
                let Some(n) = self.tree.get(x) else { continue };
                stack.extend(self.tree.children_ids(n));
                let (slot, kind) = (n.index.0, n.kind());
                let drawn = kind == NodeType::Text
                    || self.editors[slot]
                        .as_deref()
                        .map(|editor| editor.editor.state())
                        .is_some();
                let before = self.measure.computed_text(slot);
                self.measure.forget_computed_text(slot);
                if drawn {
                    self.changes.mark(x, Dirty::MEASURE.union(Dirty::REDRAW));
                    let now = text_reach_of(&self.text_props(x));
                    self.changes.restack |= before.is_none_or(|b| text_reach_of(&b) != now);
                } else if kind == NodeType::Image {
                    self.changes.mark(x, Dirty::REDRAW);
                }
            }
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
        }
        if let Some(parent) = stacks {
            self.tree.stacks(self.tree.nodes.id_at(parent));
        }
        self.changes.mark(id, Dirty::WRITTEN);
    }

    /// Attach a detached node under `parent`, before `before` (append if
    /// `None`).
    pub(crate) fn insert(
        &mut self,
        parent: NodeId,
        id: NodeId,
        before: Option<NodeId>,
        inherit: bool,
    ) {
        // New subtrees have no layout cache. A detached tree surviving sync may
        // have been measured as an overlay or fixed layer before attachment.
        let inherited =
            (inherit && self.computed_layout(id).is_some()).then(|| self.text_props(id));
        self.changes.a11y_structure = true;
        self.changes.a11y_subtrees.insert(id);
        self.changes.restack = true;
        if self.tree.insert(parent, id, before).is_none() {
            return;
        }
        // Its ancestors changed: so did the text style it inherits.
        self.forget_text_below(id);
        self.changes.mark(parent, Dirty::LAYOUT_CHAIN);
        self.changes.outdate_viewport();
        self.changes
            .layout
            .insert(crate::model::changes::LayoutReasons::STYLE);
        if inherited.is_some_and(|old| old != self.text_props(id)) {
            let mut stack = vec![id];
            while let Some(child) = stack.pop() {
                let Some(node) = self.tree.get(child) else {
                    continue;
                };
                stack.extend(self.tree.children_ids(node));
                if (node.kind() == NodeType::Text
                    || self.editors[node.index.0]
                        .as_deref()
                        .map(|editor| editor.editor.state())
                        .is_some())
                    || node.measure().is_some()
                {
                    self.changes.mark(child, Dirty::MEASURE);
                }
                self.changes.mark(child, Dirty::REDRAW);
            }
        }
        self.changes.regroup(parent);
        self.changes
            .layout
            .insert(crate::model::changes::LayoutReasons::PASS);
    }

    /// Remove `id` and its whole subtree: at once, or once the exits in it
    /// end (`model::presence`).
    pub(crate) fn remove(&mut self, id: NodeId, pop_layout: bool, notify: bool) {
        let notify = if notify {
            Notify::OnComplete
        } else {
            Notify::Silent
        };
        if !self.begin_exit(id, pop_layout, notify) {
            self.transition_presence(id, NodePresence::Leaving(LeavingRole::Root { notify }));
            self.remove_now(id);
        }
    }

    /// Remove and destroy `id` and its whole subtree now.
    pub(crate) fn remove_now(&mut self, id: NodeId) {
        self.changes.a11y_structure = true;
        self.changes.restack = true;
        let focus_lost = self.focus_in_removal(id);
        let next = if focus_lost.is_some() {
            self.next_tab_stop_outside(id)
        } else {
            None
        };
        let parent = self.tree.get(id).and_then(|n| self.tree.parent_id(n));
        if let Some(parent) = parent {
            self.changes.mark(parent, Dirty::LAYOUT_CHAIN);
        }
        self.changes.outdate_viewport();
        self.changes
            .layout
            .insert(crate::model::changes::LayoutReasons::STYLE);
        let notify: Vec<_> = self
            .tree
            .subtree(id)
            .into_iter()
            .filter(|gone| {
                self.tree.get(*gone).is_some_and(|node| {
                    matches!(
                        node.presence,
                        NodePresence::Leaving(LeavingRole::Root {
                            notify: Notify::OnComplete
                        })
                    )
                })
            })
            .collect();
        for gone in self.tree.subtree(id) {
            self.transition_presence(gone, NodePresence::Mounted);
        }
        for gone in self.remove_slots(id) {
            if notify.contains(&gone) {
                self.push(HostEvent::ExitComplete { node: gone });
            }
            self.changes.a11y_removed.insert(gone);
            let previous = self.set_edit(gone, None);
            debug_assert!(previous.is_none());
            self.changes.forget(gone);
        }
        self.focus.forget_missing(&self.tree);
        self.clear_removed_hits();
        self.pointer.forget_missing(&self.tree);
        if let Some(p) = parent {
            self.changes.regroup(p);
        }
        if let Some(left) = focus_lost {
            // Move the focus to the next tab stop before reporting, out of
            // the groups it was in (read before the removal).
            self.move_focus_off_removed(&left, next);
        }
        self.changes
            .layout
            .insert(crate::model::changes::LayoutReasons::PASS);
    }
}
