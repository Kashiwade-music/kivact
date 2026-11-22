//! Role: The catalog page of Pagination.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "pagination",
        "Pagination",
        "Navigation",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "pagination",
        "Pagination",
        "",
        collections::example_pagination,
    ))
}
