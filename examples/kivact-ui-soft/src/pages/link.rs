//! Role: The catalog page of Link.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("link", "Link", "Action", include_str!("content.rs")).example(Example::new(
        "link",
        "Link",
        "",
        content::example_link,
    ))
}
