//! Role: Derive semantic hierarchy and boxless bounds from retained nodes.
use crate::contract::NodeType;
use crate::model::Model;
use crate::tree::node::Positioning;
use kivact_types::{NodeId, Point, Rect};

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

    /// The content-space box of `id` the accessibility tree publishes: its
    /// box moved by the scrolls above its layer (`layer_offset`). A boxless
    /// group (`display: contents`) takes the union of its shown children's.
    ///
    /// Exact under scrolling: the value has no scroll term that a scroll of
    /// an ancestor below the layer could change, so a payload that a scroll
    /// does not rebuild keeps the value a rebuild computes. (`rect` takes the
    /// scrolls out and this adds them back: `(x - s) + s` is not `x` in floats.)
    pub(super) fn content_bounds(&self, id: NodeId) -> Option<Rect> {
        if !self.node(id)?.is_transparent() {
            return self.placed_content(id);
        }
        self.tree()
            .flat_children(id)
            .filter(|child| self.tree().shown(*child))
            .filter_map(|child| self.placed_content(child))
            .reduce(|left, right| left.union(right))
    }

    fn placed_content(&self, id: NodeId) -> Option<Rect> {
        let rect = self.content_rect(id)?;
        let above = self.layer_offset(id);
        Some(Rect {
            x: rect.x + above.x,
            y: rect.y + above.y,
            ..rect
        })
    }

    /// The box of `id` as `Model::rect` walks it, without the scrolls that
    /// walk takes out (`layer_offset` gives the ones that stay). `None` for a
    /// boxless group, or a node not laid out.
    pub(crate) fn content_rect(&self, id: NodeId) -> Option<Rect> {
        if self.node(id)?.is_transparent() {
            return None;
        }
        let size = self.computed_layout(id)?.size;
        let (mut x, mut y) = (0.0, 0.0);
        let mut cur = id;
        loop {
            let node = self.node(cur)?;
            if node.kind() == NodeType::Overlay {
                let o = self.overlay_origin(cur);
                x += o.x;
                y += o.y;
                break;
            }
            let loc = self.location(node);
            x += loc.x;
            y += loc.y;
            if node.place.positioning == Positioning::Fixed {
                break;
            }
            match self.tree().flat_parent(node) {
                Some(parent) => cur = parent,
                None => break,
            }
        }
        Some(Rect {
            x,
            y,
            width: size.width,
            height: size.height,
        })
    }

    /// The nearest layer (an overlay or a fixed box) at or above `id`.
    fn layer_of(&self, id: NodeId) -> Option<NodeId> {
        let mut cur = id;
        loop {
            let node = self.node(cur)?;
            if node.is_layer() {
                return Some(cur);
            }
            cur = self.tree().flat_parent(node)?;
        }
    }

    /// The scroll offsets that `content_rect` leaves in: those above the
    /// layer of `id` (none without one). A scroll of one of them moves that
    /// layer, and its subtree is built again.
    pub(crate) fn layer_offset(&self, id: NodeId) -> Point {
        match self.layer_of(id) {
            Some(layer) => self.scroll_above(layer),
            None => Point::default(),
        }
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
        let mut seen = std::collections::BTreeSet::new();
        for id in dirty.clone() {
            let mut at = self.tree().get(id).and_then(|n| self.tree().parent_id(n));
            while let Some(ancestor) = at {
                if !seen.insert(ancestor) {
                    break; // it and its ancestors were looked at
                }
                if self
                    .node(ancestor)
                    .is_some_and(|node| node.is_transparent())
                {
                    dirty.insert(ancestor);
                }
                at = self
                    .tree()
                    .get(ancestor)
                    .and_then(|n| self.tree().parent_id(n));
            }
        }
    }
}
