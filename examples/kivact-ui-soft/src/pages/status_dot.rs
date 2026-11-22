//! Role: The catalog page of Status Dot.
use super::feedback;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "status-dot",
        "Status Dot",
        "Feedback & Status",
        include_str!("feedback.rs"),
    )
    .example(Example::new(
        "status-dot",
        "Status Dot",
        "",
        feedback::example_status_dot,
    ))
}
