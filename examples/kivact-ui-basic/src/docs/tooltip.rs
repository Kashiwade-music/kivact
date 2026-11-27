//! Catalog page of Tooltip: default, sides, icon buttons and keyboard shortcut.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Placement;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("tooltip", "Tooltip", "Components", include_str!("tooltip.rs"))
        .description("Shows a short note when the pointer rests on an element or it takes focus. It is the standard kivact::ui::tooltip in the Lyra look (inverted colors and an arrow pointing at the target), which wraps its target element.")
        .usage("use kivact::prelude::*;\nuse kivact::view::Placement;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass the text and the target to tooltip. It shows on hover or focus.",
            default,
        ))
        .example(Example::new(
            "sides",
            "Sides",
            "placement chooses the side it shows on.",
            sides,
        ))
        .example(Example::new(
            "icon-button",
            "Icon button",
            "An icon button that has only a name gets its meaning from the tooltip.",
            icon_button,
        ))
        .example(Example::new(
            "shortcut",
            "With keyboard shortcut",
            "content replaces the text with kivact::ui::kbd key caps in a row.",
            shortcut,
        ))
        .example(Example::new(
            "text-shortcut",
            "Shortcut text",
            "shortcut writes the keys after the label as key caps, with no content.",
            text_shortcut,
        ))
}

// example: default
fn default() -> impl IntoView {
    tooltip(
        "Add to library",
        button("Hover").variant(ButtonVariant::Outline),
    )
}
// end example

// example: sides
fn sides() -> impl IntoView {
    let side = |label: &'static str, placement: Placement| {
        tooltip(
            format!("Add to {}", label.to_lowercase()),
            button(label).variant(ButtonVariant::Outline),
        )
        .placement(placement)
    };
    row((
        side("Above", Placement::Above),
        side("Below", Placement::Below),
        side("Start", Placement::Start),
        side("End", Placement::End),
    ))
    .style(style().gap(8))
}
// end example

// example: icon-button
fn icon_button() -> impl IntoView {
    row((
        tooltip(
            "Settings",
            button("Settings")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("settings")),
        ),
        tooltip(
            "Notifications",
            button("Notifications")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("bell")),
        ),
        tooltip(
            "Delete",
            button("Delete")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("trash-2")),
        ),
    ))
    .style(style().gap(8))
}
// end example

// example: shortcut
fn shortcut() -> impl IntoView {
    tooltip("Save", button("Save").variant(ButtonVariant::Outline)).content(|| {
        row((
            text("Save"),
            basic::kbd_group((kivact::ui::kbd("Ctrl"), kivact::ui::kbd("S"))),
        ))
        .style(style().gap(8).align_items(kivact::style::Align::Center))
    })
}
// end example

// example: text-shortcut
fn text_shortcut() -> impl IntoView {
    tooltip("Undo", button("Undo")).shortcut("Ctrl+Z")
}
// end example
