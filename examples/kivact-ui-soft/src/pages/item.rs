//! Role: The catalog page of Item.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "item",
        "Item",
        "Table & List",
        include_str!("collections.rs"),
    )
    .example(Example::new("item", "Item", "", collections::example_list))
}
