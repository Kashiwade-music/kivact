//! Catalog page of Scroll Area: vertical, horizontal and both axes.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::ScrollAxes;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("scroll-area", "Scroll Area", "Components", include_str!("scroll_area.rs"))
        .description("An area that scrolls when its content overflows. Give it a height or a width with style; thin scrollbars appear over the content.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "vertical",
            "Vertical",
            "Once the height is fixed, the part that overflows scrolls vertically.",
            vertical,
        ))
        .example(Example::new(
            "horizontal",
            "Horizontal",
            ".axes(ScrollAxes::Horizontal) scrolls horizontally.",
            horizontal,
        ))
        .example(Example::new(
            "both",
            "Both",
            ".axes(ScrollAxes::Both) scrolls both vertically and horizontally.",
            both,
        ))
}

// example: vertical
fn vertical() -> impl IntoView {
    let tags = (0..40)
        .map(|n| text(format!("v1.2.0-beta.{n}")).style(style().height(24)))
        .collect::<Vec<_>>();
    basic::scroll_area(column(tags)).style(style().height(180).width(240))
}
// end example

// example: horizontal
fn horizontal() -> impl IntoView {
    let tiles = (1..=12)
        .map(|n| {
            kivact::ui::skeleton()
                .style(style().size(96, 64).flex_shrink(0))
                .name(format!("Artwork {n}"))
        })
        .collect::<Vec<_>>();
    basic::scroll_area(row(tiles).style(style().gap(12).padding_bottom(12)))
        .axes(ScrollAxes::Horizontal)
        .style(style().width(420))
}
// end example

// example: both
fn both() -> impl IntoView {
    let rows = (0..20)
        .map(|r| {
            row((0..6)
                .map(|c| text(format!("R{r} C{c}")).style(style().width(64).flex_shrink(0)))
                .collect::<Vec<_>>())
            .style(style().height(24).align_items(Align::Center))
        })
        .collect::<Vec<_>>();
    basic::scroll_area(column(rows))
        .axes(ScrollAxes::Both)
        .style(style().height(180).width(240))
}
// end example
