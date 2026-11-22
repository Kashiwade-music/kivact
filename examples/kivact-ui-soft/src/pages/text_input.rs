//! Role: The catalog page of Text Input.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "text-input",
        "Text Input",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "text-input",
        "Text Input",
        "",
        forms::example_text_input,
    ))
}
