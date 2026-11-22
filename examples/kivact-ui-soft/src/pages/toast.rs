//! Role: The catalog page of Toast.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("toast", "Toast", "Overlay", include_str!("overlays.rs")).example(Example::new(
        "toast",
        "Toast",
        "",
        overlays::example_toast,
    ))
}
