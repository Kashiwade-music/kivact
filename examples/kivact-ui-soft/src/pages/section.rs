//! Role: The catalog page of Section.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "section",
        "Section",
        "Layout",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "section",
        "Section",
        "",
        collections::example_section,
    ))
}
