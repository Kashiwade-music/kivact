//! Role: The catalog page of Spinner.
use super::feedback;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "spinner",
        "Spinner",
        "Feedback & Status",
        include_str!("feedback.rs"),
    )
    .example(Example::new(
        "spinner",
        "Spinner",
        "",
        feedback::example_spinner,
    ))
}
