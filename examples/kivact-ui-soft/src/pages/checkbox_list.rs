//! Role: The catalog page of Checkbox List.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "checkbox-list",
        "Checkbox List",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "checkbox-list",
        "Checkbox List",
        "",
        forms::example_checkbox_list,
    ))
}
