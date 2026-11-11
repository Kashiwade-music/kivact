//! Role: Invalidate retained paint list entries and their recorded ink.

use crate::model::changes::Dispatched;
use crate::paint::state::{Ink, ListState, PaintList, PaintState, Redraw};
use kivact_types::NodeId;

impl PaintList {
    pub(crate) fn new() -> Self {
        PaintList::default()
    }

    /// Note what `changes` outdated in the inks and the child indices. The
    /// children that moved or resized in a layout pass were marked in their
    /// parent's index by [`PaintState::moved_child`], and only those are
    /// looked at again. A node whose children changed drops its index. A
    /// node whose drawing changed (its paint, scenes, states, visibility or
    /// inherited text size) or that scrolled forgets its ink and its
    /// ancestors' ([`PaintList::forget_ink`]).
    fn invalidate(
        &mut self,
        changes: &Dispatched,
        tree: &crate::tree::NodeTree,
        ink: &[std::cell::Cell<Option<Ink>>],
    ) {
        for id in changes.regrouped() {
            // A Contents node's children are indexed by the box they are
            // drawn in.
            if let Some(holder) = tree.box_of(id) {
                self.indices.remove(&holder);
            }
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
        // Inks and index entries belong to boxes: a Contents node has
        // neither, and its children's are held by its flat parent.
        let mut at = tree.box_of(id);
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
            at = tree.flat_parent(n);
            if let Some(index) = at.and_then(|parent| self.indices.get_mut(&parent)) {
                index.touch(x);
            }
        }
    }

    /// Every node's measure may have changed (fonts were added): forget
    /// every ink and index.
    pub(crate) fn forget_inks(&mut self) {
        self.ink_gen.next();
        self.indices.clear();
        self.state = ListState::Stale;
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
            self.redraw = Redraw::Needed;
        }
        let mut list = std::mem::take(&mut self.list);
        if fonts {
            list.forget_inks();
        }
        if restack {
            list.state = ListState::Stale;
        }
        list.invalidate(changes, tree, &self.ink);
        self.list = list;
    }
}
