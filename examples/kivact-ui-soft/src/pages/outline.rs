//! Role: The catalog page of Outline.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "outline",
        "Outline",
        "Navigation",
        include_str!("navigation.rs"),
    )
    .example(Example::new(
        "outline",
        "Outline",
        "",
        navigation::example_outline,
    ))
}
