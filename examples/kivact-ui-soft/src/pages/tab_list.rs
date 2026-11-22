//! Role: The catalog page of Tab List.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "tab-list",
        "Tab List",
        "Navigation",
        include_str!("navigation.rs"),
    )
    .example(Example::new(
        "tab-list",
        "Tab List",
        "",
        navigation::example_tab_list,
    ))
}
