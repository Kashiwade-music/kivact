//! Role: The catalog page of Thumbnail.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "thumbnail",
        "Thumbnail",
        "Content",
        include_str!("content.rs"),
    )
    .example(Example::new(
        "thumbnail",
        "Thumbnail",
        "",
        content::example_thumbnail,
    ))
}
