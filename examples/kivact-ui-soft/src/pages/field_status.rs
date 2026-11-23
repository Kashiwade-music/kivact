//! Role: The catalog page of Field Status.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "field-status",
        "Field Status",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "field-status",
        "Field Status",
        "",
        forms::example_field_status,
    ))
}
