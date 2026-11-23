//! Catalog page of Checkbox: basic, description, disabled and select all.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("checkbox", "Checkbox", "Components", include_str!("checkbox.rs"))
        .description("A checkbox that turns on and off. It is the standard kivact::ui::checkbox in the Lyra look; the caller's signal holds the state.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass a label and a signal and it works. Click or Space toggles it.",
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
            "A disabled checkbox is drawn faint and cannot be operated.",
            disabled,
        ))
        .example(Example::new(
            "invalid",
            "Invalid",
            "invalid set to true makes the border the destructive color and adds a faint ring. Assistive technology hears it too.",
            invalid,
        ))
        .example(Example::new(
            "select-all",
            "Select all",
            "indeterminate shows that only some of the items are selected. The state of the parent is derived from the signals of the children.",
            select_all,
        ))
        .example(Example::new(
            "field",
            "In a field",
            "Put several checkboxes in a basic::field_set and give it a title and a description.",
            in_field,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let accepted = signal(false);
    column((
        checkbox("Accept terms and conditions", accepted),
        text(move || {
            if accepted.get() {
                "Accepted".to_string()
            } else {
                "Not accepted".to_string()
            }
        })
        .style(style().font_size(12)),
    ))
    .style(style().gap(12))
}
// end example

// example: description
fn description() -> impl IntoView {
    let marketing = signal(true);
    column((
        checkbox("Marketing emails", marketing),
        basic::muted("Receive product news and special offers.").style(style().padding_left(24)),
    ))
    .style(style().gap(4))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    column((
        checkbox("Enable notifications", signal(false)).disabled(true),
        checkbox("Sync across devices", signal(true)).disabled(true),
    ))
    .style(style().gap(12))
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    let accepted = signal(false);
    checkbox("Accept terms and conditions", accepted).invalid(move || !accepted.get())
}
// end example

// example: select-all
fn select_all() -> impl IntoView {
    let items = [signal(true), signal(false), signal(false)];
    let count = move || items.iter().filter(|item| item.get()).count();
    let all = binding(
        move || count() == items.len(),
        move |on: bool| items.iter().for_each(|item| item.set(on)),
    );
    column((
        checkbox("Select all", all).indeterminate(move || count() > 0 && count() < items.len()),
        column((
            checkbox("Desktop", items[0]),
            checkbox("Documents", items[1]),
            checkbox("Downloads", items[2]),
        ))
        .style(style().gap(12).padding_left(24)),
    ))
    .style(style().gap(12))
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let (a, b) = (signal(true), signal(false));
    basic::field_set(
        column((checkbox("Hard disks", a), checkbox("External disks", b))).style(style().gap(12)),
    )
    .legend("Show on desktop")
    .description("Select the items to show on the desktop.")
}
// end example
