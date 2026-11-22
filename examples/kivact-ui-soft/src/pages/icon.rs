//! Role: The catalog page of Icon.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("icon", "Icon", "Content", include_str!("content.rs")).example(Example::new(
        "icon",
        "Icon",
        "",
        content::example_icon,
    ))
}
