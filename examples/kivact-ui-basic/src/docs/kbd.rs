//! Catalog page of Kbd: key caps, groups and use inside a button.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("kbd", "Kbd", "Components", include_str!("kbd.rs"))
        .description("A small key cap that shows a keyboard key or a shortcut.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass text to kivact::ui::kbd and it is one key cap.",
            default,
        ))
        .example(Example::new(
            "group",
            "Group",
            "basic::kbd_group combines several keys to show a shortcut.",
            group,
        ))
        .example(Example::new(
            "in-button",
            "In a button",
            "Placed as the children of a button, the key caps show the shortcut of the action.",
            in_button,
        ))
}

// example: default
fn default() -> impl IntoView {
    row((
        kivact::ui::kbd("Ctrl"),
        kivact::ui::kbd("Shift"),
        kivact::ui::kbd("Esc"),
        kivact::ui::kbd("⌘"),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: group
fn group() -> impl IntoView {
    column((
        basic::kbd_group((kivact::ui::kbd("Ctrl"), kivact::ui::kbd("B"))),
        basic::kbd_group((
            kivact::ui::kbd("Ctrl"),
            kivact::ui::kbd("Shift"),
            kivact::ui::kbd("P"),
        )),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: in-button
fn in_button() -> impl IntoView {
    row((
        button("Accept")
            .variant(ButtonVariant::Outline)
            .trailing(kivact::ui::kbd("⏎")),
        button("Cancel")
            .variant(ButtonVariant::Outline)
            .trailing(kivact::ui::kbd("Esc")),
    ))
    .style(style().gap(8))
}
// end example
