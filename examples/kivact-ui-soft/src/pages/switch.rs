//! Role: The catalog page of Switch.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "switch",
        "Switch",
        "Form Controls",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "switch",
        "Switch",
        "",
        controls::example_switch,
    ))
}
