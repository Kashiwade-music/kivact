//! Role: The catalog page of Layer.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("layer", "Layer", "Utility", include_str!("overlays.rs")).example(Example::new(
        "layer",
        "Layer",
        "",
        overlays::example_popover,
    ))
}
