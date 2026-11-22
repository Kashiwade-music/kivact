//! Role: The catalog page of Nav Menu.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "nav-menu",
        "Nav Menu",
        "Navigation",
        include_str!("navigation.rs"),
    )
    .example(Example::new(
        "nav-menu",
        "Nav Menu",
        "",
        navigation::example_nav_menu,
    ))
}
