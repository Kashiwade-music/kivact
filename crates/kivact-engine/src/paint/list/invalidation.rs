//! Role: Invalidate retained paint list entries and their recorded ink.

use crate::model::changes::Dispatched;
use crate::paint::state::{Ink, PaintList, PaintState};
use kivact_types::NodeId;

impl PaintList {
    pub(crate) fn new() -> Self {
        PaintList {
            stale: true,
            ..PaintList::default()
        }
    }

    /// Note what `changes` outdated in the inks and the child indices:
    /// after a layout pass every location (and the sizes that changed); a
    /// node whose children changed (its index is dropped); a
    /// node whose drawing changed (its paint, scenes, states, visibility or
    /// inherited text size) or that scrolled forgets its ink and its
    /// ancestors' ([`PaintList::forget_ink`]).
    fn invalidate(
        &mut self,
        changes: &Dispatched,
        tree: &crate::tree::NodeTree,
        ink: &[std::cell::Cell<Option<Ink>>],
    ) {
        if changes.was_laid_out() {
            self.pass += 1;
        }
        for id in changes.regrouped() {
            self.indices.remove(&id);
        }
        for id in changes.redrawn() {
            self.forget_ink(id, tree, ink);
        }
    }

    /// Something in the subtree of `id` changed what it draws or where:
    /// it and its ancestors forget their inks, and the indices holding
    /// them look at them again. Up to the first node without an ink (of
    /// this generation): no ink above it, and no index entry, was found
    /// from it since it forgot its own (finding an ink finds the inks it
    /// is made of, and an index entry is an ink), so they forgot theirs
    /// with it.
    pub(crate) fn forget_ink(
        &mut self,
        id: NodeId,
        tree: &crate::tree::NodeTree,
        ink: &[std::cell::Cell<Option<Ink>>],
    ) {
        let mut at = Some(id);
        while let Some(x) = at {
            let Some(n) = tree.get(x) else {
                break;
            };
            if ink[n.index.0]
                .take()
                .is_none_or(|kept| kept.generation != self.ink_gen)
            {
                break;
            }
            at = tree.parent_id(n);
            if let Some(index) = at.and_then(|parent| self.indices.get_mut(&parent)) {
                index.touch(x);
            }
        }
    }

    /// A node has a backdrop filter reading `reach` around its box: the
    /// viewport grows to it (it never shrinks back, which only lists a few
    /// more nodes).
    pub(crate) fn note_backdrop(&mut self, reach: f32) {
        if reach.is_finite() && reach > self.backdrop_reach {
            self.backdrop_reach = reach;
            self.stale = true;
        }
    }

    /// Every node's measure may have changed (fonts were added): forget
    /// every ink and index.
    pub(crate) fn forget_inks(&mut self) {
        self.ink_gen += 1;
        self.indices.clear();
        self.stale = true;
    }
}

impl PaintState {
    /// Invalidate the retained list for one sync: redraw the frame, forget
    /// font-dependent ink, and rebuild the list when stacking changed.
    pub(crate) fn outdate_paint(
        &mut self,
        tree: &crate::tree::NodeTree,
        changes: &Dispatched,
        redraw: bool,
        fonts: bool,
        restack: bool,
    ) {
        if redraw {
            self.needs_redraw = true;
        }
        let mut list = std::mem::take(&mut self.list);
        if fonts {
            list.forget_inks();
        }
        list.stale |= restack;
        list.invalidate(changes, tree, &self.ink);
        self.list = list;
    }
}
