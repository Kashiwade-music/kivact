//! Catalog page of Input: basic, label, disabled, invalid and with a button.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("input", "Input", "Components", include_str!("input.rs"))
        .description("A field for one line of text. It is the standard kivact::ui::text_input in the Lyra look; the caller's signal holds the value.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass a label (the accessible name), a signal and a placeholder. The typed value can be read from the signal.",
            basic,
        ))
        .example(Example::new(
            "label",
            "With label",
            "Pass the input to the target of basic::label, and pressing the label moves focus to the input.",
            with_label,
        ))
        .example(Example::new(
            "disabled",
            "Disabled",
            "A disabled input is drawn faint and cannot be edited.",
            disabled,
        ))
        .example(Example::new(
            "invalid",
            "Invalid",
            "invalid set to true makes the border the destructive color and adds a faint destructive ring outside. Assistive technology hears that the value is invalid.",
            invalid,
        ))
        .example(Example::new(
            "with-button",
            "With button",
            "An input and a button side by side.",
            with_button,
        ))
        .example(Example::new(
            "field",
            "In a field",
            "Wrap the input in basic::field and the label, the description and the error stack vertically.",
            in_field,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let email = signal(String::new());
    column((
        text_input(email)
            .label("Email")
            .placeholder("Email")
            .style(style().width(320)),
        basic::muted(move || format!("Value: {}", email.get())),
    ))
    .style(style().gap(8))
}
// end example

// example: label
fn with_label() -> impl IntoView {
    let field = node_ref();
    let name = signal(String::new());
    column((
        basic::label("Display name").target(field),
        text_input(name)
            .label("Display name")
            .placeholder("Ada Lovelace")
            .with_node(|node| node.node_ref(field))
            .style(style().width(320)),
    ))
    .style(style().gap(8))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    text_input(signal(String::new()))
        .label("Email")
        .placeholder("Email")
        .disabled(true)
        .style(style().width(320))
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    let username = signal(String::from("ada lovelace"));
    column((
        text_input(username)
            .label("Username")
            .invalid(true)
            .style(style().width(320)),
        text("Usernames cannot contain spaces.")
            .style(move || style().font_size(12).color(basic::DESTRUCTIVE)),
    ))
    .style(style().gap(8))
}
// end example

// example: with-button
fn with_button() -> impl IntoView {
    let email = signal(String::new());
    row((
        text_input(email)
            .label("Email")
            .placeholder("Email")
            .style(style().width(240)),
        button("Subscribe").variant(ButtonVariant::Outline),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let email = signal(String::new());
    kivact::ui::field(
        "Email",
        text_input(email)
            .label("Email")
            .placeholder("you@example.com"),
    )
    .hint("We will never share your email.")
    .style(style().width(320))
}
// end example
