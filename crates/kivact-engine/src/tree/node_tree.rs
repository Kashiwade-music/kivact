//! Role: Own retained node structure and answer ancestry and visibility queries.

use super::store::NodeStore;
use crate::contract::NodeType;
use crate::tree::node::{HostNode, NodeIndex, NodePresence, Place};
use kivact_types::NodeId;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// The node tree.
#[derive(Clone, Debug, Default)]
pub(crate) struct NodeTree {
    flat: RefCell<BTreeMap<NodeId, Rc<[NodeId]>>>,
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
pub(crate) fn states_place(styles: &kivact_types::StateStyles) -> bool {
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

    pub(super) fn children_ids<'a>(
        &'a self,
        node: &'a HostNode,
    ) -> impl DoubleEndedIterator<Item = NodeId> + ExactSizeIterator + 'a {
        self.nodes
            .links(node.index)
            .children
            .iter()
            .map(|index| self.nodes.id_at(index))
    }

    /// Children that contribute boxes, in their logical sibling order.
    pub(crate) fn flat_children(
        &self,
        id: NodeId,
    ) -> impl DoubleEndedIterator<Item = NodeId> + ExactSizeIterator {
        let cached = self.flat.borrow().get(&id).cloned();
        let children = cached.unwrap_or_else(|| {
            let mut children = Vec::new();
            let mut stack = self
                .get(id)
                .map(|n| self.children_ids(n).rev().collect::<Vec<_>>())
                .unwrap_or_default();
            while let Some(child) = stack.pop() {
                let Some(node) = self.get(child) else {
                    continue;
                };
                if node.is_transparent() {
                    if !node.hidden() {
                        stack.extend(self.children_ids(node).rev());
                    }
                } else {
                    children.push(child);
                }
            }
            let children: Rc<[NodeId]> = children.into();
            self.flat.borrow_mut().insert(id, children.clone());
            children
        });
        (0..children.len()).map(move |index| children[index])
    }

    /// Logical children still owned by the client.
    pub(crate) fn logical_children(&self, id: NodeId) -> Vec<NodeId> {
        self.get(id)
            .map(|n| self.children_ids(n).filter(|id| self.live(*id)).collect())
            .unwrap_or_default()
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

    /// Pre-order traversal of boxes participating in visual and input trees.
    pub(crate) fn flat_preorder(&self) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack: Vec<_> = self.root.into_iter().collect();
        while let Some(id) = stack.pop() {
            out.push(id);
            stack.extend(self.flat_children(id).rev());
        }
        out
    }

    /// Nearest parent contributing a box.
    pub(crate) fn flat_parent(&self, node: &HostNode) -> Option<NodeId> {
        let mut parent = self.parent_id(node);
        while let Some(id) = parent {
            let node = self.get(id)?;
            if !node.is_transparent() {
                return Some(id);
            }
            parent = self.parent_id(node);
        }
        None
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
        self.attached_where(id, |n| {
            !n.hidden() && !matches!(n.presence, NodePresence::Leaving(_))
        })
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
            .is_some_and(|n| !matches!(n.presence, NodePresence::Leaving(_)))
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
        self.flat.get_mut().clear();
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

    /// Derive the place of the node at `index` again from its props, keeping the
    /// sticky, fixed and scroller indexes with it; the place it had.
    pub(crate) fn update_place(&mut self, index: NodeIndex) -> Place {
        self.flat.get_mut().clear();
        let id = self.nodes.id_at(index);
        let n = self.nodes.at_mut(index);
        let old = n.place;
        n.update_place();
        let (place, scrolls) = (n.place, n.scroll_axes() != (false, false));
        self.track_scroller(id, scrolls);
        self.track_position(id, place.sticky, place.fixed);
        old
    }

    /// Note whether `id` is sticky or fixed.
    fn track_position(&mut self, id: NodeId, sticky: bool, fixed: bool) {
        for (set, on) in [(&mut self.sticky, sticky), (&mut self.fixed, fixed)] {
            if on {
                set.insert(id);
            } else {
                set.remove(&id);
            }
        }
    }

    fn track_scroller(&mut self, id: NodeId, scrolls: bool) {
        if scrolls {
            self.scrollers.insert(id);
        } else {
            self.scrollers.remove(&id);
        }
    }

    /// Observe `id`'s layout while `on`; `false` when nothing changed.
    pub(crate) fn set_observed(&mut self, id: NodeId, on: bool) -> bool {
        if !self.nodes.contains_key(&id) {
            return false;
        }
        let changed = if on {
            self.observed.insert(id)
        } else {
            self.observed.remove(&id)
        };
        if changed && let Some(n) = self.nodes.get_mut(&id) {
            n.sent_layout = Default::default();
        }
        changed
    }

    /// Link the detached `id` under `parent` before `before` (appended
    /// without it); returns Some when both nodes exist.
    pub(crate) fn insert(
        &mut self,
        parent: NodeId,
        id: NodeId,
        before: Option<NodeId>,
    ) -> Option<()> {
        self.flat.get_mut().clear();
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
            self.nodes.at_mut(parent_index).place.stacks = crate::tree::node::Stacking::Stacks;
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
            p.place.stacks = crate::tree::node::Stacking::Stacks;
        }
    }

    /// Unlink `id` from its parent (the node stays); returns the parent.
    pub(crate) fn detach(&mut self, id: NodeId) -> Option<NodeId> {
        self.flat.get_mut().clear();
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
