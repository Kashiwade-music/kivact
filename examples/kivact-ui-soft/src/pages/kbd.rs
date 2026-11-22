//! Role: The catalog page of Kbd.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("kbd", "Kbd", "Content", include_str!("content.rs")).example(Example::new(
        "kbd",
        "Kbd",
        "",
        content::example_kbd,
    ))
}
