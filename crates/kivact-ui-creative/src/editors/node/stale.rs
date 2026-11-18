//! Role: Node canvas stale-state check shared by the view and the input handlers.
//! It is a leaf module so that neither view nor input has to reference the other.

use crate::editors::node::gesture::{NodeMode, State};

/// Whether the held slot, the connecting output or the dragged items still exist.
pub(super) fn stale(state: State) -> bool {
    let nodes = state.nodes.get_untracked();
    let slot_valid =
        kivact::state::untrack(|| state.held_slot()).is_none_or(|(id, index, original)| {
            nodes
                .iter()
                .find(|node| node.id == id)
                .and_then(|node| node.controls.get(index))
                .is_some_and(|control| crate::editors::node::edit::same_slot(&original, control))
        });
    let connection_valid = state.mode.with_untracked(|mode| match mode {
        NodeMode::Connecting { from, output } => nodes
            .iter()
            .find(|node| node.id == *from)
            .is_some_and(|node| *output < node.outputs.len()),
        _ => true,
    });
    let drag_valid = kivact::state::untrack(|| state.drag_state()).is_none_or(|drag| match drag {
        crate::editors::node::gesture::DragMode::Items { ids } => {
            ids.iter().all(|id| nodes.iter().any(|node| node.id == *id))
        }
        crate::editors::node::gesture::DragMode::Link { source, .. } => {
            let (from, output) = source.output();
            nodes
                .iter()
                .find(|node| node.id == from)
                .is_some_and(|node| output < node.outputs.len())
        }
        _ => true,
    });
    !(slot_valid && connection_valid && drag_valid)
}
