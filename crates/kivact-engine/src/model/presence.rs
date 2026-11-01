//! Role: Enter and exit paint: where a node starts at mount and ends before it goes.
//!
//! A node's enter paint (`StateStyles::enter`, CSS `@starting-style`) is
//! where its paint transition starts when it is first attached; its exit
//! paint (`StateStyles::exit`) is where the transition takes a removed
//! node before it goes. Both are state deltas over the others (`states`)
//! moved by the node's paint transition (`state_motion`), so only what
//! that transition interpolates enters and leaves; no animation of its
//! own exists for them. Kivact's removal stays one write: the engine keeps
//! what has an exit to show.
//!
//! Invariants:
//! - A created node with an enter paint is captured at it once, at the
//!   first commit that finds it attached, and the next retarget moves it
//!   to its paint; one without an enter paint at the first commit after
//!   its creation (an enter added later) never enters.
//! - A removed subtree that has an exit to show stays in the tree as
//!   leaving (every node of it): it is drawn where it was laid out, but it
//!   is never hit, focused, exposed, observed or in a modal scope, and the
//!   focus inside it moves out at once, as for a removal; a scrollbar or
//!   file drag on it ends. Kivact forgot it at the removal and never names
//!   it again. A subtree that is not drawn (detached, or hidden) goes at
//!   once.
//! - A leaving subtree is removed at the first sync after no node in it
//!   moves; until then it keeps its place in the layout, so its siblings
//!   do not move before (as with a CSS exit transition and Framer Motion's
//!   `AnimatePresence`). One whose ancestor kivact removes goes with the
//!   ancestor.
//! - Without a paint transition, or under reduced motion with a decorative
//!   one, a node neither enters nor leaves: it shows its paint and goes at
//!   once.

use crate::model::Model;
use crate::tree::node::Presence;
use kivact_wire::NodeId;

pub(crate) mod state;

impl Model {
    /// `id` was created: it enters at the first commit that finds it
    /// attached.
    pub(crate) fn arrive(&mut self, id: NodeId) {
        self.presence.arriving.insert(id);
    }

    /// Capture the nodes attached since the last commit at their enter
    /// paint, so the next retarget moves them to their paint.
    pub(crate) fn start_entrances(&mut self) {
        if self.presence.arriving.is_empty() {
            return;
        }
        let arriving = std::mem::take(&mut self.presence.arriving);
        for id in arriving {
            let enters = self
                .tree
                .get(id)
                .and_then(|n| n.states())
                .is_some_and(|s| s.enter.is_some());
            if !enters {
                continue;
            }
            if !self.tree.attached(id) {
                self.presence.arriving.insert(id);
                continue;
            }
            if self.transition_shown(id, |s| s.enter.is_some())
                && let Some(n) = self.tree.get_mut(id)
            {
                n.presence = Presence::Entering;
                // Writing its props captured it at its paint: it starts at
                // its enter paint instead.
                self.motion.forget(id);
                self.prepare_state_motion(id);
                if let Some(n) = self.tree.get_mut(id) {
                    n.presence = Presence::Mounted;
                }
            }
        }
    }

    /// Whether `id`'s states have what `has` asks for and its paint moves
    /// to it: it has a paint transition that reduced motion does not skip.
    fn transition_shown(
        &self,
        id: NodeId,
        has: impl Fn(&kivact_wire::StateStyles) -> bool,
    ) -> bool {
        let Some(n) = self.tree.get(id) else {
            return false;
        };
        let moves = n
            .paint()
            .and_then(|p| p.transition.as_ref())
            .is_some_and(|t| !(self.frame.reduced_motion() && t.decorative));
        moves && n.states().is_some_and(has)
    }

    /// Start the exit of the subtree kivact removes at `id`: `false` when
    /// nothing in it has an exit to show, and the subtree goes at once.
    pub(crate) fn begin_exit(&mut self, id: NodeId) -> bool {
        if !self.tree.drawn(id) {
            return false;
        }
        let subtree = self.tree.subtree(id);
        let exits: Vec<NodeId> = subtree
            .iter()
            .copied()
            .filter(|x| self.transition_shown(*x, |s| s.exit.is_some()))
            .collect();
        if exits.is_empty() {
            return false;
        }
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
        for x in &subtree {
            self.tree.set_observed(*x, false);
            if let Some(n) = self.tree.get_mut(*x) {
                n.presence = Presence::Leaving;
            }
        }
        self.presence.leaving.insert(id);
        self.changes.a11y_subtrees.insert(id);
        self.changes.restack = true;
        self.revalidate_hover();
        self.focus.forget_missing(&self.tree);
        self.pointer.forget_missing(&self.tree);
        if focus_lost {
            self.move_focus_to(next);
        }
        true
    }

    /// Remove the leaving subtrees in which nothing moves any more.
    pub(crate) fn finish_exits(&mut self) {
        if self.presence.leaving.is_empty() {
            return;
        }
        let done: Vec<NodeId> = self
            .presence
            .leaving
            .iter()
            .copied()
            .filter(|root| {
                !self.tree.contains(*root)
                    || !self
                        .motion
                        .moving()
                        .any(|x| self.tree.is_in_subtree(x, *root))
            })
            .collect();
        for root in done {
            self.presence.leaving.remove(&root);
            if self.tree.contains(root) {
                self.remove_now(root);
            }
        }
    }
}
