//! Role: Semantics trees and updates for platform accessibility APIs.
//! Trees must contain unique ids and exactly one rooted hierarchy.
//!
//! Bounds are laid out in content space: where a node would be if no
//! ancestor were scrolled. A scrolling node carries its offset (`scroll`)
//! and platforms move its descendants back by it, so a scroll changes one
//! node, not every node below it. A transformed node (CSS `transform`)
//! carries its own transform the same way: it maps the node's bounds and
//! everything below it, so changing it changes one node.

use crate::geometry::Transform;
use crate::geometry::{Point, Rect};
use crate::ids::SemanticsId;
use crate::semantics::Semantics;

/// One accessibility node, usable on any thread.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticsNode {
    /// Stable node identity.
    pub id: SemanticsId,
    /// Children in reading order.
    pub children: Vec<SemanticsId>,
    /// Canonical accessible identity, state, relations and text properties.
    pub semantics: Semantics<SemanticsId>,
    /// Logical bounds in the window as laid out before the scroll offsets
    /// of its ancestors (see the module documentation). `None` means the
    /// node has no position.
    pub bounds: Option<Rect>,
    /// How far this node scrolls its content: its descendants are shown
    /// moved back by it. Zero for a node that does not scroll.
    pub scroll: Point,
    /// The node's own transform in content space (identity for most
    /// nodes): its bounds and its descendants (moved back by its `scroll`
    /// first) are shown mapped by it, then by its ancestors' scrolls and
    /// transforms. See [`SemanticsTree::window_bounds`].
    pub transform: Transform,
}

/// Complete accessibility tree, usable on any thread.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticsTree {
    /// Root node identity.
    pub root: SemanticsId,
    /// Focused node, or None for window focus.
    pub focus: Option<SemanticsId>,
    /// All nodes; each id occurs exactly once.
    pub nodes: Vec<SemanticsNode>,
}

impl SemanticsTree {
    /// Where node `id` is shown in the window, as platforms report it: the
    /// axis-aligned bounds of its content-space `bounds` mapped by its own
    /// transform, then by each ancestor's scroll (moved back) and
    /// transform, nearest first. `None` when `id` is not in the tree or has
    /// no bounds.
    pub fn window_bounds(&self, id: SemanticsId) -> Option<Rect> {
        let node = self.nodes.iter().find(|n| n.id == id)?;
        self.window_rect(id, node.bounds?)
    }

    /// Map a rectangle from `id`'s content space to logical window bounds,
    /// applying its transform and every ancestor's scroll and transform.
    /// Returns `None` when the node is absent.
    pub fn window_rect(&self, id: SemanticsId, rect: Rect) -> Option<Rect> {
        let node = self.nodes.iter().find(|n| n.id == id)?;
        let mut map = node.transform;
        let mut child = id;
        while let Some(parent) = self.nodes.iter().find(|n| n.children.contains(&child)) {
            map = map
                .then(Transform::translate(-parent.scroll.x, -parent.scroll.y))
                .then(parent.transform);
            child = parent.id;
        }
        Some(map.bounds(rect))
    }
}

/// The nodes of a semantics tree that may have changed since the previous
/// update; every other node stays as it was. A node no longer listed as a
/// child of any node leaves the tree with its subtree.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticsUpdate {
    /// Root node identity.
    pub root: SemanticsId,
    /// Focused node, or None for window focus.
    pub focus: Option<SemanticsId>,
    /// New or possibly changed nodes; each id occurs at most once.
    pub nodes: Vec<SemanticsNode>,
}
