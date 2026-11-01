//! Role: Authoritative raw node values, stable slots and structural links.
//!
//! The one ID map and one sibling order live here. Editing order is immediately
//! visible and packed before indexed layout. No layout, paint or shaping state
//! belongs to this structural owner. Upper Model lifecycle code initializes and
//! resets derived columns before publishing a slot for reuse.
//!
//! Removal accepts an upper-owned payload callback without naming its types.
//! Derived columns reset before reuse, and each payload drops after that node's
//! structural cleanup, before its descendants and before Writer attribution.
//! Detaching keeps every slot and its state.

use kivact_wire::{IdMap, NodeId, NodeType};
use std::collections::BTreeSet;

mod child_order;
pub(crate) mod node;
pub(crate) use child_order::{ChildOrder, Children};

use crate::tree::node::{HostNode, NodeIndex, Presence};

/// The sole retained parent and children of a primary node slot.
/// Child order changes immediately; linked editing order is packed before layout.
#[derive(Clone, Debug, Default)]
pub(crate) struct NodeLinks {
    pub(crate) parent: Option<NodeIndex>,
    pub(crate) children: ChildOrder,
}

/// The sole engine node store, with stable reusable positions.
///
/// Logical IDs resolve here for every engine operation. Retained parent and
/// child links use these positions directly; layout owns no second tree.
///
/// Each position holds a node (`HostNode`), its links (`NodeLinks`, the
/// one record of parent and children) and its layout components
/// (owned above this structural store), each kept once. The one logical
/// index (`by_id`, also used by `get` and `remove`) is the only place an
/// id is hashed: nodes and links refer to positions (`NodeIndex`), so
/// enumerating packed layout children hashes nothing, and kivact's `NodeId`s stay
/// as they are. The index hashes with a cheap multiplicative mix
/// (`IdMap`): the core hands ids out in increasing order and never reuses
/// them, so input cannot skew it, and SipHash had cost more than the
/// memory access of the lookups the layout, hit testing and the paint
/// list rebuild make per node.
///
/// A removed node's position has its layout components reset before it is
/// freed, and the next created node takes it without resetting it again,
/// so the store holds at most as many positions as nodes were alive at
/// once.
#[derive(Clone, Debug, Default)]
pub(crate) struct NodeStore {
    by_id: IdMap<NodeIndex>,
    entries: Vec<Option<(NodeId, HostNode)>>,
    free: Vec<NodeIndex>,
    links: Vec<NodeLinks>,
}

impl NodeStore {
    #[inline(always)]
    pub(crate) fn get(&self, id: &NodeId) -> Option<&HostNode> {
        self.by_id.get(id).map(|index| self.at(*index))
    }

    #[inline(always)]
    pub(crate) fn get_mut(&mut self, id: &NodeId) -> Option<&mut HostNode> {
        let index = *self.by_id.get(id)?;
        Some(self.at_mut(index))
    }

    pub(crate) fn contains_key(&self, id: &NodeId) -> bool {
        self.by_id.contains_key(id)
    }

    pub(crate) fn len(&self) -> usize {
        self.by_id.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&NodeId, &HostNode)> {
        self.entries
            .iter()
            .filter_map(|entry| entry.as_ref().map(|(id, node)| (id, node)))
    }

