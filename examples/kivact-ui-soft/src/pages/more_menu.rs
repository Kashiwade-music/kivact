//! Role: The catalog page of More Menu.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "more-menu",
        "More Menu",
        "Action",
        include_str!("overlays.rs"),
    )
    .example(Example::new(
        "more-menu",
        "More Menu",
        "",
        overlays::example_more_menu,
    ))
}
