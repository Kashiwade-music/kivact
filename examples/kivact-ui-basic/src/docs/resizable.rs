//! Catalog page of Resizable: horizontal, vertical and a grip handle.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::ui::split;
use kivact::view::Orientation;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("resizable", "Resizable", "Components", include_str!("resizable.rs"))
        .description("Changes the size of the areas by dragging the boundary or moving it with the keyboard. It draws kivact::ui::split in the basic theme. The sizes are a row of fractions that sum to 1, held by the app. The arrow keys move 1 % at a time; Home and End go to the minimum and the maximum; Enter and a double click split evenly with the neighbor. By default no area gets smaller than 10 % of the group (min_size changes that).")
        .usage("use kivact::prelude::*;\nuse kivact::ui::split;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "horizontal",
            "Horizontal",
            "Pass the views of the panels and a binding of the fractions to split. The default direction is side by side. Tab focuses a boundary, and the left and right arrows move it.",
            horizontal,
        ))
        .example(Example::new(
            "vertical",
            "Vertical",
            "Pass Orientation::Vertical to orientation to stack them; the boundary then moves with the up and down arrows.",
            vertical,
        ))
        .example(Example::new(
            "handle",
            "With handle",
            "grip(true) draws a handle on the boundary. min_size changes the fraction below which an area cannot shrink.",
            handle,
        ))
}

// example: horizontal
fn horizontal() -> impl IntoView {
    /// A centered label for a panel.
    fn panel(label: &str) -> impl IntoView {
        node()
            .style(
                style()
                    .height(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::Center),
            )
            .child(basic::large(label.to_string()))
    }
    let sizes = signal(vec![0.25f64, 0.75]);
    split(sizes, vec![panel("One"), panel("Two")]).style(
        style()
            .width(420)
            .height(160)
            .border(1, basic::BORDER)
            .padding(1),
    )
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    /// A centered label for a panel.
    fn panel(label: &str) -> impl IntoView {
        node()
            .style(
                style()
                    .height(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::Center),
            )
            .child(basic::large(label.to_string()))
    }
    let sizes = signal(vec![0.3f64, 0.7]);
    split(sizes, vec![panel("Header"), panel("Content")])
        .orientation(Orientation::Vertical)
        .style(
            style()
                .width(420)
                .height(220)
                .border(1, basic::BORDER)
                .padding(1),
        )
}
// end example

// example: handle
fn handle() -> impl IntoView {
    /// A centered label for a panel.
    fn panel(label: &str) -> impl IntoView {
        node()
            .style(
                style()
                    .height(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::Center),
            )
            .child(basic::large(label.to_string()))
    }
    let sizes = signal(vec![0.25f64, 0.5, 0.25]);
    column((
        split(
            sizes,
            vec![panel("Sidebar"), panel("Content"), panel("Inspector")],
        )
        .grip(true)
        .min_size(0.15)
        .style(
            style()
                .width(480)
                .height(160)
                .border(1, basic::BORDER)
                .padding(1),
        ),
        basic::muted(move || {
            let shares: Vec<String> = sizes
                .get()
                .iter()
                .map(|size| format!("{:.0}%", size * 100.0))
                .collect();
            format!("Panels: {}", shares.join(" / "))
        }),
    ))
    .style(style().gap(12))
}
// end example
