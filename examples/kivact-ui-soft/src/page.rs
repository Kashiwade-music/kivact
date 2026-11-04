//! Role: Render a component's catalog heading and documentation content.
use crate::pages::PageInfo;
use kivact::style::style;
use kivact::view::{NodeBuilder, Scroll, column, scroll, text};

pub(super) fn page_view(info: &PageInfo) -> Scroll {
    scroll(
        column((
            text(info.category).style(style().font_size(14)),
            text(info.name)
                .style(style().font_size(29).weight(600))
                .test_id("catalog-title"),
            crate::docs::examples(info),
        ))
        .style(style().padding(24).gap(12))
        .test_id("catalog-page-content"),
    )
    .style(style().grow(1).min_width(0).min_height(0))
}
