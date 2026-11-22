//! Role: The catalog page of Button.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("button", "Button", "Action", include_str!("controls.rs")).example(Example::new(
        "button",
        "Button",
        "",
        controls::example_button,
    ))
}
