//! Role: The catalog page of Theme.
use super::utilities;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("theme", "Theme", "Utility", include_str!("utilities.rs")).example(Example::new(
        "theme",
        "Theme",
        "",
        utilities::example_theme,
    ))
}
