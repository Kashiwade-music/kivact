//! Role: The catalog page of Overflow List.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "overflow-list",
        "Overflow List",
        "Table & List",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "overflow-list",
        "Overflow List",
        "",
        collections::example_overflow_list,
    ))
}
