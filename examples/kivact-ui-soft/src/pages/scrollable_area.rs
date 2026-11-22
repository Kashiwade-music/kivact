//! Role: The catalog page of Scrollable Area.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "scrollable-area",
        "Scrollable Area",
        "Layout",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "scrollable-area",
        "Scrollable Area",
        "",
        collections::example_scrollable_area,
    ))
}
