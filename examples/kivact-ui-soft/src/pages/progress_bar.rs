//! Role: The catalog page of Progress Bar.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "progress-bar",
        "Progress Bar",
        "Feedback & Status",
        include_str!("controls.rs"),
    )
    .example(Example::new(
        "progress-bar",
        "Progress Bar",
        "",
        controls::example_progress_bar,
    ))
}
