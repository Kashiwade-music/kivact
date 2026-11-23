//! Role: The catalog page of Overlay.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("overlay", "Overlay", "Overlay", include_str!("overlays.rs")).example(Example::new(
        "overlay",
        "Overlay",
        "",
        overlays::example_overlay,
    ))
}
