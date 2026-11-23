//! Role: The catalog page of Button Group.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "button-group",
        "Button Group",
        "Action",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "button-group",
        "Button Group",
        "",
        controls::example_button_group,
    ))
}
