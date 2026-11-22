//! Role: The catalog page of Heading.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("heading", "Heading", "Content", include_str!("content.rs")).example(Example::new(
        "heading",
        "Heading",
        "",
        content::example_heading,
    ))
}
