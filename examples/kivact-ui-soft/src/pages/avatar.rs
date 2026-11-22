//! Role: The catalog page of Avatar.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("avatar", "Avatar", "Content", include_str!("content.rs")).example(Example::new(
        "avatar",
        "Avatar",
        "",
        content::example_avatar,
    ))
}
