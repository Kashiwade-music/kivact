//! Role: The catalog page of Input Group.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "input-group",
        "Input Group",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "input-group",
        "Input Group",
        "",
        forms::example_input_group,
    ))
}
