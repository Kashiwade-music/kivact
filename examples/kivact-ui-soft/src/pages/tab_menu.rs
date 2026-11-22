//! Role: The catalog page of Tab Menu.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "tab-menu",
        "Tab Menu",
        "Navigation",
        include_str!("navigation.rs"),
    )
    .example(Example::new(
        "tab-menu",
        "Tab Menu",
        "",
        navigation::example_tab_menu,
    ))
}
