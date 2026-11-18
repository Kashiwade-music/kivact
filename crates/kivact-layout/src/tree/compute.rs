//! Role: Computes the roots in order, relayouts the parents of absolute boxes, and finishes the rounding.

use super::pass::LayoutPass;
use super::root::layout_root;
use super::rounding::touch;
use crate::columns::scale_item_styles;
use crate::traversal::{Traversal, taffy_id, viewport};
use crate::{LayoutState, LayoutTree, LeafMeasurer, NodeLayoutColumns, NodeLayoutValues};
use kivact_types::{NodeId, Size};
use std::rc::Rc;
use taffy::{
    LayoutOutput, LayoutPartialTree, LengthPercentageAuto, NodeId as TId, TraversePartialTree,
};

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
        ..after.clone()
    } == *before
        && reach(before) == reach(after)
}

impl NodeLayoutColumns {
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
            grid_infos,
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
            grid_infos,
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
            layout_root(pass, root, space);
            let stamp = pass.stamp;
            pass.round_changed(root, (0.0, 0.0), stamp);
        });
    }
}
