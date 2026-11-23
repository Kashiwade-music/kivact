//! Catalog page of Carousel: default, several per view, vertical and loop.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::{Node, Orientation};
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("carousel", "Carousel", "Components", include_str!("carousel.rs"))
        .description("A component that moves through the slides one at a time (or several at a time). The current position is passed as a binding; the previous and next buttons and the left and right arrow keys (up and down when vertical) move it. The buttons sit 48 px outside the box.")
        .usage("use kivact::prelude::*;\nuse kivact::style::use_theme;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass the view of the slides and a binding for the position. The buttons are disabled at the ends. Tab focuses the box, and the arrow keys move it too.",
            default,
        ))
        .example(Example::new(
            "per-view",
            "Items per view",
            "items_per_view sets how many slides show at once. The last position is the one where all the slides still show.",
            per_view,
        ))
        .example(Example::new(
            "vertical",
            "Vertical",
            "Pass Orientation::Vertical to orientation to stack the slides; the buttons go above and below. Give it a height.",
            vertical,
        ))
        .example(Example::new(
            "loop",
            "Loop",
            "looping(true) wraps from the last slide back to the first, and from the first back to the last.",
            looped,
        ))
}

// example: default
fn default() -> impl IntoView {
    /// A bordered slide with a number.
    fn slide(number: usize) -> Node {
        node()
            .style(
                style()
                    .height(120)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, basic::BORDER)
                    .font_size(30)
                    .font_weight(600),
            )
            .child(text(number.to_string()))
    }
    let index = signal(0usize);
    kivact::ui::carousel(index, (1..=5).map(slide).collect::<Vec<_>>())
        .style(style().width(240).margin_x(48))
}
// end example

// example: per-view
fn per_view() -> impl IntoView {
    /// A bordered slide with a number.
    fn slide(number: usize) -> Node {
        node()
            .style(
                style()
                    .height(120)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, basic::BORDER)
                    .font_size(30)
                    .font_weight(600),
            )
            .child(text(number.to_string()))
    }
    let index = signal(0usize);
    kivact::ui::carousel(index, (1..=6).map(slide).collect::<Vec<_>>())
        .items_per_view(2.0)
        .style(style().width(360).margin_x(48))
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    /// A bordered slide with a number.
    fn slide(number: usize) -> Node {
        node()
            .style(
                style()
                    .height(120)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, basic::BORDER)
                    .font_size(30)
                    .font_weight(600),
            )
            .child(text(number.to_string()))
    }
    let index = signal(0usize);
    kivact::ui::carousel(index, (1..=5).map(slide).collect::<Vec<_>>())
        .orientation(Orientation::Vertical)
        .style(style().width(240).height(120).margin_y(48))
}
// end example

// example: loop
fn looped() -> impl IntoView {
    /// A bordered slide with a number.
    fn slide(number: usize) -> Node {
        node()
            .style(
                style()
                    .height(120)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .border(1, basic::BORDER)
                    .font_size(30)
                    .font_weight(600),
            )
            .child(text(number.to_string()))
    }
    let index = signal(0usize);
    kivact::ui::carousel(index, (1..=3).map(slide).collect::<Vec<_>>())
        .looping(true)
        .style(style().width(240).margin_x(48))
}
// end example
