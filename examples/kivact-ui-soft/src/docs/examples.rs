//! Role: Assemble the catalog's component demonstrations.
use crate::PageInfo;
use kivact::control::{Node, NodeBuilder, column, node, text};
use kivact::style::style;
use kivact::view::IntoView;

pub(crate) fn examples(page: &PageInfo) -> Node {
    super::controls::examples(page.slug)
        .or_else(|| super::forms::examples(page.slug))
        .or_else(|| super::overlays::examples(page.slug))
        .or_else(|| super::collections::examples(page.slug))
        .unwrap_or_else(node)
}

pub(super) fn specimen(title: &str, view: impl IntoView, source: &'static str) -> Node {
    let tokens = kivact_ui_soft::tokens();
    column((
        text(title.to_owned()).style(style().weight(600)),
        column(view).style(style().gap(12).padding(16)),
        text(source).style(
            style()
                .font_family(tokens.typography.family_code)
                .font_size(12)
                .padding(12),
        ),
    ))
    .style(|| {
        let tokens = kivact_ui_soft::tokens();
        style()
            .gap(8)
            .padding(12)
            .background(tokens.colors.bg_surface)
            .border(1, tokens.colors.border)
            .radius(tokens.radius.container)
    })
}
