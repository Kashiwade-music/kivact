//! Role: The catalog page of File Input.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "file-input",
        "File Input",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "file-input",
        "File Input",
        "",
        forms::example_file_input,
    ))
}
