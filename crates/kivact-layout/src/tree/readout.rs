//! Role: Reads a node's last rounded layout as the neutral geometry value.

use crate::{LayoutTree, NodeLayout, NodeLayoutColumns};
use kivact_types::{Insets, NodeId, Point, Rect, Size};

impl NodeLayoutColumns {
    /// Read the last rounded layout at a live zero-based structural slot.
    /// The slot must identify a live node in this columns allocation domain.
    /// Outside capacity or before layout this returns None; it does not resolve
    /// identity or establish whether an allocated slot is currently live.
    #[inline]
    pub(crate) fn layout_at(&self, slot: usize) -> Option<NodeLayout> {
        let l = self
            .values
            .final_layout
            .get(slot.checked_add(1)?)
            .copied()
            .flatten()?;
        let x = l.border.left + l.padding.left;
        let y = l.border.top + l.padding.top;
        Some(NodeLayout {
            location: Point {
                x: f64::from(l.location.x),
                y: f64::from(l.location.y),
            },
            size: Size {
                width: f64::from(l.size.width),
                height: f64::from(l.size.height),
            },
            content_size: Size {
                width: f64::from(l.size.width + l.scroll_width()),
                height: f64::from(l.size.height + l.scroll_height()),
            },
            content_box: Rect {
                x: f64::from(x),
                y: f64::from(y),
                width: f64::from((l.size.width - x - l.border.right - l.padding.right).max(0.0)),
                height: f64::from(
                    (l.size.height - y - l.border.bottom - l.padding.bottom).max(0.0),
                ),
            },
            padding: Insets {
                top: f64::from(l.padding.top),
                right: f64::from(l.padding.right),
                bottom: f64::from(l.padding.bottom),
                left: f64::from(l.padding.left),
            },
            border: Insets {
                top: f64::from(l.border.top),
                right: f64::from(l.border.right),
                bottom: f64::from(l.border.bottom),
                left: f64::from(l.border.left),
            },
            margin: Insets {
                top: f64::from(l.margin.top),
                right: f64::from(l.margin.right),
                bottom: f64::from(l.margin.bottom),
                left: f64::from(l.margin.left),
            },
        })
    }
    /// Resolve a logical identity, then read its rounded layout.
    /// `tree` must use this columns instance's identity-to-slot allocation domain.
    #[inline]
    pub(crate) fn computed_layout<Tree: LayoutTree + ?Sized>(
        &self,
        tree: &Tree,
        id: NodeId,
    ) -> Option<NodeLayout> {
        self.layout_at(tree.index_of(id)?)
    }
}
