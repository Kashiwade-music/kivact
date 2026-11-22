//! Role: The catalog page of Top Nav.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "top-nav",
        "Top Nav",
        "Navigation",
        include_str!("navigation.rs"),
    )
    .example(Example::new(
        "top-nav",
        "Top Nav",
        "",
        navigation::example_top_nav,
    ))
}
