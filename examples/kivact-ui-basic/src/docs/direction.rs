//! Catalog page of Direction: provide a reading direction and read it back.

use catalog_shell::{Example, Page};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page::new("direction", "Direction", "Components", include_str!("direction.rs"))
        .description("A boundary that tells a subtree which way to read (left to right, or right to left). A component that follows the direction reads it when it is built, and reverses its order and its arrows. The standard kivact::ui components do not reverse.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "provider",
            "Provider",
            "Pass a direction and a function that builds the children to provide_direction. Inside, use_direction returns the direction that was passed.",
            provider,
        ))
        .example(Example::new(
            "toolbar",
            "Toolbar",
            "An example that reads use_direction and reverses a row by hand.",
            toolbar,
        ))
}

// example: provider
fn provider() -> impl IntoView {
    let describe = || format!("{:?}", kivact::view::use_direction());
    column((
        kivact::view::provide_direction(kivact::view::ReadingDirection::Ltr, move || {
            text(format!("Left to right: {}", describe()))
        }),
        kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, move || {
            text(format!("Right to left: {}", describe()))
        }),
    ))
    .style(style().gap(8))
}
// end example

// example: toolbar
fn toolbar() -> impl IntoView {
    let actions = |label: &'static str| {
        let reversed = kivact::view::use_direction() == kivact::view::ReadingDirection::Rtl;
        let mut items = vec![icon("arrow-left"), text(label), icon("arrow-right")];
        if reversed {
            items.reverse();
        }
        row(items).style(style().gap(8).align_items(Align::Center))
    };
    column((
        kivact::view::provide_direction(kivact::view::ReadingDirection::Ltr, move || {
            actions("Previous, next")
        }),
        kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, move || {
            actions("Previous, next")
        }),
    ))
    .style(style().gap(16))
}
// end example
