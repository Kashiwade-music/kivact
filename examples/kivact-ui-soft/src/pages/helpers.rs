//! Role: The shared demo helper: a page's title over its content.
use kivact::style::style;
use kivact::view::{IntoView, Node, column, text};

/// The demo of a page: its title over its content.
pub(crate) fn labelled(title: &str, view: impl IntoView) -> Node {
    column((
        text(title.to_owned()).style(style().font_weight(600)),
        column(view).style(style().gap(12)),
    ))
    .style(style().gap(8))
}
