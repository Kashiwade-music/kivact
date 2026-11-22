//! Role: The catalog page of Blockquote.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "blockquote",
        "Blockquote",
        "Content",
        include_str!("content.rs"),
    )
    .example(Example::new(
        "blockquote",
        "Blockquote",
        "",
        content::example_blockquote,
    ))
}
