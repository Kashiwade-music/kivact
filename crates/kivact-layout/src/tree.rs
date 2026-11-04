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
//! - A leaf's measurements are recorded by request (`Record`) and taken
//!   from the record before the leaf is measured.
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
//! - Layouts are rounded to whole pixels (`taffy::round_layout`);
//!   `NodeLayoutColumns::layout_at` reads a node's by its position, and only a
//!   lookup by `NodeId` goes through the store's index.

use crate::calcs::CalcAlloc;
use crate::calcs::Calcs;
use crate::style::node::{NodeStyle, TStyle};
use crate::traversal::{Entry, Traversal, entry, taffy_id, viewport};
use crate::{
    Flow, GeometryChange, GeometryChangeKind, LayoutState, LayoutTree, LeafMeasurer, NodeLayout,
    NodeLayoutColumns, NodeLayoutState, NodeLayoutValues,
};
use kivact_types::{
    AvailableSpace, Baseline, MeasureRequest, NodeId, Orientation, Point, Rect, Size,
};
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
    styles: &'a [NodeStyle],
    item_styles: &'a [NodeStyle],
    presence_scale: &'a [Option<(Orientation, f64)>],
    natural_size: &'a mut [Option<taffy::Size<f32>>],
    caches: &'a mut [taffy::Cache],
    unrounded: &'a mut [Layout],
    final_layout: &'a mut [Option<Layout>],
    geometry_changed: &'a mut Vec<GeometryChange>,
    performed: &'a mut [Option<LayoutInput>],
    measure: &'a mut dyn LeafMeasurer,
}

