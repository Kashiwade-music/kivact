//! Role: Derive logical semantic order and boxless group geometry from retained nodes.
use crate::model::Model;
use kivact_types::{NodeId, Rect};

impl Model {
    pub(super) fn semantic_children(&self, id: NodeId) -> Vec<NodeId> {
        self.tree().logical_children(id)
    }

    pub(super) fn semantic_order(&self) -> Vec<NodeId> {
        let mut order = Vec::new();
        let mut stack: Vec<_> = self.tree().root().into_iter().collect();
        while let Some(id) = stack.pop() {
            order.push(id);
            stack.extend(self.semantic_children(id).into_iter().rev());
        }
        order
    }

    /// Untransformed geometry for native accessibility, including boxless groups.
    pub(super) fn semantic_rect(&self, id: NodeId) -> Option<Rect> {
        self.semantic_geometry(id, Self::rect)
    }

    /// Window geometry for the public driver, matching the published semantic group.
    #[cfg(feature = "testing")]
    pub(crate) fn semantic_bounds(&self, id: NodeId) -> Option<Rect> {
        self.semantic_geometry(id, Self::bounds)
    }

    fn semantic_geometry(
        &self,
        id: NodeId,
        rect: fn(&Self, NodeId) -> Option<Rect>,
    ) -> Option<Rect> {
        let node = self.node(id)?;
        if !node.is_transparent() {
            return rect(self, id);
        }
        self.tree()
            .flat_children(id)
            .filter(|child| self.tree().shown(*child))
            .filter_map(|child| rect(self, child))
            .reduce(|left, right| left.union(right))
    }

    pub(super) fn invalidate_boxless_ancestors(
        &self,
        dirty: &mut std::collections::BTreeSet<NodeId>,
    ) {
        for id in dirty.clone() {
            for ancestor in self.tree().chain(Some(id)).into_iter().skip(1) {
                if self
                    .node(ancestor)
                    .is_some_and(|node| node.is_transparent())
                {
                    dirty.insert(ancestor);
                }
            }
        }
    }
}
