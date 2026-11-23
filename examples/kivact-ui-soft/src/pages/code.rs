//! Role: The catalog page of Code.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("code", "Code", "Content", include_str!("content.rs")).example(Example::new(
        "code",
        "Code",
        "",
        content::example_code,
    ))
}
