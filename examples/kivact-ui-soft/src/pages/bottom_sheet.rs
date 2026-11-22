//! Role: The catalog page of Bottom Sheet.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "bottom-sheet",
        "Bottom Sheet",
        "Overlay",
        include_str!("overlays.rs"),
    )
    .example(Example::new(
        "bottom-sheet",
        "Bottom Sheet",
        "",
        overlays::example_bottom_sheet,
    ))
}
