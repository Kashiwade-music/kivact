//! Role: The catalog page of Grid.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("grid", "Grid", "Layout", include_str!("collections.rs")).example(Example::new(
        "grid",
        "Grid",
        "",
        collections::example_grid,
    ))
}
