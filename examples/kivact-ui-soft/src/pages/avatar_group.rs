//! Role: The catalog page of Avatar Group.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "avatar-group",
        "Avatar Group",
        "Content",
        include_str!("content.rs"),
    )
    .example(Example::new(
        "avatar-group",
        "Avatar Group",
        "",
        content::example_avatar,
    ))
}
