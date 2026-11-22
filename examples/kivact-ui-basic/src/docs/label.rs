//! Catalog page of Label: a label for an input and a small form.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("label", "Label", "Components", include_str!("label.rs"))
        .description("A label that names an input. Pass a target, and pressing the label moves focus to the input.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "with-input",
            "With input",
            "Share a NodeRef between the input and the label's target. Pressing the label moves focus to the input.",
            with_input,
        ))
        .example(Example::new(
            "form",
            "In a form",
            "An example of a form in which each label and input pair is stacked. The labels are 12 px text with no extra line spacing.",
            form,
        ))
}

// example: with-input
fn with_input() -> impl IntoView {
    let field = node_ref();
    let email = signal(String::new());
    column((
        basic::label("Email address").target(field),
        text_input(email)
            .label("Email address")
            .placeholder("you@example.com")
            .with_node(|node| node.node_ref(field))
            .style(style().width(280)),
    ))
    .style(style().gap(8))
}
// end example

// example: form
fn form() -> impl IntoView {
    let first = node_ref();
    let last = node_ref();
    column((
        column((
            basic::label("First name").target(first),
            text_input(signal(String::new()))
                .label("First name")
                .with_node(|node| node.node_ref(first))
                .style(style().width(280)),
        ))
        .style(style().gap(8)),
        column((
            basic::label("Last name").target(last),
            text_input(signal(String::new()))
                .label("Last name")
                .with_node(|node| node.node_ref(last))
                .style(style().width(280)),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(16))
}
// end example
