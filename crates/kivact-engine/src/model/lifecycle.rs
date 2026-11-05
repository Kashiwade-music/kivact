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
            self.measure.push();
            self.paint.push_slot();
            self.editors.push(None);
        }
        self.tree.install(id, HostNode::new(kind, index), index);
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
// xtask-allow(state-fields): Removed resources are independent optional node payloads collected only to preserve destruction order.
#[expect(
    dead_code,
    reason = "the fields are held only to be dropped, in declaration order"
)]
struct RemovedPayload {
    node: HostNode,
    editor: Option<Box<crate::measure::editor::EditorNode>>,
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
