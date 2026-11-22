//! Role: The catalog page of Popover.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("popover", "Popover", "Overlay", include_str!("overlays.rs")).example(Example::new(
        "popover",
        "Popover",
        "",
        overlays::example_popover,
    ))
}
