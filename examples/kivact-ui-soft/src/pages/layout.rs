//! Role: The catalog page of Layout.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("layout", "Layout", "Layout", include_str!("collections.rs")).example(Example::new(
        "layout",
        "Layout",
        "",
        collections::example_layout,
    ))
}
