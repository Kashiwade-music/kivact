//! Role: The catalog page of Badge.
use super::feedback;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "badge",
        "Badge",
        "Feedback & Status",
        include_str!("feedback.rs"),
    )
    .example(Example::new("badge", "Badge", "", feedback::example_badge))
}
