//! Role: The catalog page of Time Input.
use super::dates;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "time-input",
        "Time Input",
        "Form Controls",
        include_str!("dates.rs"),
    )
    .example(Example::new(
        "time-input",
        "Time Input",
        "",
        dates::example_time_input,
    ))
}
