//! Role: Start entrance transitions and run exits before a node is removed.

use crate::contract::{HostEvent, NodeType};
use crate::model::Model;
use crate::mutation::Mutation;
use crate::tree::node::{ArrivalPhase, LeavingRole, NodePresence, Notify};
use kivact_types::{Dimension, Edges, NodeId, Position};

impl Model {
    /// Update authoritative presence and its scheduling indexes together.
    pub(crate) fn transition_presence(&mut self, id: NodeId, presence: NodePresence) {
        if self.tree.get(id).is_none() {
            return;
        }
        let before = self.tree.presence_of(id);
        let presence = match (before, presence) {
            (
                NodePresence::Leaving(LeavingRole::Root {
                    notify: Notify::OnComplete,
                }),
                NodePresence::Leaving(LeavingRole::Root { .. }),
            ) => NodePresence::Leaving(LeavingRole::Root {
                notify: Notify::OnComplete,
            }),
            (_, presence) => presence,
        };
        self.tree.set_presence(id, presence);
        let previous = kivact_types::Presence::from(before);
        let current = kivact_types::Presence::from(presence);
        if previous != current {
            self.events.push(HostEvent::PresenceChanged {
                node: id,
                presence: current,
            });
        }
    }

    /// Forget the presence of a node that is physically destroyed.
    pub(crate) fn forget_presence(&mut self, id: NodeId) {
        self.tree.forget_presence(id);
    }

    /// `id` was created: it enters at the first commit that finds it
    /// attached.
    pub(crate) fn arrive(&mut self, id: NodeId) {
        self.transition_presence(id, NodePresence::Arriving(ArrivalPhase::Pending));
    }

    /// Capture the nodes attached since the last commit at their enter
    /// paint, so the next retarget moves them to their paint.
    pub(crate) fn start_entrances(&mut self) {
        let arriving: Vec<_> = self
            .tree
            .presence_ids(|p| matches!(p, NodePresence::Arriving(_)))
            .into_iter()
            .filter(|id| {
                matches!(
                    self.tree.presence_of(*id),
                    NodePresence::Arriving(ArrivalPhase::Pending)
                )
            })
            .collect();
        for id in arriving {
            let enters = self
                .tree
                .get(id)
                .and_then(|n| n.states())
                .is_some_and(|s| s.enter.is_some())
                || self.size_shown(id, true);
            if !enters {
                self.transition_presence(id, NodePresence::Mounted);
                continue;
            }
            if !self.tree.attached(id) {
                continue;
            }
            if self.transition_shown(id, |s| s.enter.is_some()) || self.size_shown(id, true) {
                // Writing its props captured it at its paint: it starts at
                // its enter paint instead.
                self.transition_presence(id, NodePresence::Arriving(ArrivalPhase::Animating));
                self.motion.forget(id);
                self.prepare_entrance(id);
                self.start_size(id, true);
            } else {
                self.transition_presence(id, NodePresence::Mounted);
            }
        }
    }

    /// Complete attached entrances once their captured motion settles.
    pub(crate) fn finish_entrances(&mut self) {
        let done: Vec<_> = self
            .tree
            .presence_ids(|p| matches!(p, NodePresence::Arriving(_)))
            .into_iter()
            .filter(|id| {
                matches!(
                    self.tree.presence_of(*id),
                    NodePresence::Arriving(ArrivalPhase::Animating)
                ) && !self.motion.in_flight(*id)
                    && !self.presence.sizes.contains_key(id)
            })
            .collect();
        for id in done {
            self.transition_presence(id, NodePresence::Mounted);
        }
    }

    /// Whether `id`'s states have what `has` asks for and its paint moves
    /// to it: it has a paint transition that reduced motion does not skip.
    fn transition_shown(
        &self,
        id: NodeId,
        has: impl Fn(&kivact_types::StateStyles) -> bool,
    ) -> bool {
        let Some(n) = self.tree.get(id) else {
            return false;
        };
        let moves = n
            .paint()
            .and_then(|p| p.transitions.as_ref())
            .is_some_and(|list| {
                list.iter()
                    .any(|(_, t)| !(self.frame.reduced_motion() && t.decorative))
            });
        moves && n.states().is_some_and(has)
    }

