//! Role: The catalog page of Metadata List.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "metadata-list",
        "Metadata List",
        "Table & List",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "metadata-list",
        "Metadata List",
        "",
        collections::example_metadata_list,
    ))
}
