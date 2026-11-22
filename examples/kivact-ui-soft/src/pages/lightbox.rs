//! Role: The catalog page of Lightbox.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "lightbox",
        "Lightbox",
        "Overlay",
        include_str!("overlays.rs"),
    )
    .example(Example::new(
        "lightbox",
        "Lightbox",
        "",
        overlays::example_lightbox,
    ))
}
