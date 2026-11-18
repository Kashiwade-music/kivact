//! Role: Layout pass: a root's borrowed columns as taffy traits, and leaf measurement.
//!
//! The pass exists only while one root is computed; see `tree` for the invariants.

use super::rounding::rounded;
use crate::calcs::Calcs;
use crate::columns::SlotMap;
use crate::style::node::{NodeStyle, TStyle};
use crate::traversal::{Entry, Traversal, entry};
use crate::{Flow, GeometryChange, GeometryChangeKind, LayoutTree, LeafMeasurer, NodeLayoutState};
use kivact_types::{Baseline, Orientation};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use taffy::{
    CacheTree, CoreStyle, Layout, LayoutFlexboxContainer, LayoutInput, LayoutOutput, NodeId as TId,
    ResolveOrZero, TraversePartialTree, TraverseTree,
};

pub(super) struct LayoutPass<'a, Tree: LayoutTree + ?Sized> {
    pub(super) tree: &'a Tree,
    pub(super) traversal: Traversal<'a, Tree>,
    pub(super) calcs: &'a Calcs,
    pub(super) no_grid: &'a TStyle,
    pub(super) records: &'a mut [NodeLayoutState],
    pub(super) styles: &'a [Rc<NodeStyle>],
    /// The styles of the size-animated items as their parents see them,
    /// scaled along their axis; every other item's is its own style.
    pub(super) item_styles: &'a BTreeMap<usize, NodeStyle>,
    pub(super) presence_scale: &'a [Option<(Orientation, f64)>],
    pub(super) relative: &'a [bool],
    pub(super) natural_size: &'a mut [Option<taffy::Size<f32>>],
    pub(super) caches: &'a mut [taffy::Cache],
    pub(super) unrounded: &'a mut [Layout],
    pub(super) final_layout: &'a mut [Option<Layout>],
    pub(super) geometry_changed: &'a mut Vec<GeometryChange>,
    pub(super) performed: &'a mut SlotMap<LayoutInput>,
    pub(super) touched: &'a mut [u32],
    pub(super) below: &'a mut [u32],
    pub(super) origin: &'a mut [Option<(f32, f32)>],
    pub(super) stamp: u32,
    pub(super) never_rounded: &'a mut BTreeSet<usize>,
    /// The kept grids of the grid containers with an absolute child (`root`).
    pub(super) grid_infos: &'a mut BTreeMap<usize, taffy::DetailedLayoutInfo<String>>,
    pub(super) measure: &'a mut dyn LeafMeasurer,
}

