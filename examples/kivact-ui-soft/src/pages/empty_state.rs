//! Role: The catalog page of Empty State.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "empty-state",
        "Empty State",
        "Content",
        include_str!("content.rs"),
    )
    .example(Example::new(
        "empty-state",
        "Empty State",
        "",
        content::example_empty_state,
    ))
}
