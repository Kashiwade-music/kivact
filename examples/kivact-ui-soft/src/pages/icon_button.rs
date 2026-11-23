//! Role: The catalog page of Icon Button.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "icon-button",
        "Icon Button",
        "Action",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "icon-button",
        "Icon Button",
        "",
        controls::example_icon_button,
    ))
}
