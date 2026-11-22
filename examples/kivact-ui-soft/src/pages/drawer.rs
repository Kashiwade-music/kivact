//! Role: The catalog page of Drawer.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("drawer", "Drawer", "Overlay", include_str!("overlays.rs")).example(Example::new(
        "drawer",
        "Drawer",
        "",
        overlays::example_drawer,
    ))
}
