//! Role: The catalog page of Text Area.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "text-area",
        "Text Area",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "text-area",
        "Text Area",
        "",
        forms::example_text_area,
    ))
}
