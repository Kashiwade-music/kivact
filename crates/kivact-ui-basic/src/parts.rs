//! Role: Shared parts of display components: pressable surfaces and resized icons.

use kivact::custom::{PartState, part};
use kivact::prelude::NodeBuilder;
use kivact::state::{Callback, Value};
use kivact::style::style;
use kivact::view::{IntoView, Role};
use kivact::view::{Node, node};

/// Attach hover, focus and press tracking to `root`.
///
/// With `on_press` the node becomes a focusable `role` that reports
/// activation (click, Enter, Space, assistive technology); without it the
/// state still follows the pointer, but the node takes no focus.
#[track_caller]
pub(crate) fn pressable(
    root: Node,
    role: Role,
    on_press: Option<Callback<()>>,
) -> (Node, Value<PartState>) {
    let attachment = part(|input| input);
    let state = attachment.state();
    let root = root.with_node(|root| root.part(attachment));
    let root = match on_press {
        Some(callback) => root
            .with_node(|root| root.role(role))
            .focusable(true)
            .on_press(callback),
        None => root,
    };
    (root, state)
}

/// Show `view` (a 16 px icon) at `to` pixels: the layout box is `to` wide and the glyph is scaled.
#[track_caller]
pub(crate) fn sized_icon(view: impl IntoView, to: impl kivact::state::IntoValue<f64>) -> Node {
    let to = to.into_value();
    let from = 16.0;
    node()
        .style(move || {
            style()
                .size(from, from)
                .margin((to.get() - from) / 2.0)
                .shrink(0)
                .scale(to.get() / from)
        })
        .child(view)
}
