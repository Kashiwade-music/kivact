//! Role: The catalog page of Alert Dialog.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "alert-dialog",
        "Alert Dialog",
        "Overlay",
        include_str!("overlays.rs"),
    )
    .example(Example::new(
        "alert-dialog",
        "Alert Dialog",
        "",
        overlays::example_alert_dialog,
    ))
}
