//! Role: The catalog page of Clickable Card.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "clickable-card",
        "Clickable Card",
        "Container",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "clickable-card",
        "Clickable Card",
        "",
        collections::example_clickable_card,
    ))
}
