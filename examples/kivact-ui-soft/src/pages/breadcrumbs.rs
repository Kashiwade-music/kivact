//! Role: The catalog page of Breadcrumbs.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "breadcrumbs",
        "Breadcrumbs",
        "Navigation",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "breadcrumbs",
        "Breadcrumbs",
        "",
        collections::example_breadcrumbs,
    ))
}
