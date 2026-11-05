//! Role: Catalog page of Scroll Area: vertical, horizontal and both axes.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "scroll-area",
        name: "Scroll Area",
        description: "内容がはみ出したときにスクロールできる領域です。高さや幅は style で与え、細いスクロールバーが重なって表示されます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("scroll_area.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "vertical",
                "Vertical",
                "高さを決めると、はみ出した分を縦にスクロールします。",
                vertical,
            ),
            Example::new(
                "horizontal",
                "Horizontal",
                ".axes(ScrollAxes::Horizontal) を渡すと、横にスクロールします。",
                horizontal,
            ),
            Example::new(
                "both",
                "Both",
                ".axes(ScrollAxes::Both) は縦横の両方にスクロールします。",
                both,
            ),
        ],
    }
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
                .style(style().size(96, 64).shrink(0))
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
                .map(|c| text(format!("R{r} C{c}")).style(style().width(64).shrink(0)))
                .collect::<Vec<_>>())
            .style(style().height(24).align_items(Align::Center))
        })
        .collect::<Vec<_>>();
    basic::scroll_area(column(rows))
        .axes(ScrollAxes::Both)
        .style(style().height(180).width(240))
}
// end example
