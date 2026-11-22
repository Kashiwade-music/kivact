//! Role: The catalog page of Slider.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "slider",
        "Slider",
        "Form Controls",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "slider",
        "Slider",
        "",
        controls::example_slider,
    ))
}
