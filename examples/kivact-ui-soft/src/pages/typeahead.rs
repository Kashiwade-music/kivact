//! Role: The catalog page of Typeahead.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "typeahead",
        "Typeahead",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "typeahead",
        "Typeahead",
        "",
        forms::example_typeahead,
    ))
}
