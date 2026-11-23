//! Role: The catalog page of Markdown.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "markdown",
        "Markdown",
        "Content",
        include_str!("content.rs"),
    )
    .example(Example::new(
        "markdown",
        "Markdown",
        "",
        content::example_markdown,
    ))
}
