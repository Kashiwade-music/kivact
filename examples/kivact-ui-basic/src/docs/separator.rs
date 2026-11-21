//! Catalog page of Separator: horizontal and vertical lines.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Orientation;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("separator", "Separator", "Components", include_str!("separator.rs"))
        .description("A 1 px line that divides content visually. It has Role::Separator; passing Orientation::Vertical to orientation makes it a vertical line.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "horizontal",
            "Horizontal",
            "The default is a horizontal line that spans the full width of the parent.",
            horizontal,
        ))
        .example(Example::new(
            "vertical",
            "Vertical",
            "vertical(true) makes it a vertical line that stretches to the height of the elements beside it.",
            vertical,
        ))
}

// example: horizontal
fn horizontal() -> impl IntoView {
    column((
        column((
            text("Kivact UI").style(style().font_size(14).font_weight(500)),
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
