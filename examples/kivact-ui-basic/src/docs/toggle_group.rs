//! Catalog page of Toggle Group: multiple, single, outline and vertical.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Orientation;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("toggle-group", "Toggle Group", "Components", include_str!("toggle_group.rs"))
        .description("A component that groups related toggles and chooses one or several of them.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::ControlSize;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "multiple",
            "Multiple",
            "A toggle_group that takes a Vec can press several at once. The arrow keys move the focus, and Space presses.",
            multiple,
        ))
        .example(Example::new(
            "single",
            "Single",
            "A toggle_group that takes an Option can press at most one. Pressing it again releases it.",
            single,
        ))
        .example(Example::new(
            "sizes",
            "Spacing and size",
            "spacing is the gap in steps of 4 px, and size is the size of every item.",
            spacing,
        ))
        .example(Example::new(
            "vertical",
            "Vertical",
            "With Vertical as orientation the items stack vertically.",
            vertical,
        ))
}

// example: multiple
fn multiple() -> impl IntoView {
    let items = vec![
        ListItem::new("bold", "Bold").with_icon("bold"),
        ListItem::new("italic", "Italic").with_icon("italic"),
        ListItem::new("underline", "Underline").with_icon("underline"),
    ];
    kivact::ui::toggle_group("Options", items, signal(vec!["bold"]))
        .icon_only(true)
        .name("Text style")
}
// end example

// example: single
fn single() -> impl IntoView {
    let items = vec![
        ListItem::new("left", "Left"),
        ListItem::new("center", "Center"),
        ListItem::new("right", "Right"),
    ];
    kivact::ui::toggle_group("Options", items, signal(Some("left")))
        .variant(basic::ToggleVariant::Outline)
        .spacing(0.0)
        .name("Alignment")
}
// end example

// example: sizes
fn spacing() -> impl IntoView {
    let items = vec![
        ListItem::new("bold", "Bold"),
        ListItem::new("italic", "Italic"),
        ListItem::new("underline", "Underline"),
    ];
    kivact::ui::toggle_group("Options", items, signal(vec!["italic"]))
        .size(ControlSize::Small)
        .spacing(1.0)
        .name("Text style")
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    let items = vec![
        ListItem::new("bold", "Bold"),
        ListItem::new("italic", "Italic"),
        ListItem::new("underline", "Underline"),
    ];
    kivact::ui::toggle_group("Options", items, signal(None))
        .variant(basic::ToggleVariant::Outline)
        .orientation(Orientation::Vertical)
        .spacing(0.0)
        .name("Text style")
}
// end example