impl<Tree: LayoutTree + ?Sized> LayoutPass<'_, Tree> {
    /// The style of `t` as its parent lays it out.
    #[inline(always)]
    pub(super) fn item_style(&self, t: TId) -> &NodeStyle {
        let index = usize::from(t);
        self.item_styles.get(&index).unwrap_or(&*self.styles[index])
    }

    #[inline(never)]
    pub(super) fn leaf(&mut self, t: TId, inputs: LayoutInput) -> LayoutOutput {
        let index = usize::from(t);
        let Self {
            tree,
            measure,
            styles,
            calcs,
            records,
            ..
        } = self;
        let style: &NodeStyle = &styles[index];
        let resolve = |handle, basis| calcs.resolve(handle, basis);
        let mut baseline = None;
        let mut out = taffy::compute_leaf_layout(inputs, style, resolve, |known, avail| {
            let slot = match entry(t) {
                Entry::Viewport => return taffy::Size::ZERO,
                Entry::Slot(slot) => slot,
            };
            let ask = crate::columns::Ask::new(known, avail);
            let id = tree.id_at(slot).expect("invariant: layout node exists");
            let record = &mut records[index].measured;
            let cached = record
                .sizes
                .iter()
                .find(|(r, _)| *r == ask)
                .map(|(_, m)| m.leaf());
            let leaf = cached.unwrap_or_else(|| {
                let leaf = measure.measure(id, ask.request());
                if record.sizes.len() == 16 {
                    record.sizes.remove(0);
                    record.state = crate::columns::RecordState::Incomplete;
                }
                let measured = crate::columns::Measured::of(&leaf);
                // A leaf is measured under a few requests: keep no spare capacity.
                record.sizes.reserve_exact(1);
                record.sizes.push((ask, measured));
                measured.leaf()
            });
            baseline = leaf.baseline;
            taffy::Size {
                width: known.width.unwrap_or(leaf.size.width as f32),
                height: known.height.unwrap_or(leaf.size.height as f32),
            }
        });
        if let Some(baseline) = baseline {
            let width = inputs.parent_size.width;
            let padding = style.padding().resolve_or_zero(width, resolve);
            let border = style.border().resolve_or_zero(width, resolve);
            let top = padding.top + border.top;
            let content = out.size.height - top - padding.bottom - border.bottom;
            out.baselines.first = Some(match baseline {
                Baseline::FromTop(y) => top + y as f32,
                Baseline::FromCenter(y) => top + content.max(0.0) / 2.0 + y as f32,
            });
        }
        out
    }

    /// Round the subtrees whose unrounded layout or position changed in this
    /// pass; the others keep their final layout (taffy's `round_layout`, but
    /// only where something changed).
    pub(super) fn round_changed(&mut self, node: TId, cum: (f32, f32), stamp: u32) {
        let i = usize::from(node);
        let same_origin = self.origin[i] == Some(cum);
        if same_origin && self.below[i] != stamp {
            return;
        }
        let unrounded = self.unrounded[i];
        if !same_origin || self.touched[i] == stamp {
            let layout = rounded(unrounded, cum);
            self.set_final_layout(node, &layout);
            self.origin[i] = Some(cum);
            self.never_rounded.remove(&i);
        }
        let child_cum = (cum.0 + unrounded.location.x, cum.1 + unrounded.location.y);
        for index in 0..self.child_count(node) {
            let child = self.get_child_id(node, index);
            self.round_changed(child, child_cum, stamp);
        }
    }

    /// Round `node` and every descendant (taffy's `round_layout`): the
    /// fallback for a subtree that no changed path reached because a cached
    /// ancestor skipped it, so its nodes never got a rounded layout.
    pub(super) fn round_all(&mut self, node: TId, cum: (f32, f32)) {
        let i = usize::from(node);
        let unrounded = self.unrounded[i];
        let layout = rounded(unrounded, cum);
        self.set_final_layout(node, &layout);
        self.origin[i] = Some(cum);
        self.never_rounded.remove(&i);
        let child_cum = (cum.0 + unrounded.location.x, cum.1 + unrounded.location.y);
        for index in 0..self.child_count(node) {
            let child = self.get_child_id(node, index);
            self.round_all(child, child_cum);
        }
    }

    /// The layout parent of taffy index `i` (`None`: a layout root or detached).
    fn layout_parent(&self, i: usize) -> Option<usize> {
        let slot = i.checked_sub(1)?;
        let id = self.tree.id_at(slot)?;
        match self.tree.flow(slot) {
            Flow::Overlay => None,
            Flow::Fixed => self.tree.parent(slot).map(|_| 0),
            Flow::Normal if self.tree.root() == Some(id) => None,
            Flow::Normal => self.tree.parent(slot).map(|p| p + 1),
        }
    }

    /// The topmost never-rounded slot above (or at) `i` and its parent's cumulative
    /// origin, or `None` when that slot is detached.
    pub(super) fn unrounded_top(&self, i: usize) -> Option<(TId, (f32, f32))> {
        let mut top = i;
        loop {
            match self.layout_parent(top) {
                Some(p) => match self.origin[p] {
                    None => top = p,
                    Some((ox, oy)) => {
                        let l = self.unrounded[p].location;
                        return Some((TId::from(top), (ox + l.x, oy + l.y)));
                    }
                },
                None => {
                    // A layout root: the viewport, the main root or an
                    // attached overlay (a detached overlay is not laid out).
                    let id = top.checked_sub(1).and_then(|s| self.tree.id_at(s));
                    let root = top == 0
                        || id.is_some_and(|id| {
                            self.tree.root() == Some(id) || self.tree.overlays().contains(&id)
                        });
                    return root.then(|| (TId::from(top), (0.0, 0.0)));
                }
            }
        }
    }

    /// Whether taffy index `i` lies at or below `root` in the layout tree.
    pub(super) fn within(&self, mut i: usize, root: usize) -> bool {
        loop {
            if i == root {
                return true;
            }
            match self.layout_parent(i) {
                Some(p) => i = p,
                None => return false,
            }
        }
    }
}

