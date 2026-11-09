//! Role: Adapts the authoritative engine tree and resource measurer to layout.

use crate::contract::NodeType;
use crate::tree::NodeTree;
use crate::tree::node::{NodeIndex, Positioning};
use kivact_layout::{Flow, LayoutTree};
use kivact_types::{LayoutProps, NodeId, OverlayAt, OverlayFit};

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
        self.flat_parent(self.nodes.at(NodeIndex(slot)))
            .and_then(|id| self.nodes.index_of(id))
            .map(|index| index.0)
    }
    #[inline]
    fn child_count(&self, slot: usize) -> usize {
        self.flat_children(self.nodes.id_at(NodeIndex(slot))).len()
    }
    #[inline]
    fn child_at(&self, slot: usize, raw_child: usize) -> usize {
        self.nodes
            .index_of(
                self.flat_child(self.nodes.id_at(NodeIndex(slot)), raw_child)
                    .expect("invariant: layout child exists"),
            )
            .expect("invariant: layout child is retained")
            .0
    }
    #[inline]
    fn flow(&self, slot: usize) -> Flow {
        let n = self.nodes.at(NodeIndex(slot));
        if n.kind() == NodeType::Overlay {
            Flow::Overlay
        } else if n.place.positioning == Positioning::Fixed {
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
        match overlay.at {
            OverlayAt::Anchor {
                node,
                at_least_width: true,
            } => node,
            _ => None,
        }
    }

    #[inline]
    fn overlay_fit(&self, slot: usize) -> Option<OverlayFit> {
        self.nodes.at(NodeIndex(slot)).overlay()?.fit
    }
}
