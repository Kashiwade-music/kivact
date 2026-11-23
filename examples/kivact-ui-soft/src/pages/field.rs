//! Role: The catalog page of Field.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("field", "Field", "Form Controls", include_str!("forms.rs")).example(Example::new(
        "field",
        "Field",
        "",
        forms::example_field,
    ))
}
