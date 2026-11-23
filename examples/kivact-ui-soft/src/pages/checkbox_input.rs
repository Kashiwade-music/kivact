//! Role: The catalog page of Checkbox Input.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "checkbox-input",
        "Checkbox Input",
        "Form Controls",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "checkbox-input",
        "Checkbox Input",
        "",
        controls::example_checkbox_input,
    ))
}
