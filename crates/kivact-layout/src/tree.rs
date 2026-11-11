//! Role: Computes retained layouts from borrowed topology and leaf measurements.
//!
//! The layout pass implements taffy's custom tree traits on `LayoutPass`,
//! which exists only while one root is computed: it shares the borrow of
//! the `LayoutTree` and borrows `LeafMeasurer` and the layout columns mutably,
//! and no copy of the tree or shared measuring context outlives it. Each root
//! computation splits the columns once: the styles as a shared slice, the
//! caches and the geometry as mutable slices, indexed directly.
//! The tree parameter preserves its concrete type through traversal so Taffy
//! can specialize at the caller.
//!
//! Invariants:
//! - The root fills the window, then each attached overlay is a root of
//!   its own (in the order attached), then the fixed boxes, laid out as
//!   children of the shared viewport (position zero) in `NodeId` order;
//!   consumers retain their own presentation order independently.
//! - A leaf's measurements are recorded by request (`Record`), in the f32
//!   the layout reads, and taken from the record before the leaf is measured.
//! - A leaf's first baseline is placed inside its padding and border:
//!   from the content top, or from the content centre (a single-line text
//!   input's).
//! - Invalidating a node clears its record and the caches up the chain
//!   until a cache already empty, an overlay (a root of its own), a fixed
//!   box (which clears the viewport's instead), or an absolute box. An
//!   absolute box takes no part in its parent's size (taffy places it after
//!   the in-flow items), so its parent is queued instead
//!   (`LayoutState::relayout`): before the roots, the parent is laid out
//!   again alone with the inputs of its last full layout (the `performed`
//!   column), and its own chain is cleared only when that answers another
//!   size, baseline or reach of its overflow than its cache held. A box that becomes or stops being absolute
//!   clears the chain through itself (its parent's intrinsic sizes change).
//! - Layouts are rounded to whole pixels (`round_changed`: taffy's
//!   `round_layout` only where something changed);
//!   `NodeLayoutColumns::layout_at` reads a node's by its position, and only a
//!   lookup by `NodeId` goes through the store's index.
//! - A box whose own lengths are absolute is cached without its parent's size
//!   (`relative`).

use crate::calcs::CalcAlloc;
use crate::calcs::Calcs;
use crate::columns::{SlotMap, scale_item_styles};
use crate::style::node::{NodeStyle, TStyle};
use crate::traversal::{Entry, Traversal, entry, taffy_id, viewport};
use crate::{
    Flow, GeometryChange, GeometryChangeKind, LayoutState, LayoutTree, LeafMeasurer, NodeLayout,
    NodeLayoutColumns, NodeLayoutState, NodeLayoutValues,
};
use kivact_types::{Baseline, Insets, NodeId, Orientation, Point, Rect, Size};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use taffy::{
    CacheTree, ClearState, CoreStyle, Display, Layout, LayoutFlexboxContainer, LayoutGridContainer,
    LayoutInput, LayoutOutput, LayoutPartialTree, LengthPercentageAuto, NodeId as TId,
    ResolveOrZero, RoundTree, RunMode, TraversePartialTree, TraverseTree,
};

struct LayoutPass<'a, Tree: LayoutTree + ?Sized> {
    tree: &'a Tree,
    traversal: Traversal<'a, Tree>,
    calcs: &'a Calcs,
    no_grid: &'a TStyle,
    records: &'a mut [NodeLayoutState],
    styles: &'a [Rc<NodeStyle>],
    /// The styles of the size-animated items as their parents see them,
    /// scaled along their axis; every other item's is its own style.
    item_styles: &'a BTreeMap<usize, NodeStyle>,
    presence_scale: &'a [Option<(Orientation, f64)>],
    relative: &'a [bool],
    natural_size: &'a mut [Option<taffy::Size<f32>>],
    caches: &'a mut [taffy::Cache],
    unrounded: &'a mut [Layout],
    final_layout: &'a mut [Option<Layout>],
    geometry_changed: &'a mut Vec<GeometryChange>,
    performed: &'a mut SlotMap<LayoutInput>,
    touched: &'a mut [u32],
    below: &'a mut [u32],
    origin: &'a mut [Option<(f32, f32)>],
    stamp: u32,
    never_rounded: &'a mut BTreeSet<usize>,
    measure: &'a mut dyn LeafMeasurer,
}

