//! Role: Adapts the authoritative engine tree and resource measurer to layout.

use crate::tree::NodeTree;
use crate::tree::node::NodeIndex;
use kivact_layout::{Flow, LayoutTree};
use kivact_wire::{LayoutProps, NodeId, NodeType};

impl LayoutTree for NodeTree {
    #[inline]
    fn root(&self) -> Option<NodeId> {
        self.root
    }
    #[inline]
    fn overlays(&self) -> &[NodeId] {
        &self.overlays
    }
    #[inline]
    fn fixed(&self) -> &std::collections::BTreeSet<NodeId> {
        &self.fixed
    }
    #[inline]
    fn index_of(&self, id: NodeId) -> Option<usize> {
        self.nodes.index_of(id).map(|i| i.0)
    }
    #[inline]
    fn id_at(&self, slot: usize) -> Option<NodeId> {
        self.nodes.slot_id(slot)
    }
    #[inline]
    fn parent(&self, slot: usize) -> Option<usize> {
        self.nodes.links(NodeIndex(slot)).parent.map(|i| i.0)
    }
    #[inline]
    fn child_count(&self, slot: usize) -> usize {
        self.nodes.links(NodeIndex(slot)).children.len()
    }
    #[inline]
    fn child_at(&self, slot: usize, raw_child: usize) -> usize {
        self.nodes.links(NodeIndex(slot)).children.as_slice()[raw_child].0
    }
    #[inline]
    fn flow(&self, slot: usize) -> Flow {
        let n = self.nodes.at(NodeIndex(slot));
        if n.kind() == NodeType::Overlay {
            Flow::Overlay
        } else if n.place.fixed {
            Flow::Fixed
        } else {
            Flow::Normal
        }
    }
    #[inline]
    fn layout_props(&self, slot: usize) -> (Option<&LayoutProps>, bool) {
        let n = self.nodes.at(NodeIndex(slot));
        (n.layout(), n.hidden())
    }
    #[inline]
    fn overlay_width_anchor(&self, slot: usize) -> Option<NodeId> {
        let overlay = self.nodes.at(NodeIndex(slot)).overlay()?;
        overlay
            .anchor
            .filter(|_| overlay.at_least_anchor_width && overlay.point.is_none())
    }
}
