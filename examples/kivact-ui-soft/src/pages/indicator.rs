//! Role: The catalog page of Indicator.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "indicator",
        "Indicator",
        "Form Controls",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "indicator",
        "Indicator",
        "",
        controls::example_indicator,
    ))
}
