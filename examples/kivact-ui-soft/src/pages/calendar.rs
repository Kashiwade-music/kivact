//! Role: The catalog page of Calendar.
use super::dates;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "calendar",
        "Calendar",
        "Form Controls",
        include_str!("dates.rs"),
    )
    .example(Example::new(
        "calendar",
        "Calendar",
        "",
        dates::example_calendar,
    ))
}
