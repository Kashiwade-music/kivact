//! Role: Shared parts of display components: pressable surfaces and resized icons.

use kivact::control::{Node, node};
use kivact::prelude::NodeBuilder;
use kivact::primitive::{PartState, part};
use kivact::style::style;
use kivact::view::{IntoView, Role};
use kivact::{Callback, Value};

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
    let attachment = match on_press {
        Some(callback) => attachment
            .role(role)
            .focusable(true)
            .on_press(move || callback.call(())),
        None => attachment,
    };
    (root.part(attachment), state)
}

/// Show `view` (a 16 px icon) at `to` pixels: the layout box is `to` wide and the glyph is scaled.
#[track_caller]
pub(crate) fn sized_icon(view: impl IntoView, to: f64) -> Node {
    let from = 16.0;
    node()
        .style(
            style()
                .size(from, from)
                .margin((to - from) / 2.0)
                .shrink(0)
                .scale(to / from),
        )
        .child(view)
}