    /// Start the exit of the subtree kivact removes at `id`: `false` when
    /// nothing in it has an exit to show, and the subtree goes at once.
    pub(crate) fn begin_exit(&mut self, id: NodeId, pop_layout: bool, notify: Notify) -> bool {
        if !self.tree.drawn(id) {
            return false;
        }
        let subtree = self.tree.subtree(id);
        let exits: Vec<NodeId> = subtree
            .iter()
            .copied()
            .filter(|x| {
                self.transition_shown(*x, |s| s.exit.is_some())
                    || (!pop_layout && self.size_shown(*x, false))
                    || self.tree.leaving(*x)
            })
            .collect();
        if exits.is_empty() {
            return false;
        }
        let size_starts: Vec<_> = exits
            .iter()
            .copied()
            .filter(|x| !self.tree.leaving(*x))
            .collect();
        let focus_lost = self
            .focused()
            .is_some_and(|f| self.tree.is_in_subtree(f.node, id));
        let next = if focus_lost {
            self.next_tab_stop_outside(id)
        } else {
            None
        };
        // The exit starts from what is shown now.
        for x in &exits {
            self.prepare_state_motion(*x);
        }
        self.record_departures(&subtree);
        for x in &subtree {
            self.tree.set_observed(*x, false);
            let role = if *x == id {
                LeavingRole::Root { notify }
            } else if let NodePresence::Leaving(role @ LeavingRole::Root { .. }) =
                self.tree.presence_of(*x)
            {
                role
            } else {
                LeavingRole::Descendant
            };
            self.transition_presence(*x, NodePresence::Leaving(role));
        }
        if !pop_layout {
            for x in &size_starts {
                self.start_size(*x, false);
            }
        }
        if pop_layout {
            for x in &subtree {
                self.presence.sizes.remove(x);
                if let Some(n) = self.tree.get(*x) {
                    self.layout.set_presence_scale(&self.tree, n.index.0, None);
                }
            }
            self.pop_exit_layout(id);
        }
        self.a11y.subtree(id);
        self.changes.restack = true;
        self.stale_fact(id);
        self.revalidate_hover();
        self.focus.forget_missing(&self.tree);
        self.pointer.forget_missing(&self.tree);
        if focus_lost {
            self.move_focus_to(next);
        }
        true
    }

    /// Keep the old box in the same parent and paint layer, but release its slot.
    fn pop_exit_layout(&mut self, id: NodeId) {
        let Some(node) = self.tree.get(id) else {
            return;
        };
        if node.is_transparent() {
            let children: Vec<_> = self.tree.flat_children(id).collect();
            for child in children {
                self.pop_exit_layout(child);
            }
            return;
        }
        let mut layout = node.layout().cloned().unwrap_or_default();
        if node.kind() == NodeType::Overlay
            || matches!(layout.position, Position::Absolute | Position::Fixed)
        {
            return;
        }
        let Some(parent) = self.tree.flat_parent(node) else {
            return;
        };
        let Some(last) = self.computed_layout(id) else {
            return;
        };
        let border = self
            .tree
            .get(parent)
            .and_then(|parent| parent.layout())
            .map_or(Edges::all(0.0), |parent| parent.border);
        // Layout locations are parent border-box relative, while an absolute
        // inset is relative to the inside of the parent's border.
        let left = last.location.x + node.shift.x - border.left;
        let top = last.location.y + node.shift.y - border.top;
        layout.position = Position::Absolute;
        layout.inset = Edges::all(Dimension::Auto);
        layout.inset.left = Dimension::Px(left);
        layout.inset.top = Dimension::Px(top);
        layout.margin = Edges::all(Dimension::Px(0.0));
        layout.width = Dimension::Px(last.size.width);
        layout.height = Dimension::Px(last.size.height);
        layout.min_width = Dimension::Auto;
        layout.min_height = Dimension::Auto;
        layout.max_width = Dimension::Auto;
        layout.max_height = Dimension::Auto;
        if let Some(grid) = &mut layout.grid {
            grid.column = Default::default();
            grid.row = Default::default();
        }
        // The client already dropped the host's core record and cannot write
        // another layout. Reuse the ordinary style invalidation path.
        if let Some(node) = self.tree.get_mut(id) {
            node.properties.layout = Some(std::rc::Rc::new(layout));
        }
        self.changed(id, Mutation::STYLE.and(Mutation::RESTACK));
    }

    /// Remove the leaving subtrees in which nothing moves any more.
    pub(crate) fn finish_exits(&mut self) {
        let done: Vec<NodeId> = self
            .tree
            .presence_ids(|p| matches!(p, NodePresence::Leaving(LeavingRole::Root { .. })))
            .into_iter()
            .filter(|root| {
                !self.tree.contains(*root)
                    || (!self
                        .motion
                        .moving()
                        .any(|x| self.tree.is_in_subtree(x, *root))
                        && !self
                            .presence
                            .sizes
                            .keys()
                            .any(|x| self.tree.is_in_subtree(*x, *root)))
            })
            .collect();
        for root in done {
            if self.tree.contains(root) {
                self.remove_now(root);
            } else {
                self.transition_presence(root, NodePresence::Mounted);
            }
        }
    }
}
