//! Role: The catalog page of Center.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("center", "Center", "Layout", include_str!("collections.rs")).example(Example::new(
        "center",
        "Center",
        "",
        collections::example_center,
    ))
}
