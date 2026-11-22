//! Role: The catalog page of Radio List.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "radio-list",
        "Radio List",
        "Form Controls",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "radio-list",
        "Radio List",
        "",
        controls::example_radio_list,
    ))
}
