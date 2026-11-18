//! Role: The taffy partial-tree hooks of the layout pass: a child's layout with its scaled and out-of-flow steps.

use super::pass::LayoutPass;
use super::root::place_out_of_flow;
use super::rounding::touch;
use crate::LayoutTree;
use crate::calcs::Calcs;
use crate::style::node::NodeStyle;
use kivact_types::Orientation;
use taffy::{
    Display, Layout, LayoutInput, LayoutOutput, LayoutPartialTree, NodeId as TId, RunMode,
    TraversePartialTree,
};

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
        _ if value.into_raw().is_plain_calc() => {
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
            place_out_of_flow(pass, t, inputs.run_mode, &mut output);
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
