//! Role: The catalog page of Timer.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("timer", "Timer", "Content", include_str!("content.rs")).example(Example::new(
        "timer",
        "Timer",
        "",
        content::example_timer,
    ))
}
