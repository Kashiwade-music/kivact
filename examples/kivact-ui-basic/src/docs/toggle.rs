//! Catalog page of Toggle: default, outline, sizes, icons and disabled.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::ui::toggle_button;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("toggle", "Toggle", "Components", include_str!("toggle.rs"))
        .description("A two-state button that stays pressed. It draws kivact::ui::toggle in the basic theme.")
        .usage("use kivact::prelude::*;\nuse kivact::ui::{ControlSize, toggle_button};\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass a name and a bool binding. The binding holds the pressed state.",
            basic,
        ))
        .example(Example::new(
            "outline",
            "Outline",
            "Pass basic::ToggleVariant::Outline as variant to draw a border.",
            outline,
        ))
        .example(Example::new(
            "sizes",
            "Sizes",
            "Small, Medium and Large for size are 28, 32 and 36 px high.",
            sizes,
        ))
        .example(Example::new(
            "disabled",
            "Disabled",
            "A disabled toggle is drawn faint and cannot be pressed.",
            disabled,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let on = signal(false);
    toggle_button("Bold", on)
        .icon(icon("bold"))
        .hide_label(true)
}
// end example

// example: outline
fn outline() -> impl IntoView {
    let on = signal(true);
    toggle_button("Italic", on)
        .variant(basic::ToggleVariant::Outline)
        .icon(icon("italic"))
}
// end example

// example: sizes
fn sizes() -> impl IntoView {
    row((
        toggle_button("Small", signal(false)).size(ControlSize::Small),
        toggle_button("Default", signal(false)),
        toggle_button("Large", signal(false)).size(ControlSize::Large),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    toggle_button("Underline", signal(false)).disabled(true)
}
// end example
