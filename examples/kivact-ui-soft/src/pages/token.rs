//! Role: The catalog page of Token.
use super::feedback;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("token", "Token", "Content", include_str!("feedback.rs")).example(Example::new(
        "token",
        "Token",
        "",
        feedback::example_token,
    ))
}
