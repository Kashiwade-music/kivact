//! Catalog page of Switch: basic, description, disabled and settings.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("switch", "Switch", "Components", include_str!("switch.rs"))
        .description("A switch that turns on and off. It is the standard kivact::ui::switch in the Lyra look.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass a label and a signal. Click or Space toggles it.",
            basic,
        ))
        .example(Example::new(
            "description",
            "With description",
            "An example with a supporting sentence under the label.",
            description,
        ))
        .example(Example::new(
            "disabled",
            "Disabled",
            "A disabled switch is drawn faint and cannot be operated.",
            disabled,
        ))
        .example(Example::new(
            "invalid",
            "Invalid",
            "invalid set to true makes the border the destructive color and adds a faint ring.",
            invalid,
        ))
        .example(Example::new(
            "settings",
            "Settings list",
            "An example of a settings list with several switches. Each switch holds its state in its own signal.",
            settings,
        ))
        .example(Example::new(
            "field",
            "In a field",
            "With Horizontal in basic::field, the label and the description sit on the left and the switch on the right.",
            in_field,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let airplane = signal(false);
    column((
        switch("Airplane mode", airplane),
        text(move || {
            if airplane.get() {
                "Radios are off".to_string()
            } else {
                "Radios are on".to_string()
            }
        })
        .style(style().font_size(12)),
    ))
    .style(style().gap(12))
}
// end example

// example: description
fn description() -> impl IntoView {
    let emails = signal(true);
    column((
        switch("Marketing emails", emails),
        basic::muted("Receive emails about new products and features.")
            .style(style().padding_left(44)),
    ))
    .style(style().gap(4))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    column((
        switch("Disabled off", signal(false)).disabled(true),
        switch("Disabled on", signal(true)).disabled(true),
    ))
    .style(style().gap(12))
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    let agreed = signal(false);
    switch("Share usage data", agreed).invalid(move || !agreed.get())
}
// end example

// example: settings
fn settings() -> impl IntoView {
    column((
        switch("Share across devices", signal(true)),
        switch("Notify me about replies", signal(false)),
        switch("Use compact density", signal(false)),
    ))
    .style(style().gap(12))
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let on = signal(false);
    kivact::ui::field("Share across devices", switch("Share across devices", on))
        .hint("Focus is shared across devices.")
        .style(style().flex_direction(kivact::style::FlexDirection::Row))
        .style(style().width(420))
}
// end example
