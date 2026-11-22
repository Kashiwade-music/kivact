//! Role: The catalog page of Selectable Card.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "selectable-card",
        "Selectable Card",
        "Container",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "selectable-card",
        "Selectable Card",
        "",
        collections::example_selectable_card,
    ))
}
