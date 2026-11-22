//! Role: The catalog page of Tree List.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "tree-list",
        "Tree List",
        "Table & List",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "tree-list",
        "Tree List",
        "",
        collections::example_tree_list,
    ))
}
