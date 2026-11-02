//! Role: Slot storage of host nodes, their logical index and structural links.

use crate::tree::child_order::ChildOrder;
use crate::tree::node::{HostNode, NodeIndex};
use kivact_wire::{IdMap, NodeId};

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
    pub(crate) fn install(&mut self, id: NodeId, node: HostNode, slot: NodeIndex) {
        debug_assert!(!self.by_id.contains_key(&id));
        debug_assert!(self.entries[slot.0].is_none());
        debug_assert_eq!(node.index, slot);
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