    /// Take remaining payloads in storage order during owner destruction.
    /// No slot is published or tree mutation callback is invoked.
    pub(crate) fn drain_payloads(&mut self, dispose: impl FnMut(NodeIndex, HostNode)) {
        struct Remaining<'a, F: FnMut(NodeIndex, HostNode)> {
            entries: &'a mut [Option<(NodeId, HostNode)>],
            next: usize,
            dispose: F,
        }
        impl<F: FnMut(NodeIndex, HostNode)> Remaining<'_, F> {
            fn run(&mut self) {
                while self.next < self.entries.len() {
                    let slot = self.next;
                    self.next += 1;
                    if let Some((_, node)) = self.entries[slot].take() {
                        (self.dispose)(NodeIndex(slot), node);
                    }
                }
            }
        }
        impl<F: FnMut(NodeIndex, HostNode)> Drop for Remaining<'_, F> {
            fn drop(&mut self) {
                self.run();
            }
        }
        let mut remaining = Remaining {
            entries: &mut self.entries,
            next: 0,
            dispose,
        };
        remaining.run();
    }

    /// Reserve one vacant structural slot. Derived owners initialize it before install.
    pub(crate) fn reserve(&mut self) -> (NodeIndex, bool) {
        if let Some(index) = self.free.pop() {
            return (index, false);
        }
        let index = NodeIndex(self.entries.len());
        self.entries.push(None);
        self.links.push(NodeLinks::default());
        (index, true)
    }
    pub(crate) fn install(&mut self, id: NodeId, mut node: HostNode, slot: NodeIndex) {
        debug_assert!(!self.by_id.contains_key(&id));
        debug_assert!(self.entries[slot.0].is_none());
        node.index = slot;
        self.links[slot.0] = NodeLinks::default();
        self.entries[slot.0] = Some((id, node));
        self.by_id.insert(id, slot);
    }
    pub(crate) fn take(&mut self, id: NodeId) -> Option<(NodeIndex, HostNode, NodeLinks)> {
        let index = self.by_id.remove(&id)?;
        let (_, node) = self.entries[index.0].take().expect("indexed slot is live");
        let links = std::mem::take(&mut self.links[index.0]);
        Some((index, node, links))
    }
    /// Only the upper lifecycle owner calls this after resetting all derived columns.
    pub(crate) fn release(&mut self, index: NodeIndex) {
        assert!(self.entries[index.0].is_none());
        self.free.push(index);
    }
    #[inline(always)]
    pub(crate) fn links(&self, index: NodeIndex) -> &NodeLinks {
        &self.links[index.0]
    }

    #[inline(always)]
    pub(crate) fn links_mut(&mut self, index: NodeIndex) -> &mut NodeLinks {
        &mut self.links[index.0]
    }

    #[inline(always)]
    pub(crate) fn slot_id(&self, slot: usize) -> Option<NodeId> {
        self.entries.get(slot)?.as_ref().map(|(id, _)| *id)
    }

    #[inline(always)]
    pub(crate) fn index_of(&self, id: NodeId) -> Option<NodeIndex> {
        self.by_id.get(&id).copied()
    }

    #[inline(always)]
    pub(crate) fn at(&self, index: NodeIndex) -> &HostNode {
        &self.entries[index.0]
            .as_ref()
            .expect("invariant: indexed node is live")
            .1
    }

    #[inline(always)]
    pub(crate) fn at_mut(&mut self, index: NodeIndex) -> &mut HostNode {
        &mut self.entries[index.0]
            .as_mut()
            .expect("invariant: indexed node is live")
            .1
    }

    #[inline(always)]
    pub(crate) fn id_at(&self, index: NodeIndex) -> NodeId {
        self.entries[index.0]
            .as_ref()
            .expect("invariant: indexed node is live")
            .0
    }
}

/// The node tree.
#[derive(Clone, Debug, Default)]
pub(crate) struct NodeTree {
    /// Derived packing work only; the child order remains solely in NodeLinks.
    pending_child_order: BTreeSet<NodeId>,
    pub(crate) nodes: NodeStore,
    pub(crate) root: Option<NodeId>,
    pub(crate) overlays: Vec<NodeId>,
    pub(crate) scrollers: BTreeSet<NodeId>,
    pub(crate) observed: BTreeSet<NodeId>,
    /// Image leaves, attached or detached, ordered for asset polling.
    pub(crate) images: BTreeSet<NodeId>,
    pub(crate) sticky: BTreeSet<NodeId>,
    pub(crate) fixed: BTreeSet<NodeId>,
}

/// Whether state paint can transform or reorder a node.
pub(crate) fn states_place(styles: &kivact_wire::StateStyles) -> bool {
    styles
        .deltas()
        .any(|delta| delta.transform.is_some() || delta.z_index.is_some())
}

impl NodeTree {
    /// The node `id`.
    pub(crate) fn get(&self, id: NodeId) -> Option<&HostNode> {
        self.nodes.get(&id)
    }

    pub(crate) fn get_mut(&mut self, id: NodeId) -> Option<&mut HostNode> {
        self.nodes.get_mut(&id)
    }

    pub(crate) fn parent_id(&self, node: &HostNode) -> Option<NodeId> {
        self.nodes
            .links(node.index)
            .parent
            .map(|index| self.nodes.id_at(index))
    }

