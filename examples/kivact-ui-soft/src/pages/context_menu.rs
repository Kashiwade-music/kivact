//! Role: The catalog page of Context Menu.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "context-menu",
        "Context Menu",
        "Action",
        include_str!("overlays.rs"),
    )
    .example(Example::new(
        "context-menu",
        "Context Menu",
        "",
        overlays::example_context_menu,
    ))
}
