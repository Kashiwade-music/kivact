//! Catalog page of Field: label, description, error, orientation and sets.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("field", "Field", "Components", include_str!("field.rs"))
        .description("A form section that groups a label, a description and an error with an input.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass an input to field(label, control) and add a hint. Stacked vertically is the default.",
            basic,
        ))
        .example(Example::new(
            "error",
            "Error",
            "Pass Some to error to show the message as a Role::Alert and make the label the destructive color.",
            error,
        ))
        .example(Example::new(
            "horizontal",
            "Horizontal",
            "The standard field with the direction of style set lays the parts out side by side. For a richer or responsive layout, compose the parts with row and column in the caller.",
            horizontal,
        ))
        .example(Example::new(
            "set",
            "Field set",
            "field_set and field_group group several fields. field_separator is a dividing line.",
            set,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let username = signal(String::new());
    kivact::ui::field(
        "Username",
        text_input(username).label("Username").placeholder("shadcn"),
    )
    .hint("Choose a unique username for your account.")
    .style(style().width(320))
}
// end example

// example: error
fn error() -> impl IntoView {
    let email = signal(String::from("ada@"));
    let message = memo(move || {
        (!email.get().contains("@example.com")).then(|| "Use an @example.com address.".to_string())
    });
    kivact::ui::field(
        "Email",
        text_input(email)
            .label("Email")
            .invalid(move || message.get().is_some()),
    )
    .error(message)
    .style(style().width(320))
}
// end example

// example: horizontal
fn horizontal() -> impl IntoView {
    let on = signal(true);
    kivact::ui::field("Marketing emails", switch("Marketing emails", on))
        .hint("Receive news and offers.")
        .style(style().flex_direction(kivact::style::FlexDirection::Row))
        .style(style().width(420))
}
// end example

// example: set
fn set() -> impl IntoView {
    let (name, city) = (signal(String::new()), signal(String::new()));
    basic::field_set(basic::field_group((
        kivact::ui::field("Name", text_input(name).label("Name")),
        basic::field_separator().text("Or"),
        kivact::ui::field("City", text_input(city).label("City")).hint("Where you live."),
    )))
    .legend("Address")
    .description("Used to ship your order.")
    .style(style().width(320))
}
// end example
