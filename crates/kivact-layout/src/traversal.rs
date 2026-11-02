//! Role: Traverses retained children through derived layout cursors.
//!
//! The children taffy sees: a node's children in flow (its links without
//! its overlays, which are roots of their own, and its fixed boxes, which
//! are children of the shared viewport), and the viewport's attached fixed
//! boxes.
//!
//! Invariants:
//! - While no overlay is attached and no node is fixed, children are the
//!   links as they are: no cursor is read and nothing is filtered. A node
//!   whose children are all in flow is walked directly as well.
//! - Otherwise a node's cursor (`ChildCursor`: never counted, or counted at
//!   an epoch with its count, whether it is unfiltered and the last position)
//!   only follows indices into its links past the filtered children; it
//!   copies no child ids or links. It is counted again once per layout epoch, which the sync bumps when the
//!   structure or the fixed membership changed: a new height alone counts
//!   nothing again.
//! - Iterators borrow the same concrete Tree for their entire traversal;
//!   no boxing, conversion or ownership change accompanies specialization.

use crate::{ChildCursor, Counted, Flow, LayoutState, LayoutTree};
use kivact_wire::NodeId;
use taffy::{NodeId as TId, TraversePartialTree, TraverseTree};

/// A borrowed traversal view; all retained data are derived cursors, not links.
pub(crate) struct Traversal<'a, Tree: LayoutTree + ?Sized> {
    pub(crate) tree: &'a Tree,
    pub(crate) cursors: &'a [ChildCursor],
    pub(crate) state: &'a LayoutState,
}

pub(crate) struct FlowChildren<'a, Tree: LayoutTree + ?Sized> {
    tree: &'a Tree,
    slot: usize,
    children: std::ops::Range<usize>,
    unfiltered: bool,
}

impl<Tree: LayoutTree + ?Sized> Iterator for FlowChildren<'_, Tree> {
    type Item = TId;
    #[inline(always)]
    fn next(&mut self) -> Option<TId> {
        for raw in self.children.by_ref() {
            let slot = self.tree.child_at(self.slot, raw);
            if self.unfiltered || self.tree.flow(slot) == Flow::Normal {
                return Some(taffy_id(slot));
            }
        }
        None
    }
}

pub(crate) struct FixedChildren<'a, Tree: LayoutTree + ?Sized> {
    tree: &'a Tree,
    children: std::collections::btree_set::Iter<'a, NodeId>,
}

fn attached_fixed<Tree: LayoutTree + ?Sized>(tree: &Tree, id: NodeId) -> bool {
    tree.index_of(id)
        .is_some_and(|slot| tree.parent(slot).is_some() && tree.flow(slot) != Flow::Overlay)
}

impl<Tree: LayoutTree + ?Sized> Iterator for FixedChildren<'_, Tree> {
    type Item = TId;
    #[inline(always)]
    fn next(&mut self) -> Option<TId> {
        self.children
            .find(|id| attached_fixed(self.tree, **id))
            .map(|id| {
                taffy_id(
                    self.tree
                        .index_of(*id)
                        .expect("invariant: fixed node exists"),
                )
            })
    }
}

pub(crate) enum LayoutChildren<'a, Tree: LayoutTree + ?Sized> {
    Flow(FlowChildren<'a, Tree>),
    Fixed(FixedChildren<'a, Tree>),
}
impl<Tree: LayoutTree + ?Sized> Iterator for LayoutChildren<'_, Tree> {
    type Item = TId;
    #[inline(always)]
    fn next(&mut self) -> Option<TId> {
        match self {
            Self::Flow(i) => i.next(),
            Self::Fixed(i) => i.next(),
        }
    }
}

/// What a taffy node id names: the shared viewport (position 0) or a tree slot (position - 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    /// The shared viewport, the container of the fixed boxes.
    Viewport,
    /// The tree slot (zero-based, excluding the viewport).
    Slot(usize),
}

/// The entry taffy's `t` names.
#[inline(always)]
pub(crate) fn entry(t: TId) -> Entry {
    match usize::from(t) {
        0 => Entry::Viewport,
        index => Entry::Slot(index - 1),
    }
}

/// The taffy id of the shared viewport.
#[inline(always)]
pub(crate) fn viewport() -> TId {
    TId::from(0usize)
}

#[inline(always)]
pub(crate) fn taffy_id(slot: usize) -> TId {
    TId::from(
        slot.checked_add(1)
            .expect("invariant: layout index fits usize"),
    )
}