impl<Tree: LayoutTree + ?Sized> LayoutPass<'_, Tree> {
    /// The style of `t` as its parent lays it out.
    #[inline(always)]
    fn item_style(&self, t: TId) -> &NodeStyle {
        let index = usize::from(t);
        self.item_styles.get(&index).unwrap_or(&*self.styles[index])
    }

    #[inline(never)]
    fn leaf(&mut self, t: TId, inputs: LayoutInput) -> LayoutOutput {
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
    fn round_changed(&mut self, node: TId, cum: (f32, f32), stamp: u32) {
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
    fn round_all(&mut self, node: TId, cum: (f32, f32)) {
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
    fn unrounded_top(&self, i: usize) -> Option<(TId, (f32, f32))> {
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
    fn within(&self, mut i: usize, root: usize) -> bool {
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

/// Whether a node's new layout output reads the same to its parent as the
/// old one: the same size, baselines and margins, and the same reach of
/// its overflow beyond its box (what taffy's
/// `compute_scrollable_overflow_contribution` takes from it), so its new
/// overflow changes only its own scroll range.
fn same_from_outside(before: &LayoutOutput, after: &LayoutOutput) -> bool {
    let reach = |o: &LayoutOutput| {
        let r = o.scrollable_overflow_rect;
        (
            r.left.min(0.0),
            r.top.min(0.0),
            r.right.max(o.size.width),
            r.bottom.max(o.size.height),
        )
    };
    LayoutOutput {
        scrollable_overflow_rect: before.scrollable_overflow_rect,
        ..*after
    } == *before
        && reach(before) == reach(after)
}

/// Record that column `index` got a new unrounded layout in pass `stamp`,
/// and that every ancestor (up to the viewport, column 0) has such a node below it.
fn touch(
    tree: &(impl LayoutTree + ?Sized),
    touched: &mut [u32],
    below: &mut [u32],
    stamp: u32,
    index: usize,
) {
    touched[index] = stamp;
    let mut at = Some(index);
    while let Some(i) = at {
        if below[i] == stamp {
            break;
        }
        below[i] = stamp;
        at = if i == 0 {
            None
        } else {
            Some(tree.parent(i - 1).map_or(0, |p| p + 1))
        };
    }
}

/// The rounded copy of `unrounded`, whose parent's cumulative origin is `cum`
/// (the `layout` part of taffy 0.14's `round_layout_inner`, and its
/// `round_scrollable_overflow_rect`: the same arithmetic in the same order,
/// with taffy's `round`).
fn rounded(unrounded: Layout, cum: (f32, f32)) -> Layout {
    let mut layout = unrounded;
    let x = cum.0 + unrounded.location.x;
    let y = cum.1 + unrounded.location.y;
    layout.location.x = round(unrounded.location.x);
    layout.location.y = round(unrounded.location.y);
    layout.size.width = round(x + unrounded.size.width) - round(x);
    layout.size.height = round(y + unrounded.size.height) - round(y);
    layout.scrollbar_size.width = round(unrounded.scrollbar_size.width);
    layout.scrollbar_size.height = round(unrounded.scrollbar_size.height);
    layout.border.left = round(x + unrounded.border.left) - round(x);
    layout.border.right =
        round(x + unrounded.size.width) - round(x + unrounded.size.width - unrounded.border.right);
    layout.border.top = round(y + unrounded.border.top) - round(y);
    layout.border.bottom = round(y + unrounded.size.height)
        - round(y + unrounded.size.height - unrounded.border.bottom);
    layout.padding.left = round(x + unrounded.padding.left) - round(x);
    layout.padding.right =
        round(x + unrounded.size.width) - round(x + unrounded.size.width - unrounded.padding.right);
    layout.padding.top = round(y + unrounded.padding.top) - round(y);
    layout.padding.bottom = round(y + unrounded.size.height)
        - round(y + unrounded.size.height - unrounded.padding.bottom);
    let overflow = unrounded.scrollable_overflow_rect;
    layout.scrollable_overflow_rect.left = round(x + overflow.left) - round(x);
    layout.scrollable_overflow_rect.right = round(x + overflow.right) - round(x);
    layout.scrollable_overflow_rect.top = round(y + overflow.top) - round(y);
    layout.scrollable_overflow_rect.bottom = round(y + overflow.bottom) - round(y);
    layout
}

/// Taffy's rounding (`(value + 0.5).floor()`, not `f32::round`): halves
/// round toward positive infinity, also below zero.
#[inline(always)]
fn round(value: f32) -> f32 {
    (value + 0.5).floor()
}

fn relative_length(raw: taffy::style::CompactLength) -> bool {
    use taffy::style::CompactLength;
    matches!(
        raw.tag(),
        CompactLength::PERCENT_TAG | CompactLength::CALC_TAG
    )
}

/// Whether `s` resolves a length of its own box against its parent's size.
fn uses_parent_size(s: &NodeStyle) -> bool {
    let d = |v: taffy::Dimension| relative_length(v.into_raw());
    let a = |v: taffy::LengthPercentageAuto| relative_length(v.into_raw());
    let p = |v: taffy::LengthPercentage| relative_length(v.into_raw());
    d(s.size.width)
        || d(s.size.height)
        || d(s.flex_basis)
        || a(s.min_size.width)
        || a(s.min_size.height)
        || a(s.max_size.width)
        || a(s.max_size.height)
        || [
            s.margin.left,
            s.margin.right,
            s.margin.top,
            s.margin.bottom,
            s.inset.left,
            s.inset.right,
            s.inset.top,
            s.inset.bottom,
        ]
        .into_iter()
        .any(a)
        || [
            s.padding.left,
            s.padding.right,
            s.padding.top,
            s.padding.bottom,
            s.border.left,
            s.border.right,
            s.border.top,
            s.border.bottom,
        ]
        .into_iter()
        .any(p)
}

/// `inputs` with the parent's size left out unless the box is `relative`: a box
/// whose own lengths are absolute lays out the same at any parent size, and taffy
/// keys its caches by the parent's size.
fn keep_parent_size(relative: bool, inputs: LayoutInput) -> LayoutInput {
    if relative {
        inputs
    } else {
        LayoutInput {
            parent_size: taffy::Size::NONE,
            ..inputs
        }
    }
}

fn resolve_dimension(value: taffy::Dimension, basis: Option<f32>, calcs: &Calcs) -> Option<f32> {
    use taffy::style::CompactLength;
    match value.tag() {
        CompactLength::LENGTH_TAG => Some(value.value()),
        CompactLength::PERCENT_TAG => basis.map(|basis| basis * value.value()),
        CompactLength::CALC_TAG => {
            basis.map(|basis| calcs.resolve(value.into_raw().calc_value(), basis))
        }
        _ => None,
    }
}

impl<Tree: LayoutTree + ?Sized> LayoutPartialTree for LayoutPass<'_, Tree> {
    type CoreContainerStyle<'a>
        = &'a NodeStyle
    where
        Self: 'a;

    type CustomIdent = String;

    #[inline(always)]
    fn get_core_container_style(&self, t: TId) -> Self::CoreContainerStyle<'_> {
        &*self.styles[usize::from(t)]
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, t: TId, layout: &Layout) {
        let index = usize::from(t);
        // Taffy sets every child again when it lays out the parent: mark only a
        // new value, or a slot that was never rounded (its origin is `None`).
        let changed = self.unrounded[index] != *layout || self.origin[index].is_none();
        self.unrounded[index] = *layout;
        if changed {
            touch(self.tree, self.touched, self.below, self.stamp, index);
        }
    }

    #[inline(always)]
    fn resolve_calc_value(&self, handle: *const (), basis: f32) -> f32 {
        self.calcs.resolve(handle, basis)
    }

    fn compute_child_layout(&mut self, t: TId, inputs: LayoutInput) -> LayoutOutput {
        // Under a `display: none` ancestor.
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            return taffy::compute_hidden_layout(self, t);
        }
        let inputs = keep_parent_size(self.relative[usize::from(t)], inputs);
        let output = taffy::compute_cached_layout(self, t, inputs, |pass, t, inputs| {
            let scale = pass.presence_scale[usize::from(t)];
            let mut natural_inputs = inputs;
            let mut natural = pass.natural_size[usize::from(t)];
            if let Some((axis, _)) = scale {
                let style: &NodeStyle = &pass.styles[usize::from(t)];
                let preferred = match axis {
                    Orientation::Horizontal => {
                        resolve_dimension(style.size.width, inputs.parent_size.width, pass.calcs)
                    }
                    Orientation::Vertical => {
                        resolve_dimension(style.size.height, inputs.parent_size.height, pass.calcs)
                    }
                    Orientation::Unspecified => None,
                };
                if let Some(preferred) = preferred {
                    let natural = natural.get_or_insert(taffy::Size::ZERO);
                    match axis {
                        Orientation::Horizontal => natural.width = preferred,
                        Orientation::Vertical => natural.height = preferred,
                        Orientation::Unspecified => {}
                    }
                }
            }
            // Restore an intrinsic outer size that the parent read from this
            // hook. Preserve independent flex or grid allocations unchanged.
            if let (Some((axis, scale)), Some(natural)) = (scale, natural) {
                let (known, natural) = match axis {
                    Orientation::Horizontal => {
                        (&mut natural_inputs.known_dimensions.width, natural.width)
                    }
                    Orientation::Vertical => {
                        (&mut natural_inputs.known_dimensions.height, natural.height)
                    }
                    Orientation::Unspecified => {
                        (&mut natural_inputs.known_dimensions.width, natural.width)
                    }
                };
                if known.is_some_and(|known| known == natural * scale as f32) {
                    *known = Some(natural);
                }
            }
            let display = pass.styles[usize::from(t)].display.taffy();
            let mut output = match (display, pass.child_count(t) == 0) {
                (Display::None, _) => taffy::compute_hidden_layout(pass, t),
                (Display::Grid, false) => taffy::compute_grid_layout(pass, t, natural_inputs),
                (_, false) => taffy::compute_flexbox_layout(pass, t, natural_inputs),
                (_, true) => pass.leaf(t, natural_inputs),
            };
            // A min-content probe is not the size the box is laid out at:
            // recording it would leave the next layout unrestored, scaled twice.
            let min_probe = inputs.run_mode == RunMode::ComputeSize
                && match scale {
                    Some((Orientation::Horizontal, _)) => {
                        inputs.available_space.width == taffy::AvailableSpace::MinContent
                    }
                    Some((Orientation::Vertical, _)) => {
                        inputs.available_space.height == taffy::AvailableSpace::MinContent
                    }
                    None | Some((Orientation::Unspecified, _)) => false,
                };
            if !min_probe {
                pass.natural_size[usize::from(t)] = Some(output.size);
            }
            match scale {
                Some((Orientation::Horizontal, scale)) => output.size.width *= scale as f32,
                Some((Orientation::Vertical, scale)) => output.size.height *= scale as f32,
                None | Some((Orientation::Unspecified, _)) => {}
            }
            output
        });
        // Only a parent is laid out again alone (`relayout_parents`): a leaf keeps none.
        if inputs.run_mode == RunMode::PerformLayout && self.child_count(t) > 0 {
            self.performed.insert(usize::from(t) as u64, inputs);
        }
        output
    }
}

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

impl<Tree: LayoutTree + ?Sized> LayoutGridContainer for LayoutPass<'_, Tree> {
    type GridContainerStyle<'a>
        = &'a TStyle
    where
        Self: 'a;

    type GridItemStyle<'a>
        = &'a NodeStyle
    where
        Self: 'a;

    #[inline(always)]
    fn get_grid_container_style(&self, t: TId) -> Self::GridContainerStyle<'_> {
        self.styles[usize::from(t)]
            .display
            .grid()
            .unwrap_or(self.no_grid)
    }

    #[inline(always)]
    fn get_grid_child_style(&self, t: TId) -> Self::GridItemStyle<'_> {
        self.item_style(t)
    }
}

