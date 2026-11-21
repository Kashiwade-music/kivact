//! Role: Derive semantic hierarchy and boxless bounds from retained nodes.
use crate::contract::NodeType;
use crate::model::Model;
use crate::semantic_role;
use crate::tree::node::Positioning;
use kivact_types::{NodeId, Point, Rect};

impl Model {
    /// A fragment: a boxless group (`display: contents`) with neither a role
    /// nor a name. The accessibility tree does not publish it, as a browser
    /// leaves out a `display: contents` element without a role.
    pub(super) fn is_fragment(&self, id: NodeId) -> bool {
        self.node(id).is_some_and(|n| {
            n.is_transparent()
                && semantic_role(n, self.slots.editors[n.index.0].is_some()).is_none()
                && n.name().is_none_or(str::is_empty)
        })
    }

    /// The published children of `id`: its logical children, with each
    /// fragment replaced by its own published children (so they belong to the
    /// nearest published ancestor).
    pub(super) fn semantic_children(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack: Vec<_> = self.tree().logical_children(id).into_iter().rev().collect();
        while let Some(child) = stack.pop() {
            if self.is_fragment(child) {
                stack.extend(self.tree().logical_children(child).into_iter().rev());
            } else {
                out.push(child);
            }
        }
        out
    }

    /// The published parent of `id`: its tree parent, or the nearest
    /// ancestor that is published.
    pub(super) fn semantic_parent(&self, id: NodeId) -> Option<NodeId> {
        let mut at = self.tree().get(id).and_then(|n| self.tree().parent_id(n));
        while let Some(parent) = at.filter(|parent| self.is_fragment(*parent)) {
            at = self
                .tree()
                .get(parent)
                .and_then(|n| self.tree().parent_id(n));
        }
        at
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
