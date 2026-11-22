//! Catalog page of Input Group: icons, text, buttons and block addons.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("input-group", "Input Group", "Components", include_str!("input_group.rs"))
        .description("A component that attaches text, icons or buttons before and after an input.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "icon",
            "Icon and text",
            "Put icons or text in start and end. The group, not the input, draws the frame and the focus ring.",
            icon,
        ))
        .example(Example::new(
            "button",
            "Button",
            "input_group_button places a pressable button inside the group.",
            button_addon,
        ))
        .example(Example::new(
            "textarea",
            "Textarea",
            "block_end puts actions below a text area.",
            textarea,
        ))
        .example(Example::new(
            "invalid",
            "Invalid",
            "invalid set to true makes the frame and the ring of the group the destructive color.",
            invalid,
        ))
}

// example: icon
fn icon() -> impl IntoView {
    let query = signal(String::new());
    kivact::ui::input_group(
        "Search",
        (kivact::ui::text_input(query)
            .label("Search")
            .variant(basic::InputGroupControl)
            .placeholder("Search..."),),
    )
    .start(kivact::view::icon("search"))
    .end(kivact::ui::input_addon("12 results"))
    .style(style().width(320))
}
// end example

// example: button
fn button_addon() -> impl IntoView {
    let url = signal(String::from("https://example.com"));
    let copied = signal(false);
    kivact::ui::input_group(
        "Link",
        (kivact::ui::text_input(url)
            .label("Link")
            .variant(basic::InputGroupControl),),
    )
    .end(
        kivact::ui::button("Copy")
            .variant(kivact::ui::ButtonVariant::Ghost)
            .variant(basic::InputGroupButtonSize::Xs)
            .style(style().margin_right(-4.8))
            .icon(kivact::view::icon(if copied.get() {
                "check"
            } else {
                "copy"
            }))
            .on_press(move || copied.set(true)),
    )
    .style(style().width(320))
}
// end example

// example: textarea
fn textarea() -> impl IntoView {
    let body = signal(String::new());
    kivact::ui::input_group(
        "Message",
        (kivact::ui::text_area(body)
            .label("Message")
            .variant(basic::InputGroupControl)
            .placeholder("Write a message"),),
    )
    .block_end(kivact::ui::input_addon("0 / 280"))
    .style(style().width(320))
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    let value = signal(String::from("not an email"));
    kivact::ui::input_group(
        "Email",
        (kivact::ui::text_input(value)
            .label("Email")
            .variant(basic::InputGroupControl),),
    )
    .start(kivact::view::icon("mail"))
    .invalid(true)
    .style(style().width(320))
}
// end example
