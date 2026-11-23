//! Role: The catalog page of Code Block.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "code-block",
        "Code Block",
        "Content",
        include_str!("content.rs"),
    )
    .example(Example::new(
        "code-block",
        "Code Block",
        "",
        content::example_code_block,
    ))
}
