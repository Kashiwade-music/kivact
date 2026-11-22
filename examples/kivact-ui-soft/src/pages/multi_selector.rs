//! Role: The catalog page of Multi Selector.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "multi-selector",
        "Multi Selector",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "multi-selector",
        "Multi Selector",
        "",
        forms::example_multi_selector,
    ))
}
