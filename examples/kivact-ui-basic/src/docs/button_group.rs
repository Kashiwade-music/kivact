//! Catalog page of Button Group: horizontal, vertical, separator and text.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Orientation;

pub(crate) fn page() -> Page {
    Page::new("button-group", "Button Group", "Components", include_str!("button_group.rs"))
        .description("Groups related buttons side by side. Each child after the first drops the border that touches the one before it, so the boundary is one line. button_group takes a tuple of buttons (the kinds may differ) or a Vec, not one view, because it lays out each child.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Pass a tuple of buttons to button_group.",
            default,
        ))
        .example(Example::new(
            "vertical",
            "Vertical",
            "Pass Orientation::Vertical to orientation to stack them. The separators turn into horizontal lines.",
            vertical,
        ))
        .example(Example::new(
            "separator",
            "Separator",
            "button_group_separator puts a line of the input color between the buttons. Use it to separate Secondary buttons that have no border. In a row group the line is vertical.",
            separator,
        ))
        .example(Example::new(
            "text",
            "Text",
            "button_group_text is a text cell that is not an action of the group (shadcn's ButtonGroupText). Use it for a prefix or a suffix, such as https:// before a URL field or a unit after a number. The group gives it the height of the neighboring buttons and fields, and the text is centered vertically.",
            with_text,
        ))
}

// example: default
fn default() -> impl IntoView {
    kivact::ui::button_group(
        "Actions",
        (
            button("Archive").variant(ButtonVariant::Outline),
            button("Report").variant(ButtonVariant::Outline),
            button("Snooze").variant(ButtonVariant::Outline),
        ),
    )
}
// end example

// example: vertical
fn vertical() -> impl IntoView {
    kivact::ui::button_group(
        "Actions",
        (
            button("Top").variant(ButtonVariant::Outline),
            kivact::ui::button_group_separator(),
            button("Middle").variant(ButtonVariant::Outline),
            button("Bottom").variant(ButtonVariant::Outline),
        ),
    )
    .orientation(Orientation::Vertical)
}
// end example

// example: separator
fn separator() -> impl IntoView {
    kivact::ui::button_group(
        "Actions",
        (
            button("Copy").variant(ButtonVariant::Secondary),
            kivact::ui::button_group_separator(),
            button("Paste").variant(ButtonVariant::Secondary),
        ),
    )
}
// end example

// example: text
fn with_text() -> impl IntoView {
    let url = signal(String::from("example.com"));
    kivact::ui::button_group(
        "Actions",
        (
            kivact::ui::button_group_text("https://"),
            text_input(url).label("URL").style(style().width(160)),
            button("Go").variant(ButtonVariant::Outline),
        ),
    )
}
// end example
