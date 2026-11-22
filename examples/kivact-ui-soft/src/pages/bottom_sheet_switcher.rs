//! Role: The catalog page of Bottom Sheet Switcher.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "bottom-sheet-switcher",
        "Bottom Sheet Switcher",
        "Overlay",
        include_str!("overlays.rs"),
    )
    .example(Example::new(
        "bottom-sheet-switcher",
        "Bottom Sheet Switcher",
        "",
        overlays::example_bottom_sheet_switcher,
    ))
}
