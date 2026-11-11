//! Role: Tests of incremental rounding against taffy and the reference rebuild.

use super::Layout;
use crate::{Flow, LayoutTree, LeafMeasurer};
use kivact_types::{
    Dimension, Justify, LayoutProps, LeafSize, MeasureRequest, NodeId, OverlayFit, Size,
};
use std::collections::BTreeSet;

/// A root (slot 0, id 1) 100x100 with one child (slot 1, id 2) at margin-left -0.5.
struct Two {
    props: [LayoutProps; 2],
    fixed: BTreeSet<NodeId>,
}

impl Two {
    fn new(child_width: f64) -> Self {
        let root = LayoutProps {
            width: Dimension::Px(100.0),
            height: Dimension::Px(100.0),
            justify: Some(Justify::Center),
            ..Default::default()
        };
        let child = LayoutProps {
            width: Dimension::Px(child_width),
            height: Dimension::Px(10.0),
            shrink: 0.0,
            margin: kivact_types::Edges {
                top: Dimension::Px(0.0),
                right: Dimension::Px(0.0),
                bottom: Dimension::Px(0.0),
                left: Dimension::Px(-0.5),
            },
            ..Default::default()
        };
        Self {
            props: [root, child],
            fixed: BTreeSet::new(),
        }
    }
}

impl LayoutTree for Two {
    fn root(&self) -> Option<NodeId> {
        Some(NodeId(1))
    }
    fn overlays(&self) -> &[NodeId] {
        &[]
    }
    fn fixed(&self) -> &BTreeSet<NodeId> {
        &self.fixed
    }
    fn index_of(&self, id: NodeId) -> Option<usize> {
        (1..=2).contains(&id.0).then(|| id.0 as usize - 1)
    }
    fn id_at(&self, slot: usize) -> Option<NodeId> {
        (slot < 2).then(|| NodeId(slot as u64 + 1))
    }
    fn parent(&self, slot: usize) -> Option<usize> {
        (slot == 1).then_some(0)
    }
    fn child_count(&self, slot: usize) -> usize {
        usize::from(slot == 0)
    }
    fn child_at(&self, _slot: usize, _raw_child: usize) -> usize {
        1
    }
    fn flow(&self, _slot: usize) -> Flow {
        Flow::Normal
    }
    fn layout_props(&self, slot: usize) -> (Option<&LayoutProps>, bool) {
        (Some(&self.props[slot]), false)
    }
    fn overlay_width_anchor(&self, _slot: usize) -> Option<NodeId> {
        None
    }
    fn overlay_fit(&self, _slot: usize) -> Option<OverlayFit> {
        None
    }
}

struct Empty;
impl LeafMeasurer for Empty {
    fn measure(&mut self, _id: NodeId, _request: MeasureRequest) -> LeafSize {
        LeafSize {
            size: Size::new(0.0, 0.0),
            baseline: None,
        }
    }
}

fn laid_out(tree: &Two) -> Layout {
    let mut layout = Layout::default();
    layout.push();
    layout.push();
    layout.rebuild_style(tree, NodeId(1));
    layout.rebuild_style(tree, NodeId(2));
    assert!(layout.compute(tree, Size::new(100.0, 100.0), &mut Empty));
    layout
}

#[test]
fn rounding_matches_taffy_at_negative_half_offsets() {
    // The child sits at x = -0.5 unrounded; taffy rounds with (v + 0.5).floor().
    let tree = Two::new(100.0);
    let layout = laid_out(&tree);
    let child = layout.layout_at(1).expect("child laid out");
    assert_eq!(child.location.x, 0.0);
    assert_eq!(child.size.width, 100.0);
}

#[test]
fn rebuild_all_rounds_every_node_again() {
    let tree = Two::new(50.0);
    let mut layout = laid_out(&tree);
    // A stale rounded layout must be found by the reference rebuild.
    layout.columns.values.final_layout[2] = Some(taffy::Layout::new());
    let mut fresh = layout.clone();
    fresh.rebuild_all(&tree);
    assert!(fresh.compute(&tree, Size::new(100.0, 100.0), &mut Empty));
    assert!(!layout.compare(&fresh, &tree).is_empty());
}

/// Two as above plus slot 2 (id 3): an overlay node with no parent.
struct WithDetachedOverlay(Two);
impl LayoutTree for WithDetachedOverlay {
    fn root(&self) -> Option<NodeId> {
        self.0.root()
    }
    fn overlays(&self) -> &[NodeId] {
        &[]
    }
    fn fixed(&self) -> &BTreeSet<NodeId> {
        self.0.fixed()
    }
    fn index_of(&self, id: NodeId) -> Option<usize> {
        (1..=3).contains(&id.0).then(|| id.0 as usize - 1)
    }
    fn id_at(&self, slot: usize) -> Option<NodeId> {
        (slot < 3).then(|| NodeId(slot as u64 + 1))
    }
    fn parent(&self, slot: usize) -> Option<usize> {
        self.0.parent(slot)
    }
    fn child_count(&self, slot: usize) -> usize {
        self.0.child_count(slot)
    }
    fn child_at(&self, slot: usize, raw: usize) -> usize {
        self.0.child_at(slot, raw)
    }
    fn flow(&self, slot: usize) -> Flow {
        if slot == 2 {
            Flow::Overlay
        } else {
            Flow::Normal
        }
    }
    fn layout_props(&self, slot: usize) -> (Option<&LayoutProps>, bool) {
        (Some(&self.0.props[slot.min(1)]), false)
    }
    fn overlay_width_anchor(&self, _slot: usize) -> Option<NodeId> {
        None
    }
    fn overlay_fit(&self, _slot: usize) -> Option<OverlayFit> {
        None
    }
}

#[test]
fn detached_overlay_gets_no_layout() {
    let tree = WithDetachedOverlay(Two::new(50.0));
    let mut layout = Layout::default();
    for _ in 0..3 {
        layout.push();
    }
    for id in 1..=3 {
        layout.rebuild_style(&tree, NodeId(id));
    }
    assert!(layout.compute(&tree, Size::new(100.0, 100.0), &mut Empty));
    assert_eq!(layout.layout_at(2), None);
}

#[test]
fn wrapped_pass_number_still_rounds_changes() {
    let mut tree = Two::new(50.0);
    let mut layout = laid_out(&tree);
    layout.columns.stamp = u32::MAX;
    let values = &mut layout.columns.values;
    values.touched[..2].fill(7);
    values.below[..2].fill(7);
    values.below[2] = 1;
    tree.props[1].padding.left = Dimension::Px(5.0);
    layout.rebuild_style(&tree, NodeId(2));
    assert!(layout.compute(&tree, Size::new(100.0, 100.0), &mut Empty));
    assert_eq!(
        layout.layout_at(1).expect("child laid out").padding.left,
        5.0
    );
}
