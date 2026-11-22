//! Catalog page of Spinner: default, in a button and with text.

use catalog_shell::{Example, Page};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page::new("spinner", "Spinner", "Components", include_str!("spinner.rs"))
        .description("A rotating Lucide loader-circle that shows a pending state. It is named 'Loading' with Role::Status, and it stops when reduced motion is on.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "Put kivact::ui::spinner() as it is. Its color follows the text color.",
            default,
        ))
        .example(Example::new(
            "in-button",
            "In a button",
            "The loading state of a button shows a spinner automatically.",
            in_button,
        ))
        .example(Example::new(
            "with-text",
            "With text",
            "An example of a spinner set beside a line of description text.",
            with_text,
        ))
}

// example: default
fn default() -> impl IntoView {
    kivact::ui::spinner()
}
// end example

// example: in-button
fn in_button() -> impl IntoView {
    row((
        button("Loading").busy(true),
        button("Please wait")
            .variant(ButtonVariant::Outline)
            .busy(true),
        button("Deleting").variant(ButtonVariant::Danger).busy(true),
    ))
    .style(style().gap(8))
}
// end example

// example: with-text
fn with_text() -> impl IntoView {
    row((
        kivact::ui::spinner(),
        text("Processing payment...").style(style().font_size(14)),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example
