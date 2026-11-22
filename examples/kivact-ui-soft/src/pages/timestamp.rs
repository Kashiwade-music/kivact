//! Role: The catalog page of Timestamp.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "timestamp",
        "Timestamp",
        "Content",
        include_str!("content.rs"),
    )
    .example(Example::new(
        "timestamp",
        "Timestamp",
        "",
        content::example_timestamp,
    ))
}
