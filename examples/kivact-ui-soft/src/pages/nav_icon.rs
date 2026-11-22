//! Role: The catalog page of Nav Icon.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "nav-icon",
        "Nav Icon",
        "Navigation",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "nav-icon",
        "Nav Icon",
        "",
        collections::example_nav_icon,
    ))
}
