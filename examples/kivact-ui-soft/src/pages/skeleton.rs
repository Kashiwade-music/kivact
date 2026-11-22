//! Role: The catalog page of Skeleton.
use super::feedback;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "skeleton",
        "Skeleton",
        "Feedback & Status",
        include_str!("feedback.rs"),
    )
    .example(Example::new(
        "skeleton",
        "Skeleton",
        "",
        feedback::example_skeleton,
    ))
}
