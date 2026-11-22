//! Role: The catalog page of Side Nav.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "side-nav",
        "Side Nav",
        "Navigation",
        include_str!("navigation.rs"),
    )
    .example(Example::new(
        "side-nav",
        "Side Nav",
        "",
        navigation::example_sidebar,
    ))
}
