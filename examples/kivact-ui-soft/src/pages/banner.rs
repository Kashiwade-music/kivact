//! Role: The catalog page of Banner.
use super::feedback;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "banner",
        "Banner",
        "Feedback & Status",
        include_str!("feedback.rs"),
    )
    .example(Example::new(
        "banner",
        "Banner",
        "",
        feedback::example_banner,
    ))
}
