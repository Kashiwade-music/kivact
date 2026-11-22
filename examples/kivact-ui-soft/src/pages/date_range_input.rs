//! Role: The catalog page of Date Range Input.
use super::dates;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "date-range-input",
        "Date Range Input",
        "Form Controls",
        include_str!("dates.rs"),
    )
    .example(Example::new(
        "date-range-input",
        "Date Range Input",
        "",
        dates::example_date_range_input,
    ))
}
