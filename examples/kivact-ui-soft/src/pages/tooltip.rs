//! Role: The catalog page of Tooltip.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("tooltip", "Tooltip", "Overlay", include_str!("overlays.rs")).example(Example::new(
        "tooltip",
        "Tooltip",
        "",
        overlays::example_tooltip,
    ))
}