impl<Tree: LayoutTree + ?Sized> LayoutPass<'_, Tree> {
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
        let style = &styles[index];
        let resolve = |handle, basis| calcs.resolve(handle, basis);
        let mut baseline = None;
        let mut out = taffy::compute_leaf_layout(inputs, style, resolve, |known, avail| {
            let slot = match entry(t) {
                Entry::Viewport => return taffy::Size::ZERO,
                Entry::Slot(slot) => slot,
            };
            let request = MeasureRequest {
                known_width: known.width.map(f64::from),
                known_height: known.height.map(f64::from),
                available_width: from_taffy(avail.width),
                available_height: from_taffy(avail.height),
            };
            let id = tree.id_at(slot).expect("invariant: layout node exists");
            let record = &mut records[index].measured;
            let cached = record
                .sizes
                .iter()
                .find(|(r, _)| *r == request)
                .map(|(_, s)| *s);
            let leaf = cached.unwrap_or_else(|| {
                let leaf = measure.measure(id, request);
                if record.sizes.len() == 16 {
                    record.sizes.remove(0);
                    record.state = crate::columns::RecordState::Incomplete;
                }
                record.sizes.push((request, leaf));
                leaf
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

fn scaled_dimension(
    value: taffy::Dimension,
    scale: f64,
    calcs: &Calcs,
    alloc: &mut CalcAlloc<'_>,
) -> taffy::Dimension {
    use taffy::style::CompactLength;
    match value.tag() {
        CompactLength::LENGTH_TAG => taffy::Dimension::length(value.value() * scale as f32),
        CompactLength::PERCENT_TAG => taffy::Dimension::percent(value.value() * scale as f32),
        CompactLength::CALC_TAG => {
            let handle = value.into_raw().calc_value();
            let px = f64::from(calcs.resolve(handle, 0.0));
            let percent = f64::from(calcs.resolve(handle, 100.0)) - px;
            taffy::Dimension::calc(alloc.calc(percent * scale, px * scale))
        }
        _ => value,
    }
}

fn scaled_length(
    value: taffy::LengthPercentageAuto,
    scale: f64,
    calcs: &Calcs,
    alloc: &mut CalcAlloc<'_>,
) -> taffy::LengthPercentageAuto {
    use taffy::style::CompactLength;
    match value.into_raw().tag() {
        CompactLength::LENGTH_TAG => {
            taffy::LengthPercentageAuto::length(value.into_raw().value() * scale as f32)
        }
        CompactLength::PERCENT_TAG => {
            taffy::LengthPercentageAuto::percent(value.into_raw().value() * scale as f32)
        }
        CompactLength::CALC_TAG => {
            let handle = value.into_raw().calc_value();
            let px = f64::from(calcs.resolve(handle, 0.0));
            let percent = f64::from(calcs.resolve(handle, 100.0)) - px;
            taffy::LengthPercentageAuto::calc(alloc.calc(percent * scale, px * scale))
        }
        _ => value,
    }
}

fn from_taffy(space: taffy::AvailableSpace) -> AvailableSpace {
    match space {
        taffy::AvailableSpace::Definite(v) => AvailableSpace::Definite(f64::from(v)),
        taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
        taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
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
        &self.styles[usize::from(t)]
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, t: TId, layout: &Layout) {
        self.unrounded[usize::from(t)] = *layout;
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
        let output = taffy::compute_cached_layout(self, t, inputs, |pass, t, inputs| {
            let scale = pass.presence_scale[usize::from(t)];
            let mut natural_inputs = inputs;
            let mut natural = pass.natural_size[usize::from(t)];
            if let Some((axis, _)) = scale {
                let style = &pass.styles[usize::from(t)];
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
            let mut output = match (
                pass.styles[usize::from(t)].display,
                pass.child_count(t) == 0,
            ) {
                (Display::None, _) => taffy::compute_hidden_layout(pass, t),
                (Display::Grid, false) => taffy::compute_grid_layout(pass, t, natural_inputs),
                (_, false) => taffy::compute_flexbox_layout(pass, t, natural_inputs),
                (_, true) => pass.leaf(t, natural_inputs),
            };
            pass.natural_size[usize::from(t)] = Some(output.size);
            match scale {
                Some((Orientation::Horizontal, scale)) => output.size.width *= scale as f32,
                Some((Orientation::Vertical, scale)) => output.size.height *= scale as f32,
                None | Some((Orientation::Unspecified, _)) => {}
            }
            output
        });
        if inputs.run_mode == RunMode::PerformLayout {
            self.performed[usize::from(t)] = Some(inputs);
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
        &self.styles[usize::from(t)]
    }

    #[inline(always)]
    fn get_flexbox_child_style(&self, t: TId) -> Self::FlexboxItemStyle<'_> {
        &self.item_styles[usize::from(t)]
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
            .grid
            .as_deref()
            .unwrap_or(self.no_grid)
    }

    #[inline(always)]
    fn get_grid_child_style(&self, t: TId) -> Self::GridItemStyle<'_> {
        &self.item_styles[usize::from(t)]
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
        self.values.styles[index] = style;
        state.changed = true;
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
        state.changed = true;
        if let Some(slot) = tree.index_of(id) {
            self.states[slot + 1].measured = Default::default();
            self.clear_caches_up(tree, state, slot);
        }
    }
    /// Invalidate the shared viewport's layout (the container of the fixed boxes).
    pub(crate) fn invalidate_viewport(&mut self, state: &mut LayoutState) {
        state.changed = true;
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
                state
                    .relayout
                    .push(tree.id_at(parent).expect("invariant: layout parent exists"));
                break;
            }
            at = parent;
        }
    }
    fn clear_caches_through<Tree: LayoutTree + ?Sized>(
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
        measure: &mut dyn LeafMeasurer,
    ) {
        while let Some(id) = state.relayout.pop() {
            let Some(slot) = tree.index_of(id) else {
                continue;
            };
            if tree.root() == Some(id) {
                continue;
            }
            let at = slot + 1;
            let performed = self.values.performed[at];
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
        } = self;
        let NodeLayoutValues {
            styles,
            presence_scale,
            natural_size,
            caches,
            unrounded,
            final_layout,
            geometry_changed,
            performed,
        } = values;
        let mut pass_calcs = std::borrow::Cow::Borrowed(&*calcs);
        let mut item_styles = presence_scale
            .iter()
            .any(Option::is_some)
            .then(|| styles.clone());
        for (index, (style, scale)) in item_styles
            .iter_mut()
            .flat_map(|styles| styles.iter_mut())
            .zip(presence_scale.iter())
            .enumerate()
        {
            let Some((axis, scale)) = *scale else {
                continue;
            };
            if axis == Orientation::Unspecified {
                continue;
            }
            let mut alloc = CalcAlloc {
                calcs: pass_calcs.to_mut(),
                owned: Vec::new(),
            };
            let (size, min, max) = match axis {
                Orientation::Horizontal => (
                    &mut style.size.width,
                    &mut style.min_size.width,
                    &mut style.max_size.width,
                ),
                Orientation::Vertical => (
                    &mut style.size.height,
                    &mut style.min_size.height,
                    &mut style.max_size.height,
                ),
                Orientation::Unspecified => continue,
            };
            *size = scaled_dimension(*size, scale, calcs, &mut alloc);
            *min = scaled_length(*min, scale, calcs, &mut alloc);
            *max = scaled_length(*max, scale, calcs, &mut alloc);
            if axis == Orientation::Horizontal {
                style.min_width_floor *= scale as f32;
            }
            if let Some(parent) = index.checked_sub(1).and_then(|slot| tree.parent(slot)) {
                let row = matches!(
                    styles[parent + 1].flex_direction,
                    taffy::FlexDirection::Row | taffy::FlexDirection::RowReverse
                );
                if row == (axis == Orientation::Horizontal) {
                    style.flex_basis = scaled_dimension(style.flex_basis, scale, calcs, &mut alloc);
                }
            }
        }
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
            item_styles: item_styles.as_deref().unwrap_or(styles),
            presence_scale,
            natural_size,
            caches,
            unrounded,
            final_layout,
            geometry_changed,
            performed,
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
        available: Size,
        measure: &mut dyn LeafMeasurer,
    ) {
        self.values.geometry_changed.clear();
        state.fixed_cursor.set(None);
        self.relayout_parents(tree, state, measure);
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
        }
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
            if values.styles[index].set_min_width_floor(floor as f32) {
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
        let style = &mut self.values.styles[index];
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
        self.values.styles[index].set_size_bounds(min, max);
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
            let style = &mut values.styles[usize::from(root)];
            let size = taffy::Size {
                width: taffy::Dimension::length(available.width as f32),
                height: taffy::Dimension::length(available.height as f32),
            };
            style.size = size;
            if let Some(grid) = style.grid.as_mut() {
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
            taffy::round_layout(pass, root);
        });
    }
}
