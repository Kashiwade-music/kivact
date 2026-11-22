//! Role: The catalog page of Card.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("card", "Card", "Container", include_str!("collections.rs")).example(Example::new(
        "card",
        "Card",
        "",
        collections::example_card,
    ))
}
