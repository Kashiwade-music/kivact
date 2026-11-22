//! Role: The catalog page of Text.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("text", "Text", "Content", include_str!("content.rs")).example(Example::new(
        "text",
        "Text",
        "",
        content::example_text,
    ))
}
