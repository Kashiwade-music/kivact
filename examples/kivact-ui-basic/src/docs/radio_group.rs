//! Catalog page of Radio Group: basic, descriptions, horizontal and in a field.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Orientation;

pub(crate) fn page() -> Page {
    Page::new("radio-group", "Radio Group", "Components", include_str!("radio_group.rs"))
        .description("A component that chooses one of several options. It is the standard radio_group drawn in the Lyra look.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass the options and a binding. Moving focus with the arrow keys also selects, and disabled items are skipped.",
            basic,
        ))
        .example(Example::new(
            "description",
            "Description",
            "The description of each ListItem adds a note to the item.",
            description,
        ))
        .example(Example::new(
            "horizontal",
            "Horizontal",
            "With orientation set to Horizontal the items sit side by side.",
            horizontal,
        ))
        .example(Example::new(
            "field",
            "In a field",
            "Wrap it in field to give it a label and a description.",
            in_field,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let options = vec![
        ListItem::new("default", "Default"),
        ListItem::new("comfortable", "Comfortable"),
        ListItem::new("compact", "Compact"),
    ];
    radio_group("Density", options, signal(Some("comfortable")))
}
// end example

// example: description
fn description() -> impl IntoView {
    let options = vec![
        ListItem::new("default", "Default").with_description("Standard spacing."),
        ListItem::new("comfortable", "Comfortable"),
        ListItem::new("compact", "Compact").with_description("Fits more on screen."),
    ];
    radio_group("Density", options, signal(Some("default")))
}
// end example

// example: horizontal
fn horizontal() -> impl IntoView {
    let options = vec![
        ListItem::new("default", "Default"),
        ListItem::new("comfortable", "Comfortable"),
        ListItem::new("compact", "Compact"),
    ];
    radio_group("Density", options, signal(None)).orientation(Orientation::Horizontal)
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let options = vec![
        ListItem::new("default", "Default"),
        ListItem::new("comfortable", "Comfortable"),
        ListItem::new("compact", "Compact"),
    ];
    kivact::ui::field(
        "Density",
        radio_group("Density", options, signal(Some("compact"))),
    )
    .hint("Choose how tightly rows are packed.")
}
// end example
