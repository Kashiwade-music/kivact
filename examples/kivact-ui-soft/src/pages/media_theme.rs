//! Role: The catalog page of Media Theme.
use super::utilities;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "media-theme",
        "Media Theme",
        "Utility",
        include_str!("utilities.rs"),
    )
    .example(Example::new(
        "media-theme",
        "Media Theme",
        "",
        utilities::example_media_theme,
    ))
}
