//! Role: Frame an interactive catalog specimen and show its source.
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::{Node, column, text};

pub(super) fn specimen(title: &str, view: impl IntoView, source: &'static str) -> Node {
    let tokens = kivact::style::use_tokens::<kivact_ui_soft::Tokens>();
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
        let tokens = kivact::style::use_tokens::<kivact_ui_soft::Tokens>();
        style()
            .gap(8)
            .padding(12)
            .background(tokens.colors.bg_surface)
            .border(1, tokens.colors.border)
            .radius(tokens.radius.container)
    })
}
