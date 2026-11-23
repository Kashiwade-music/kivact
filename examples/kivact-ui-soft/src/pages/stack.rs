//! Role: The catalog page of Stack.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("stack", "Stack", "Layout", include_str!("collections.rs")).example(Example::new(
        "stack",
        "Stack",
        "",
        collections::example_stack,
    ))
}
