//! Role: The catalog page of Form Layout.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "form-layout",
        "Form Layout",
        "Layout",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "form-layout",
        "Form Layout",
        "",
        collections::example_form_layout,
    ))
}
