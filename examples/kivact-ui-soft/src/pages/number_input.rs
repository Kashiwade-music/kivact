//! Role: The catalog page of Number Input.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "number-input",
        "Number Input",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "number-input",
        "Number Input",
        "",
        forms::example_number_input,
    ))
}
