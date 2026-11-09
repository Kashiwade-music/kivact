//! Role: Coordinate structural slots and the Model-owned derived columns.
use crate::contract::NodeKind;
use crate::model::Model;
use crate::tree::node::HostNode;
use kivact_types::NodeId;

impl Model {
    pub(crate) fn create_slot(&mut self, id: NodeId, kind: NodeKind) {
        // Duplicate IDs are a trusted-client violation; never replace/drop a live slot.
        debug_assert!(!self.tree.contains(id), "node ids are unique");
        let (index, fresh) = self.tree.nodes.reserve();
        if fresh {
            self.layout.push();
            self.measure.push(&kind);
            self.paint.push_slot();
            self.editors.push(None);
        } else {
            // A reused slot measures the new node's kind, never the old one's.
            self.measure.set_measure(index.0, &kind, false);
        }
        self.tree.install(id, HostNode::new(kind, index), index);
        if self.tree.root() == Some(id) {
            // A new root is a new set of layer roots.
            self.paint.stale_all_facts();
        }
    }
    pub(crate) fn remove_slots(&mut self, id: NodeId) -> Vec<NodeId> {
        let Self {
            tree,
            layout,
            measure,
            paint,
            editors,
            ..
        } = self;
        tree.remove_with(id, |index, node| {
            let editor = editors[index.0].take();
            let extents = measure.take_slot(index.0);
            // Own user-bearing fields before a reset can fail, as the old
            // removed HostNode did. Slot publication still follows all resets.
            let payload = RemovedPayload {
                node,
                editor,
                extents,
            };
            layout.reset(index.0);
            paint.reset_slot(index.0);
            payload
        })
    }
}

// Declaration order preserves raw props/scenes -> native editor -> text extents.
// Dropped under the existing Model borrow: structural reentry remains OPEN.
#[expect(
    dead_code,
    reason = "the fields are held only to be dropped, in declaration order"
)]
struct RemovedPayload {
    node: HostNode,
    // xtask-allow(state-fields): independent: a value held only to be dropped after the node, in declaration order
    editor: Option<Box<crate::measure::editor::EditorNode>>,
    // xtask-allow(state-fields): independent: a value held only to be dropped after the node, in declaration order
    extents: Option<Box<crate::measure::state::TextExtents>>,
}
impl Drop for Model {
    fn drop(&mut self) {
        let Self {
            tree,
            measure,
            editors,
            ..
        } = self;
        tree.nodes.drain_payloads(|index, node| {
            let extents = measure.take_slot(index.0);
            let editor = editors[index.0].take();
            drop(RemovedPayload {
                node,
                editor,
                extents,
            });
        });
    }
}
