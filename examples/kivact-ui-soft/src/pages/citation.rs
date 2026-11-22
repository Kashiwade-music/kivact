//! Role: The catalog page of Citation.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "citation",
        "Citation",
        "Content",
        include_str!("content.rs"),
    )
    .example(Example::new(
        "citation",
        "Citation",
        "",
        content::example_citation,
    ))
}
