//! Role: An external text input built from public headless editing parts.

use kivact::Binding;
use kivact::control::{Node, NodeBuilder, node};
use kivact::interaction::use_text_edit;
use kivact::style::{ColorToken, style};
use kivact::view::IntoText;

/// Build an independently styled editor from the public `text_edit` Part.
pub fn text_input(label: impl IntoText, value: impl Into<Binding<String>>) -> Node {
    node()
        .name(label)
        .part(use_text_edit(value).control())
        .style(
            style()
                .width(240)
                .height(36)
                .padding(8)
                .border(1, ColorToken::Border)
                .background(ColorToken::Surface)
                .color(ColorToken::Text),
        )
}
