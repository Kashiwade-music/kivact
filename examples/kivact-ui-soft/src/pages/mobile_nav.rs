//! Role: The catalog page of Mobile Nav.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "mobile-nav",
        "Mobile Nav",
        "Navigation",
        include_str!("navigation.rs"),
    )
    .example(Example::new(
        "mobile-nav",
        "Mobile Nav",
        "",
        navigation::example_mobile_nav,
    ))
}
