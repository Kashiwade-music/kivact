//! Role: Region templates retain one part while adoption lifetimes change.

use crate::view::common::Common;
use crate::view::handler_layers::Layer;
use crate::view::tree::{Handlers, PartActions};

/// A region takes its part's observers and defaults. The node's fallback for
/// unhandled region input is not offered to regions.
const REGION_LAYERS: [Layer; 2] = [Layer::Observer, Layer::PartDefault];

fn handlers(source: &Handlers) -> Handlers {
    Handlers {
        activate: source.activate.clone(),
        key: source.key.only(&REGION_LAYERS),
        key_up: source.key_up.only(&REGION_LAYERS),
        pointer: source.pointer.only(&REGION_LAYERS),
        wheel: source.wheel.only(&REGION_LAYERS),
        a11y: source.a11y.clone(),
        ..Default::default()
    }
}

/// Only semantic and input transport is copied; no view or appearance is created.
pub(super) fn instantiate(source: &Common) -> Common {
    Common {
        node_ref: source.node_ref.clone(),
        role: source.role,
        name: source.name.clone(),
        description: source.description.clone(),
        value: source.value.clone(),
        selected: source.selected.clone(),
        disabled: source.disabled.clone(),
        hidden: source.hidden.clone(),
        invalid: source.invalid.clone(),
        semantics: source.semantics.clone(),
        checked: source.checked.clone(),
        indeterminate: source.indeterminate.clone(),
        expanded: source.expanded.clone(),
        focusable: source.focusable.clone(),
        focus_on_press: source.focus_on_press,
        handlers: handlers(&source.handlers),
        part_constraints: source.part_constraints.clone(),
        part_actions: PartActions {
            text: None,
            ..source.part_actions.clone()
        },
        part_bindings: source.part_bindings.clone(),
        part_modal: source.part_modal,
        ..Default::default()
    }
}
