//! Role: The layout of a root and the out-of-flow hooks of the layout pass.
//!
//! Taffy 0.15 leaves an absolute box to its containing block: a container's
//! layout lists the box and the containing block places it in its own pass
//! (`compute_oof_layout`). Every node here is positioned (`Relative` or
//! `Absolute`; `NodeStyle::default` is `Relative`), so the containing block of
//! an absolute box is always its layout parent: the box's location stays
//! relative to the parent's border box, the incremental rounding reaches it as
//! a child, and no hoisted list is kept.
//!
//! An absolute child of a grid container is placed in the area of its grid
//! lines, as in taffy 0.14: taffy hands each full grid layout's track
//! geometry to `set_detailed_grid_info` and reads it back in the out-of-flow
//! pass (`get_detailed_layout_info`). It is kept (`NodeLayoutValues::grid_infos`)
//! only for a grid container with an absolute child; any other grid keeps
//! nothing, and `reset` and `rebuild_style` drop a slot's entry.
//!
//! A root is laid out as taffy 0.14 did: at the origin, its margins taken out
//! of the space it lays out in. (Taffy 0.15's whole-root layout function would move
//! a root by its margins and insets and size an absolute root to fit.)

use super::LayoutPass;
use crate::LayoutTree;
use crate::style::node::{NodeStyle, TStyle};
use taffy::{
    AvailableSpace, CoreStyle, DetailedGridInfo, DetailedLayoutInfo, Layout, LayoutContainingBlock,
    LayoutGridContainer, LayoutInput, LayoutOutput, LayoutPartialTree, Line, NodeId as TId, Point,
    Position, RequestedAxis, ResolveOrZero, RunMode, Size, SizingMode, TraversePartialTree,
};

/// Lay out the root `root` in `space` and store its unrounded layout.
pub(super) fn layout_root<Tree: LayoutTree + ?Sized>(
    pass: &mut LayoutPass<'_, Tree>,
    root: TId,
    space: Size<AvailableSpace>,
) {
    let styles = pass.styles;
    let calcs = pass.calcs;
    let style: &NodeStyle = &styles[usize::from(root)];
    let resolve = |handle, basis| calcs.resolve(handle, basis);
    let parent_size = space.into_options();
    let margin = style.margin().resolve_or_zero(parent_size.width, resolve);
    let padding = style.padding().resolve_or_zero(parent_size.width, resolve);
    let border = style.border().resolve_or_zero(parent_size.width, resolve);
    let inputs = LayoutInput {
        run_mode: RunMode::PerformLayout,
        sizing_mode: SizingMode::InherentSize,
        axis: RequestedAxis::Both,
        known_dimensions: Size::NONE,
        known_dimensions_are_definite: Size {
            width: true,
            height: true,
        },
        parent_size,
        available_space: Size {
            width: without(space.width, margin.left + margin.right),
            height: without(space.height, margin.top + margin.bottom),
        },
        vertical_margins_are_collapsible: Line::FALSE,
    };
    let output = pass.compute_child_layout(root, inputs);
    debug_assert!(
        output.oof_candidates.is_empty(),
        "invariant: every node is positioned, so a parent places each absolute child"
    );
    pass.set_unrounded_layout(
        root,
        &Layout {
            order: 0,
            location: Point::ZERO,
            size: output.size,
            scrollable_overflow_rect: output.scrollable_overflow_rect,
            scrollbar_size: Size::ZERO,
            padding,
            border,
            margin,
        },
    );
}

/// `space` less `margins`: a definite available space is the border box's.
fn without(space: AvailableSpace, margins: f32) -> AvailableSpace {
    match space {
        AvailableSpace::Definite(value) => AvailableSpace::Definite((value - margins).max(0.0)),
        other => other,
    }
}

/// Place the absolute children of `t` after a full layout of it (`run_mode`
/// is `PerformLayout`); a measure must not place them.
#[inline(always)]
pub(super) fn place_out_of_flow<Tree: LayoutTree + ?Sized>(
    pass: &mut LayoutPass<'_, Tree>,
    t: TId,
    run_mode: RunMode,
    output: &mut LayoutOutput,
) {
    if run_mode == RunMode::PerformLayout {
        taffy::compute_oof_layout(pass, t, output);
    }
}

impl<Tree: LayoutTree + ?Sized> LayoutContainingBlock for LayoutPass<'_, Tree> {
    type OofItemStyle<'a>
        = &'a NodeStyle
    where
        Self: 'a;

    #[inline(always)]
    fn get_oof_item_style(&self, t: TId) -> Self::OofItemStyle<'_> {
        self.item_style(t)
    }

    // The containing block of an absolute box is its layout parent (see the
    // module docs): no hoisted list is kept.
    #[inline(always)]
    fn clear_hoisted_children(&mut self, _t: TId) {}

    #[inline(always)]
    fn add_hoisted_children(&mut self, _t: TId, _hoisted: &[TId]) {}

    /// The kept grid of `t` (see the module docs), else none.
    #[inline(always)]
    fn get_detailed_layout_info(&self, t: TId) -> &DetailedLayoutInfo<String> {
        self.grid_infos
            .get(&usize::from(t))
            .unwrap_or(&DetailedLayoutInfo::None)
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

    /// Keep the grid of `t` while it has an absolute child (its out-of-flow
    /// pass places that child by its grid lines); drop it otherwise.
    fn set_detailed_grid_info(&mut self, t: TId, info: DetailedGridInfo<String>) {
        let index = usize::from(t);
        let absolute = self
            .child_ids(t)
            .any(|child| self.item_style(child).position == Position::Absolute);
        if absolute {
            self.grid_infos
                .insert(index, DetailedLayoutInfo::Grid(Box::new(info)));
        } else {
            self.grid_infos.remove(&index);
        }
    }
}
