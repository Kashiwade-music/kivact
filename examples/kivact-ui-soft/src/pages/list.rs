//! Role: The catalog page of List.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "list",
        "List",
        "Table & List",
        include_str!("collections.rs"),
    )
    .example(Example::new("list", "List", "", collections::example_list))
}