    pub(crate) fn children_ids<'a>(
        &'a self,
        node: &'a HostNode,
    ) -> impl DoubleEndedIterator<Item = NodeId> + ExactSizeIterator + 'a {
        self.nodes
            .links(node.index)
            .children
            .iter()
            .map(|index| self.nodes.id_at(index))
    }

    /// Whether `id` is a node of the tree (attached or not).
    pub(crate) fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// The root node.
    pub(crate) fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Every node, in no particular order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (NodeId, &HostNode)> {
        self.nodes.iter().map(|(id, n)| (*id, n))
    }

    /// Pre-order traversal from the root (overlays where they are attached).
    pub(crate) fn preorder(&self) -> Vec<NodeId> {
        let mut out = Vec::with_capacity(self.nodes.len());
        let mut stack: Vec<NodeId> = self.root.into_iter().collect();
        while let Some(id) = stack.pop() {
            out.push(id);
            if let Some(n) = self.nodes.get(&id) {
                stack.extend(self.children_ids(n).rev());
            }
        }
        out
    }

    /// `node` and its ancestors, nearest first (empty for `None`).
    pub(crate) fn chain(&self, node: Option<NodeId>) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut cur = node.filter(|n| self.nodes.contains_key(n));
        while let Some(x) = cur {
            out.push(x);
            cur = self.nodes.get(&x).and_then(|n| self.parent_id(n));
        }
        out
    }

    /// The child indices from the root down to `id`: ordering paths orders
    /// nodes in tree order.
    pub(crate) fn path(&self, id: NodeId) -> Vec<usize> {
        let mut out = Vec::new();
        let Some(mut cur) = self.nodes.index_of(id) else {
            return out;
        };
        while let Some(parent) = self.nodes.links(cur).parent {
            let at = self
                .nodes
                .links(parent)
                .children
                .iter()
                .position(|child| child == cur);
            out.push(at.unwrap_or(0));
            cur = parent;
        }
        out.reverse();
        out
    }

    /// Whether `node` is `ancestor` or lies below it.
    pub(crate) fn is_in_subtree(&self, mut node: NodeId, ancestor: NodeId) -> bool {
        loop {
            if node == ancestor {
                return true;
            }
            match self.nodes.get(&node).and_then(|n| self.parent_id(n)) {
                Some(p) => node = p,
                None => return false,
            }
        }
    }

    /// Attached to the root, and neither `id` nor an ancestor is hidden or
    /// leaving: what takes part in hit testing, focus, accessibility and
    /// the modal scope.
    pub(crate) fn shown(&self, id: NodeId) -> bool {
        self.attached_where(id, |n| !n.hidden() && n.presence != Presence::Leaving)
    }

    /// Attached to the root, and neither `id` nor an ancestor is hidden:
    /// what is drawn (a leaving subtree is drawn until its exit ends).
    pub(crate) fn drawn(&self, id: NodeId) -> bool {
        self.attached_where(id, |n| !n.hidden())
    }

    /// Attached to the root, hidden or not.
    pub(crate) fn attached(&self, id: NodeId) -> bool {
        self.attached_where(id, |_| true)
    }

    /// `id` and every node below it, in pre-order.
    pub(crate) fn subtree(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![id];
        while let Some(x) = stack.pop() {
            let Some(n) = self.nodes.get(&x) else {
                continue;
            };
            out.push(x);
            stack.extend(self.children_ids(n).rev());
        }
        out
    }

    /// A node of the tree that is not leaving: what the pointer, the focus
    /// and the reports may still hold.
    pub(crate) fn live(&self, id: NodeId) -> bool {
        self.nodes
            .get(&id)
            .is_some_and(|n| n.presence != Presence::Leaving)
    }

    fn attached_where(&self, mut id: NodeId, keep: impl Fn(&HostNode) -> bool) -> bool {
        loop {
            let Some(n) = self.nodes.get(&id) else {
                return false;
            };
            if !keep(n) {
                return false;
            }
            match self.parent_id(n) {
                Some(p) => id = p,
                None => return self.root == Some(id),
            }
        }
    }

    /// Install a detached node into its reserved, initialized structural slot.
    pub(crate) fn install(&mut self, id: NodeId, node: HostNode, slot: NodeIndex) {
        debug_assert_ne!(id.0, 0, "node zero is the layout viewport");
        if node.kind() == NodeType::Root {
            self.root = Some(id);
        }
        if node.kind() == NodeType::Image {
            self.images.insert(id);
        } else {
            self.images.remove(&id);
        }
        let scrolls = node.scroll_axes() != (false, false);
        let place = node.place;
        self.nodes.install(id, node, slot);
        self.track_scroller(id, scrolls);
        self.track_position(id, place.sticky, place.fixed);
    }

    /// Note whether `id` is sticky or fixed.
    pub(crate) fn track_position(&mut self, id: NodeId, sticky: bool, fixed: bool) {
        for (set, on) in [(&mut self.sticky, sticky), (&mut self.fixed, fixed)] {
            if on {
                set.insert(id);
            } else {
                set.remove(&id);
            }
        }
    }

    pub(crate) fn track_scroller(&mut self, id: NodeId, scrolls: bool) {
        if scrolls {
            self.scrollers.insert(id);
        } else {
            self.scrollers.remove(&id);
        }
    }

    /// Observe `id`'s layout while `on`; `false` when nothing changed.
    pub(crate) fn set_observed(&mut self, id: NodeId, on: bool) -> bool {
        let Some(n) = self.nodes.get_mut(&id) else {
            return false;
        };
        if n.observed == on {
            return false;
        }
        n.observed = on;
        n.sent_layout = None;
        if on {
            self.observed.insert(id);
        } else {
            self.observed.remove(&id);
        }
        true
    }

    /// Link the detached `id` under `parent` before `before` (appended
    /// without it); returns Some when both nodes exist.
    pub(crate) fn insert(
        &mut self,
        parent: NodeId,
        id: NodeId,
        before: Option<NodeId>,
    ) -> Option<()> {
        let child = self.nodes.index_of(id)?;
        let parent_index = self.nodes.index_of(parent)?;
        let before_index = before.and_then(|before| self.nodes.index_of(before));
        let p = self.nodes.links_mut(parent_index);
        p.children.insert(child, before_index);
        if p.children.is_editing() {
            self.pending_child_order.insert(parent);
        }
        self.nodes.links_mut(child).parent = Some(parent_index);
        let c = self.nodes.at_mut(child);
        let (overlay, z) = (c.kind() == NodeType::Overlay, c.place.has_z());
        if z {
            self.nodes.at_mut(parent_index).place.stacks = true;
        }
        if overlay {
            self.overlays.push(id);
        }
        Some(())
    }

    /// Order the children of `parent` by z-index from now on (one of them
    /// has one).
    pub(crate) fn stacks(&mut self, parent: NodeId) {
        if let Some(p) = self.nodes.get_mut(&parent) {
            p.place.stacks = true;
        }
    }

    /// Unlink `id` from its parent (the node stays); returns the parent.
    pub(crate) fn detach(&mut self, id: NodeId) -> Option<NodeId> {
        let index = self.nodes.index_of(id)?;
        let parent = self.nodes.links(index).parent?;
        let children = &mut self.nodes.links_mut(parent).children;
        children.remove(index);
        if children.is_editing() {
            self.pending_child_order.insert(self.nodes.id_at(parent));
        }
        self.nodes.links_mut(index).parent = None;
        self.overlays.retain(|overlay| *overlay != id);
        Some(self.nodes.id_at(parent))
    }

    /// Pack the one active sibling order before layout or a revision boundary.
    /// Work names logical parents so removed slots cannot redirect queued work.
    pub(crate) fn pack_child_orders(&mut self) {
        for id in std::mem::take(&mut self.pending_child_order) {
            if let Some(index) = self.nodes.index_of(id) {
                self.nodes.links_mut(index).children.pack();
            }
        }
    }

    /// Remove one node at a time, preparing its upper payload before slot reuse.
    /// The callback returns an opaque payload destroyed after this node's index
    /// cleanup and before the next descendant, matching structural removal order.
    pub(crate) fn remove_with<R>(
        &mut self,
        id: NodeId,
        mut prepare: impl FnMut(NodeIndex, HostNode) -> R,
    ) -> Vec<NodeId> {
        self.detach(id);
        let mut gone = Vec::new();
        let Some(index) = self.nodes.index_of(id) else {
            return gone;
        };
        let mut stack = vec![index];
        while let Some(index) = stack.pop() {
            let x = self.nodes.id_at(index);
            if let Some((slot, node, links)) = self.nodes.take(x) {
                let payload = prepare(slot, node);
                self.nodes.release(slot);
                stack.extend(links.children);
                self.pending_child_order.remove(&x);
                self.overlays.retain(|o| *o != x);
                self.scrollers.remove(&x);
                self.observed.remove(&x);
                self.images.remove(&x);
                self.sticky.remove(&x);
                self.fixed.remove(&x);
                if self.root == Some(x) {
                    self.root = None;
                }
                gone.push(x);
                drop(payload);
            }
        }
        gone
    }
}
