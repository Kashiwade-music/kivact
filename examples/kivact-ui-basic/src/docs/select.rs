//! Catalog page of Select: basic, sizes, disabled item and with icon.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("select", "Select", "Components", include_str!("select.rs"))
        .description("Chooses one item from a list of options. It is the standard kivact::ui::select in the Lyra look; the caller holds the choice as a key.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass a ListItem list and an Option key. The arrow keys move through the options, and Enter decides.",
            basic,
        ))
        .example(Example::new(
            "sizes",
            "Sizes",
            "Pass ControlSize::Small as size for a small selection field 28 px high.",
            sizes,
        ))
        .example(Example::new(
            "disabled-item",
            "Disabled item",
            "Set disabled on a ListItem to make an option that cannot be chosen.",
            disabled_item,
        ))
        .example(Example::new(
            "with-icon",
            "With icon",
            "icon puts a Lucide icon at the start of the selection field.",
            with_icon,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let fruit = signal(None::<&'static str>);
    let options = vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("blueberry", "Blueberry"),
        ListItem::new("grapes", "Grapes"),
        ListItem::new("pineapple", "Pineapple"),
    ];
    column((
        select(options, fruit)
            .label("Fruit")
            .placeholder("Select a fruit")
            .style(style().width(200)),
        basic::muted(move || format!("Selected: {}", fruit.get().unwrap_or("none"))),
    ))
    .style(style().gap(8))
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    let options = vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("grapes", "Grapes"),
    ];
    column((
        select(options.clone(), signal(Some("apple")))
            .label("Small fruit")
            .size(ControlSize::Small)
            .style(style().width(200)),
        select(options, signal(Some("banana")))
            .label("Default fruit")
            .style(style().width(200)),
    ))
    .style(style().gap(12))
}
// end example

// example: disabled-item
fn disabled_item() -> impl IntoView {
    let options = vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("blueberry", "Blueberry").disabled(true),
        ListItem::new("grapes", "Grapes").disabled(true),
        ListItem::new("pineapple", "Pineapple"),
    ];
    select(options, signal(None::<&'static str>))
        .label("Fruit")
        .placeholder("Select a fruit")
        .style(style().width(200))
}
// end example

// example: with-icon
fn with_icon() -> impl IntoView {
    let places = vec![
        ListItem::new("home", "Home"),
        ListItem::new("work", "Work"),
        ListItem::new("travel", "Travel"),
    ];
    select(places, signal(Some("home")))
        .label("Place")
        .icon(icon("map-pin"))
        .style(style().width(200))
}
// end example
