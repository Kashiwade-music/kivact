//! Role: The catalog page of Toggle Button.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "toggle-button",
        "Toggle Button",
        "Action",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "toggle-button",
        "Toggle Button",
        "",
        controls::example_toggle_button,
    ))
}
