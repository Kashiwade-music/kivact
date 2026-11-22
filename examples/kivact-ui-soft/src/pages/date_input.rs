//! Role: The catalog page of Date Input.
use super::dates;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "date-input",
        "Date Input",
        "Form Controls",
        include_str!("dates.rs"),
    )
    .example(Example::new(
        "date-input",
        "Date Input",
        "",
        dates::example_date_input,
    ))
}