impl<Tree: LayoutTree + ?Sized> RoundTree for LayoutPass<'_, Tree> {
    #[inline(always)]
    fn get_unrounded_layout(&self, t: TId) -> Layout {
        self.unrounded[usize::from(t)]
    }

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
    /// Replace compiled style from authoritative raw props and invalidate its chain.
    /// Tree, state and columns must share the same slot allocation domain.
    /// An absent identity is ignored; every resolved slot must already be allocated.
    pub(crate) fn rebuild_style<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        id: NodeId,
    ) {
        let Some(slot) = tree.index_of(id) else {
            return;
        };
        let index = slot + 1;
        let style = {
            self.calcs
                .free
                .extend(std::mem::take(&mut self.states[index].calcs));
            let mut alloc = CalcAlloc {
                calcs: &mut self.calcs,
                owned: Vec::new(),
            };
            let (props, hidden) = tree.layout_props(slot);
            let style =
                crate::style::taffy_style(props.unwrap_or(&Default::default()), hidden, &mut alloc);
            self.states[index].calcs = alloc.owned;
            style
        };
        let root = self.values.styles[index].position == taffy::Position::Absolute
            && style.position == taffy::Position::Absolute;
        self.values.relative[index] = uses_parent_size(&style);
        self.values.styles[index] = self.values.recent_styles.share(style);
        state.mark_dirty();
        self.states[index].measured = Default::default();
        if root {
            self.clear_caches_up(tree, state, slot);
        } else {
            self.clear_caches_through(tree, state, slot);
        }
    }
    /// Invalidate a content measurement and its dependency chain.
    /// Tree, state and columns must share the same slot allocation domain.
    /// Absent node identities mark work changed without accessing a slot.
    pub(crate) fn invalidate_chain<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        id: NodeId,
    ) {
        state.mark_dirty();
        if let Some(slot) = tree.index_of(id) {
            self.states[slot + 1].measured = Default::default();
            self.clear_caches_up(tree, state, slot);
        }
    }
    /// Invalidate the shared viewport's layout (the container of the fixed boxes).
    pub(crate) fn invalidate_viewport(&mut self, state: &mut LayoutState) {
        state.mark_dirty();
        self.values.caches[0].clear();
    }
    fn clear_caches_up<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        slot: usize,
    ) {
        let mut at = Some(slot);
        while let Some(slot) = at {
            // Contents boxes never participate in layout, so their empty
            // caches cannot prove that the contributing parent is clean.
            if tree
                .layout_props(slot)
                .0
                .is_some_and(|props| props.display == kivact_types::Display::Contents)
            {
                at = tree.parent(slot);
                continue;
            }
            let flow = tree.flow(slot);
            let parent = tree.parent(slot);
            let values = &mut self.values;
            if let ClearState::AlreadyEmpty = values.caches[slot + 1].clear() {
                break;
            }
            if flow == Flow::Overlay {
                break;
            }
            if flow == Flow::Fixed && parent.is_some() {
                values.caches[0].clear();
                break;
            }
            if values.styles[slot + 1].position == taffy::Position::Absolute
                && let Some(parent) = parent
            {
                state.queue_relayout(tree.id_at(parent).expect("invariant: layout parent exists"));
                break;
            }
            at = parent;
        }
    }
    pub(crate) fn clear_caches_through<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        slot: usize,
    ) {
        let flow = tree.flow(slot);
        let parent = tree.parent(slot);
        self.values.caches[slot + 1].clear();
        if flow == Flow::Overlay {
            return;
        }
        if flow == Flow::Fixed && parent.is_some() {
            self.values.caches[0].clear();
        } else if let Some(parent) = parent {
            self.clear_caches_up(tree, state, parent);
        }
    }
    fn relayout_parents<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        mut queued: Vec<kivact_types::NodeId>,
        measure: &mut dyn LeafMeasurer,
    ) {
        loop {
            let id = match queued.pop() {
                Some(id) => id,
                // Parents queued while laying these out go in this pass too.
                None => match state.relayout_pop() {
                    Some(id) => id,
                    None => break,
                },
            };
            let Some(slot) = tree.index_of(id) else {
                continue;
            };
            if tree.root() == Some(id) {
                continue;
            }
            let at = slot + 1;
            let performed = self.values.performed.get(&(at as u64)).copied();
            let previous = performed.and_then(|inputs| {
                self.values.caches[at]
                    .get(&inputs)
                    .map(|before| (inputs, before))
            });
            let Some((inputs, before)) = previous else {
                self.clear_caches_up(tree, state, slot);
                continue;
            };
            self.values.caches[at].clear();
            let after = self.with_pass(tree, state, measure, |pass| {
                pass.compute_child_layout(TId::from(at), inputs)
            });
            if same_from_outside(&before, &after) {
                self.values.unrounded[at].scrollable_overflow_rect = after.scrollable_overflow_rect;
                touch(
                    tree,
                    &mut self.values.touched,
                    &mut self.values.below,
                    self.stamp,
                    at,
                );
            } else {
                self.clear_caches_up(tree, state, slot);
            }
        }
    }
    fn with_pass<Tree: LayoutTree + ?Sized, R>(
        &mut self,
        tree: &Tree,
        state: &LayoutState,
        measure: &mut dyn LeafMeasurer,
        f: impl FnOnce(&mut LayoutPass<'_, Tree>) -> R,
    ) -> R {
        let NodeLayoutColumns {
            cursors,
            states,
            calcs,
            values,
            stamp,
        } = self;
        let NodeLayoutValues {
            styles,
            presence_scale,
            relative,
            natural_size,
            caches,
            unrounded,
            final_layout,
            geometry_changed,
            performed,
            touched,
            below,
            origin,
            scaled,
            never_rounded,
            default_style: _,
            recent_styles: _,
        } = values;
        let mut pass_calcs = std::borrow::Cow::Borrowed(&*calcs);
        let item_styles =
            scale_item_styles(tree, styles, presence_scale, scaled, calcs, &mut pass_calcs);
        let mut pass = LayoutPass {
            tree,
            traversal: Traversal {
                tree,
                cursors,
                state,
            },
            calcs: &pass_calcs,
            no_grid: &state.no_grid,
            records: states,
            styles,
            item_styles: &item_styles,
            presence_scale,
            relative,
            natural_size,
            caches,
            unrounded,
            final_layout,
            geometry_changed,
            performed,
            touched,
            below,
            origin,
            stamp: *stamp,
            never_rounded,
            measure,
        };
        f(&mut pass)
    }
    /// Compute roots in main/overlay/fixed order, retaining unrelated derived state.
    /// Tree, state and columns must share a slot allocation domain with every live
    /// tree slot allocated here. Measurement must describe that same source tree.
    /// Topology and raw properties must remain stable throughout the computation.
    /// Measurement callbacks may inspect content/resources; they cannot reach
    /// these columns, which are lent mutably for the whole computation.
    pub(crate) fn compute<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        relayout: Vec<kivact_types::NodeId>,
        available: Size,
        measure: &mut dyn LeafMeasurer,
    ) {
        self.values.geometry_changed.clear();
        self.next_stamp();
        state.fixed_cursor.set(None);
        self.relayout_parents(tree, state, relayout, measure);
        if let Some(root) = tree.root() {
            let root = taffy_id(tree.index_of(root).expect("invariant: layout node exists"));
            self.compute_at(tree, state, root, available, true, measure);
            for id in tree.overlays().iter().copied() {
                self.compute_overlay(tree, state, id, available, None, measure);
            }
            let traversal = Traversal {
                tree,
                cursors: &self.cursors,
                state,
            };
            if traversal.child_count(viewport()) != 0 {
                self.compute_at(tree, state, viewport(), available, true, measure);
            }
            self.finish_rounding(tree, state, measure);
        }
    }
    /// Round the live slots that take part in layout and that no pass has
    /// rounded since they were allocated or reset: each such subtree from its
    /// nearest rounded ancestor, never the whole tree. A Contents slot never takes
    /// part (its children are its flat parent's) and leaves the set; a detached
    /// slot stays (attaching it lays it out under a new parent, whose pass rounds
    /// it: its origin is `None`).
    fn finish_rounding<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &LayoutState,
        measure: &mut dyn LeafMeasurer,
    ) {
        let pending = self.pending_rounding(tree);
        if pending.is_empty() {
            return;
        }
        self.with_pass(tree, state, measure, |pass| {
            for index in pending {
                if !pass.never_rounded.contains(&index) {
                    continue;
                }
                if let Some((top, cum)) = pass.unrounded_top(index) {
                    pass.round_all(top, cum);
                }
            }
        });
    }
    /// Round the live slots no pass has rounded below the overlay `id`.
    pub(crate) fn finish_rounding_at<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &LayoutState,
        id: NodeId,
        measure: &mut dyn LeafMeasurer,
    ) {
        let pending = self.pending_rounding(tree);
        if pending.is_empty() {
            return;
        }
        let Some(slot) = tree.index_of(id) else {
            return;
        };
        let root = usize::from(taffy_id(slot));
        self.with_pass(tree, state, measure, |pass| {
            for index in pending {
                if !pass.never_rounded.contains(&index) || !pass.within(index, root) {
                    continue;
                }
                if let Some((top, cum)) = pass.unrounded_top(index) {
                    pass.round_all(top, cum);
                }
            }
        });
    }
    /// The never-rounded slots that take part in layout, in index order.
    /// Vacant slots and Contents slots leave the set.
    fn pending_rounding<Tree: LayoutTree + ?Sized>(&mut self, tree: &Tree) -> Vec<usize> {
        self.values.never_rounded.retain(|&index| {
            index > 0
                && tree.id_at(index - 1).is_some()
                && !tree
                    .layout_props(index - 1)
                    .0
                    .is_some_and(|p| p.display == kivact_types::Display::Contents)
        });
        self.values.never_rounded.iter().copied().collect()
    }
    /// Compute one overlay root again with an optional limit from anchor placement.
    pub(crate) fn compute_overlay<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &LayoutState,
        id: NodeId,
        available: Size,
        limit: Option<Size>,
        measure: &mut dyn LeafMeasurer,
    ) {
        let slot = tree.index_of(id).expect("invariant: overlay exists");
        let overlay = taffy_id(slot);
        let floor = tree
            .overlay_width_anchor(slot)
            .and_then(|anchor| self.computed_layout(tree, anchor))
            .map_or(0.0, |layout| layout.size.width);
        let index = usize::from(overlay);
        let Some(fit) = tree.overlay_fit(slot) else {
            let values = &mut self.values;
            if Rc::make_mut(&mut values.styles[index]).set_min_width_floor(floor as f32) {
                values.caches[index].clear();
            }
            self.compute_at(tree, state, overlay, available, false, measure);
            return;
        };
        let rect = fit.viewport(available);
        let limit = limit.unwrap_or(Size::new(rect.width, rect.height));
        let bounds = Size {
            width: limit.width.min(rect.width).floor(),
            height: limit.height.min(rect.height).floor(),
        };
        let style = Rc::make_mut(&mut self.values.styles[index]);
        let (min, max) = (style.min_size, style.max_size);
        let cap = |length: LengthPercentageAuto, limit, fallback| {
            LengthPercentageAuto::length(
                length
                    .resolve_to_option(limit, |handle, basis| self.calcs.resolve(handle, basis))
                    .unwrap_or(fallback)
                    .clamp(0.0, limit),
            )
        };
        style.set_min_width_floor(floor.min(bounds.width) as f32);
        style.set_size_bounds(
            taffy::Size {
                width: cap(min.width, bounds.width as f32, 0.0),
                height: cap(min.height, bounds.height as f32, 0.0),
            },
            taffy::Size {
                width: cap(max.width, bounds.width as f32, bounds.width as f32),
                height: cap(max.height, bounds.height as f32, bounds.height as f32),
            },
        );
        self.values.caches[index].clear();
        self.compute_at(tree, state, overlay, bounds, false, measure);
        Rc::make_mut(&mut self.values.styles[index]).set_size_bounds(min, max);
    }
    fn compute_at<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &LayoutState,
        root: TId,
        available: Size,
        fill: bool,
        measure: &mut dyn LeafMeasurer,
    ) {
        if fill {
            let values = &mut self.values;
            let style = Rc::make_mut(&mut values.styles[usize::from(root)]);
            let size = taffy::Size {
                width: taffy::Dimension::length(available.width as f32),
                height: taffy::Dimension::length(available.height as f32),
            };
            style.size = size;
            if let Some(grid) = style.display.grid_mut() {
                grid.size = size;
            }
            values.caches[usize::from(root)].clear();
        }
        let space = taffy::Size {
            width: taffy::AvailableSpace::Definite(available.width as f32),
            height: taffy::AvailableSpace::Definite(available.height as f32),
        };
        self.with_pass(tree, state, measure, |pass| {
            taffy::compute_root_layout(pass, root, space);
            let stamp = pass.stamp;
            pass.round_changed(root, (0.0, 0.0), stamp);
        });
    }
}
