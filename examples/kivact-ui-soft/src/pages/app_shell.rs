//! Role: The catalog page of App Shell.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "app-shell",
        "App Shell",
        "Layout",
        include_str!("navigation.rs"),
    )
    .example(Example::new(
        "app-shell",
        "App Shell",
        "",
        navigation::example_app_shell,
    ))
}
