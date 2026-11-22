//! Role: The catalog page of Selector.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "selector",
        "Selector",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "selector",
        "Selector",
        "",
        forms::example_selector,
    ))
}
