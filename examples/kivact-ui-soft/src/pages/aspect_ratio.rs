//! Role: The catalog page of Aspect Ratio.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "aspect-ratio",
        "Aspect Ratio",
        "Layout",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "aspect-ratio",
        "Aspect Ratio",
        "",
        collections::example_aspect_ratio,
    ))
}