impl<Tree: LayoutTree + ?Sized> TraversePartialTree for LayoutPass<'_, Tree> {
    type ChildIter<'a>
        = crate::traversal::LayoutChildren<'a, Tree>
    where
        Self: 'a;
    #[inline(always)]
    fn child_ids(&self, parent: TId) -> Self::ChildIter<'_> {
        self.traversal.child_ids(parent)
    }
    #[inline(always)]
    fn child_count(&self, parent: TId) -> usize {
        self.traversal.child_count(parent)
    }
    #[inline(always)]
    fn get_child_id(&self, parent: TId, index: usize) -> TId {
        self.traversal.get_child_id(parent, index)
    }
}
impl<Tree: LayoutTree + ?Sized> TraverseTree for LayoutPass<'_, Tree> {}

impl<Tree: LayoutTree + ?Sized> CacheTree for LayoutPass<'_, Tree> {
    #[inline(always)]
    fn cache_get(&mut self, t: TId, inputs: &LayoutInput) -> Option<LayoutOutput> {
        self.caches[usize::from(t)].get(inputs)
    }

    #[inline(always)]
    fn cache_store(&mut self, t: TId, inputs: &LayoutInput, output: LayoutOutput) {
        self.caches[usize::from(t)].store(inputs, output);
    }

    #[inline(always)]
    fn cache_clear(&mut self, t: TId) {
        self.caches[usize::from(t)].clear();
    }
}

impl<Tree: LayoutTree + ?Sized> LayoutFlexboxContainer for LayoutPass<'_, Tree> {
    type FlexboxContainerStyle<'a>
        = &'a NodeStyle
    where
        Self: 'a;

    type FlexboxItemStyle<'a>
        = &'a NodeStyle
    where
        Self: 'a;

    #[inline(always)]
    fn get_flexbox_container_style(&self, t: TId) -> Self::FlexboxContainerStyle<'_> {
        &*self.styles[usize::from(t)]
    }

    #[inline(always)]
    fn get_flexbox_child_style(&self, t: TId) -> Self::FlexboxItemStyle<'_> {
        self.item_style(t)
    }
}

impl<Tree: LayoutTree + ?Sized> LayoutPass<'_, Tree> {
    /// Store the rounded layout of `t` and report how its geometry changed.
    #[inline(always)]
    fn set_final_layout(&mut self, t: TId, layout: &Layout) {
        let index = usize::from(t);
        let before = self.final_layout[index].map(|mut before| {
            // Taffy order is traversal bookkeeping, not rounded geometry.
            before.order = layout.order;
            before
        });
        if let Entry::Slot(slot) = entry(t)
            && before != Some(*layout)
        {
            let local = before.map(|mut before| {
                before.location = layout.location;
                before
            });
            let kind = if local != Some(*layout) {
                GeometryChangeKind::Local
            } else {
                GeometryChangeKind::PositionOnly
            };
            self.geometry_changed.push(GeometryChange {
                id: self
                    .tree
                    .id_at(slot)
                    .expect("invariant: layout node exists"),
                kind,
            });
        }
        self.final_layout[index] = Some(*layout);
    }
}
