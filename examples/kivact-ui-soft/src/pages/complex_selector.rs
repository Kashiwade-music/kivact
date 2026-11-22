//! Role: The catalog page of Complex Selector.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "complex-selector",
        "Complex Selector",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "complex-selector",
        "Complex Selector",
        "",
        forms::example_complex_selector,
    ))
}
