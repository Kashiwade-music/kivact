//! Role: The catalog page of Table.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "table",
        "Table",
        "Table & List",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "table",
        "Table",
        "",
        collections::example_table,
    ))
}
