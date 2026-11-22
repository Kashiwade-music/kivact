//! Role: The catalog page of Hover Card.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "hover-card",
        "Hover Card",
        "Overlay",
        include_str!("overlays.rs"),
    )
    .example(Example::new(
        "hover-card",
        "Hover Card",
        "",
        overlays::example_hover_card,
    ))
}
