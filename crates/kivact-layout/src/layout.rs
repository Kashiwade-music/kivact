//! Role: Owns derived layout columns and their pending computation state.
use crate::{GeometryChanges, LayoutDifference, LayoutTree, LeafMeasurer, NodeLayout};
use crate::{LayoutState, NodeLayoutColumns};
use kivact_types::{NodeId, Orientation, Size};

/// Derived layout and invalidation for one structural slot allocation domain.
#[derive(Clone, Default)]
pub struct Layout {
    columns: NodeLayoutColumns,
    state: LayoutState,
}
impl Layout {
    /// Append the next fresh structural slot, excluding the existing viewport.
    pub fn push(&mut self) {
        self.columns.push();
    }
    /// Reset an allocated zero-based structural slot before reuse.
    pub fn reset(&mut self, slot: usize) {
        self.columns.reset(slot);
    }
    /// Scale one output axis while retaining the natural layout of its children.
    /// The slot must belong to this layout's allocated structural domain.
    pub fn set_presence_scale<T: LayoutTree + ?Sized>(
        &mut self,
        tree: &T,
        slot: usize,
        scale: Option<(Orientation, f64)>,
    ) {
        let current = &mut self.columns.values.presence_scale[slot + 1];
        if *current == scale {
            return;
        }
        *current = scale;
        if scale.is_some() {
            self.columns.values.scaled.insert(slot + 1);
        } else {
            self.columns.values.scaled.remove(&(slot + 1));
        }
        // A changed scale changes the node's output and, through it, its
        // ancestors' cached outputs.
        self.columns
            .clear_caches_through(tree, &mut self.state, slot);
        self.state.changed = true;
    }
    /// Rebuild one live node style from its authoritative properties.
    pub fn rebuild_style<T: LayoutTree + ?Sized>(&mut self, tree: &T, id: NodeId) {
        self.columns.rebuild_style(tree, &mut self.state, id);
    }
    /// Rebuild live styles and discard incremental measurement shortcuts.
    pub fn rebuild_all<T: LayoutTree + ?Sized>(&mut self, tree: &T) {
        self.columns.rebuild_all(tree, &mut self.state);
    }
    /// Expire traversal cursors after authoritative topology changes.
    pub fn topology_changed(&mut self) {
        self.state.topology_changed();
    }
    /// Invalidate a node measurement and its layout dependency chain.
    pub fn invalidate_chain<T: LayoutTree + ?Sized>(&mut self, tree: &T, id: NodeId) {
        self.columns.invalidate_chain(tree, &mut self.state, id);
    }
    /// Invalidate the shared viewport cache for fixed boxes.
    pub fn invalidate_viewport(&mut self) {
        self.columns.invalidate_viewport(&mut self.state);
    }
    /// Compute pending layout work and return whether a computation ran.
    pub fn compute<T: LayoutTree + ?Sized>(
        &mut self,
        tree: &T,
        available: Size,
        measure: &mut dyn LeafMeasurer,
    ) -> bool {
        if !self.state.take_changed() || tree.root().is_none() {
            return false;
        }
        self.columns
            .compute(tree, &mut self.state, available, measure);
        true
    }
    /// Compute one overlay with an optional anchor placement limit.
    pub fn compute_overlay<T: LayoutTree + ?Sized>(
        &mut self,
        tree: &T,
        id: NodeId,
        available: Size,
        limit: Option<Size>,
        measure: &mut dyn LeafMeasurer,
    ) {
        self.columns.next_stamp();
        self.columns
            .compute_overlay(tree, &self.state, id, available, limit, measure);
    }
    /// Drain rounded geometry observations, discarding unread entries on drop.
    pub fn drain_geometry_changes(&mut self) -> GeometryChanges<'_> {
        self.columns.drain_geometry_changes()
    }
    /// Resolve a live identity and read its last rounded layout.
    pub fn computed_layout<T: LayoutTree + ?Sized>(
        &self,
        tree: &T,
        id: NodeId,
    ) -> Option<NodeLayout> {
        self.columns.computed_layout(tree, id)
    }
    /// Read rounded layout at an allocated zero-based structural slot.
    pub fn layout_at(&self, slot: usize) -> Option<NodeLayout> {
        self.columns.layout_at(slot)
    }
    /// Revalidate every recorded request when its measurement record is complete.
    pub fn remeasures_same(&self, slot: usize, id: NodeId, measure: &mut dyn LeafMeasurer) -> bool {
        self.columns.remeasures_same(slot, id, measure)
    }
    /// Compare retained measurements against an independent measurer.
    pub fn compare_measurements(
        &self,
        tree: &dyn LayoutTree,
        measure: &mut dyn LeafMeasurer,
    ) -> Vec<LayoutDifference> {
        self.columns.compare_measurements(tree, measure)
    }
    /// Compare independently derived geometry for the same structural slots.
    pub fn compare(&self, other: &Self, tree: &dyn LayoutTree) -> Vec<LayoutDifference> {
        self.columns.compare(&other.columns, tree)
    }
}
