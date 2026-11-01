//! Role: Catalog page of Separator: horizontal and vertical lines.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "separator",
        name: "Separator",
        description: "内容を視覚的に区切る 1 px の線です。Role::Separator を持ち、orientation に Orientation::Vertical を渡すと縦線になります。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("separator.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "horizontal",
                "Horizontal",
                "既定は横線で、親の幅いっぱいに広がります。",
                horizontal,
            ),
            Example::new(
                "vertical",
                "Vertical",
                "vertical(true) で縦線になり、並んだ要素の高さに合わせて伸びます。",
                vertical,
            ),
        ],
    }
}

// example: horizontal
fn horizontal() -> impl IntoView {
    column((
        column((
            text("Kivact UI").style(style().font_size(14).weight(500)),
            basic::muted("An open-source UI toolkit for Rust."),
        ))
        .style(style().gap(4)),
        basic::separator(),
        row((text("Docs"), text("Source"), text("Examples"))).style(style().gap(16)),
    ))
    .style(style().gap(16).width(320))
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    row((
        text("Blog"),
        basic::separator().orientation(Orientation::Vertical),
        text("Docs"),
        basic::separator().orientation(Orientation::Vertical),
        text("Source"),
    ))
    .style(style().gap(16).height(20).align_items(Align::Center))
}
// end example
