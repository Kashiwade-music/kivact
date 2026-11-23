//! Role: The catalog page of Dialog.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("dialog", "Dialog", "Overlay", include_str!("overlays.rs")).example(Example::new(
        "dialog",
        "Dialog",
        "",
        overlays::example_dialog,
    ))
}
