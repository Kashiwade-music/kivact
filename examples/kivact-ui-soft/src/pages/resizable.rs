//! Role: The catalog page of Resizable.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "resizable",
        "Resizable",
        "Layout",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "resizable",
        "Resizable",
        "",
        collections::example_resizable,
    ))
}
