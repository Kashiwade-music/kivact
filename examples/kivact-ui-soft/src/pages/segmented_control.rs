//! Role: The catalog page of Segmented Control.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "segmented-control",
        "Segmented Control",
        "Action",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "segmented-control",
        "Segmented Control",
        "",
        controls::example_segmented_control,
    ))
}
