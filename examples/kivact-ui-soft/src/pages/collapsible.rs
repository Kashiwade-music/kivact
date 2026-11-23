//! Role: The catalog page of Collapsible.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "collapsible",
        "Collapsible",
        "Container",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "collapsible",
        "Collapsible",
        "",
        collections::example_collapsible,
    ))
}
