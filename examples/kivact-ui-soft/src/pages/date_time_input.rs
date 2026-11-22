//! Role: The catalog page of Date Time Input.
use super::dates;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "date-time-input",
        "Date Time Input",
        "Form Controls",
        include_str!("dates.rs"),
    )
    .example(Example::new(
        "date-time-input",
        "Date Time Input",
        "",
        dates::example_date_time_input,
    ))
}