impl<Tree: LayoutTree + ?Sized> Traversal<'_, Tree> {
    #[inline(always)]
    fn filters_layout_children(&self) -> bool {
        !self.tree.overlays().is_empty() || !self.tree.fixed().is_empty()
    }
    #[inline(always)]
    fn in_flow(&self, slot: usize) -> bool {
        self.tree.flow(slot) == Flow::Normal
    }
    #[inline(always)]
    fn flow_cursor(&self, slot: usize) -> Counted {
        self.cursors[slot + 1].at_epoch(self.state.epoch, || self.count_flow(slot))
    }
    /// The children of `slot` in flow, and whether none is filtered out.
    #[inline(never)]
    fn count_flow(&self, slot: usize) -> (usize, bool) {
        let raw_count = self.tree.child_count(slot);
        let count = (0..raw_count)
            .filter(|raw| self.in_flow(self.tree.child_at(slot, *raw)))
            .count();
        (count, count == raw_count)
    }
    #[inline(always)]
    fn flow_child(&self, slot: usize, index: usize) -> TId {
        if !self.filters_layout_children() {
            assert!(
                index < self.tree.child_count(slot),
                "invariant: valid layout child index"
            );
            return taffy_id(self.tree.child_at(slot, index));
        }
        let counted = self.flow_cursor(slot);
        assert!(index < counted.count, "invariant: valid layout child index");
        if counted.unfiltered {
            return taffy_id(self.tree.child_at(slot, index));
        }
        self.filtered_flow_child(slot, index, counted)
    }
    #[inline(never)]
    fn filtered_flow_child(&self, slot: usize, index: usize, counted: Counted) -> TId {
        let at = counted.at;
        let (mut filtered, mut raw) = at.unwrap_or((0, 0));
        if at.is_none() {
            raw = (0..self.tree.child_count(slot))
                .position(|raw| self.in_flow(self.tree.child_at(slot, raw)))
                .expect("invariant: nonempty filtered children");
        }
        while filtered < index {
            raw += 1;
            while !self.in_flow(self.tree.child_at(slot, raw)) {
                raw += 1;
            }
            filtered += 1;
        }
        while filtered > index {
            raw -= 1;
            while !self.in_flow(self.tree.child_at(slot, raw)) {
                raw -= 1;
            }
            filtered -= 1;
        }
        self.cursors[slot + 1].store(Counted {
            at: Some((filtered, raw)),
            ..counted
        });
        taffy_id(self.tree.child_at(slot, raw))
    }
    fn fixed_child(&self, index: usize) -> TId {
        use std::ops::Bound::{Excluded, Unbounded};
        let mut at = self.state.fixed_cursor.get();
        if at.is_none() {
            at = self
                .tree
                .fixed()
                .iter()
                .copied()
                .find(|id| attached_fixed(self.tree, *id))
                .map(|id| (0, id));
        }
        let (mut current, mut id) = at.expect("invariant: fixed layout child exists");
        if index > current {
            for candidate in self.tree.fixed().range((Excluded(id), Unbounded)).copied() {
                if attached_fixed(self.tree, candidate) {
                    current += 1;
                    id = candidate;
                    if current == index {
                        break;
                    }
                }
            }
        } else if index < current {
            for candidate in self
                .tree
                .fixed()
                .range((Unbounded, Excluded(id)))
                .rev()
                .copied()
            {
                if attached_fixed(self.tree, candidate) {
                    current -= 1;
                    id = candidate;
                    if current == index {
                        break;
                    }
                }
            }
        }
        assert_eq!(current, index, "invariant: valid fixed layout index");
        self.state.fixed_cursor.set(Some((current, id)));
        taffy_id(
            self.tree
                .index_of(id)
                .expect("invariant: fixed node exists"),
        )
    }
}
impl<Tree: LayoutTree + ?Sized> TraversePartialTree for Traversal<'_, Tree> {
    type ChildIter<'a>
        = LayoutChildren<'a, Tree>
    where
        Self: 'a;
    #[inline(always)]
    fn child_ids(&self, parent: TId) -> Self::ChildIter<'_> {
        match entry(parent) {
            Entry::Viewport => LayoutChildren::Fixed(FixedChildren {
                tree: self.tree,
                children: self.tree.fixed().iter(),
            }),
            Entry::Slot(slot) => LayoutChildren::Flow(FlowChildren {
                tree: self.tree,
                slot,
                children: 0..self.tree.child_count(slot),
                unfiltered: !self.filters_layout_children() || self.flow_cursor(slot).unfiltered,
            }),
        }
    }
    #[inline(always)]
    fn child_count(&self, parent: TId) -> usize {
        match entry(parent) {
            Entry::Viewport => {
                self.cursors[0]
                    .at_epoch(self.state.epoch, || {
                        let count = self
                            .tree
                            .fixed()
                            .iter()
                            .filter(|id| attached_fixed(self.tree, **id))
                            .count();
                        (count, false)
                    })
                    .count
            }
            Entry::Slot(slot) if !self.filters_layout_children() => self.tree.child_count(slot),
            Entry::Slot(slot) => self.flow_cursor(slot).count,
        }
    }
    #[inline(always)]
    fn get_child_id(&self, parent: TId, index: usize) -> TId {
        match entry(parent) {
            Entry::Viewport => self.fixed_child(index),
            Entry::Slot(slot) => self.flow_child(slot, index),
        }
    }
}
impl<Tree: LayoutTree + ?Sized> TraverseTree for Traversal<'_, Tree> {}
