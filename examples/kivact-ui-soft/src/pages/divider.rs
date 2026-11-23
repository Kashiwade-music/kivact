//! Role: The catalog page of Divider.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "divider",
        "Divider",
        "Layout",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "divider",
        "Divider",
        "",
        collections::example_divider,
    ))
}
