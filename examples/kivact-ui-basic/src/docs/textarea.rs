//! Catalog page of Textarea: basic, label, disabled and with a button.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("textarea", "Textarea", "Components", include_str!("textarea.rs"))
        .description("A field for several lines of text. It is the standard kivact::ui::text_area in the Lyra look.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass a label, a signal and a placeholder. Enter starts a new line, and the height grows from 64 px with the number of lines.",
            basic,
        ))
        .example(Example::new(
            "label",
            "With label",
            "Pass the input to the target of basic::label so that moving from the label moves focus to the input.",
            with_label,
        ))
        .example(Example::new(
            "disabled",
            "Disabled",
            "A disabled input is drawn faint and cannot be edited.",
            disabled,
        ))
        .example(Example::new(
            "with-button",
            "With button",
            "An example with a send button below the input. The character count comes from the signal.",
            with_button,
        ))
        .example(Example::new(
            "field",
            "In a field",
            "Give it a label and a description with basic::field.",
            in_field,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    text_area(signal(String::new()))
        .label("Message")
        .placeholder("Type your message here.")
        .style(style().width(360))
}
// end example

// example: label
fn with_label() -> impl IntoView {
    let field = node_ref();
    column((
        basic::label("Your message").target(field),
        text_area(signal(String::new()))
            .label("Your message")
            .placeholder("Type your message here.")
            .with_node(|node| node.node_ref(field))
            .style(style().width(360)),
        basic::muted("Your message will be copied to the support team."),
    ))
    .style(style().gap(8))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    text_area(signal(String::new()))
        .label("Message")
        .placeholder("Type your message here.")
        .disabled(true)
        .style(style().width(360))
}
// end example

// example: with-button
fn with_button() -> impl IntoView {
    let message = signal(String::new());
    column((
        text_area(message)
            .label("Your message")
            .placeholder("Type your message here.")
            .style(style().width(360)),
        basic::muted(move || format!("{} characters", message.with(|text| text.chars().count()))),
        button("Send message"),
    ))
    .style(style().gap(8))
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let bio = signal(String::new());
    kivact::ui::field(
        "Bio",
        text_area(bio)
            .label("Bio")
            .placeholder("Tell us about yourself"),
    )
    .hint("You can mention people with @.")
    .style(style().width(320))
}
// end example
