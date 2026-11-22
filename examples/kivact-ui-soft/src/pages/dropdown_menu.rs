//! Role: The catalog page of Dropdown Menu.
use super::overlays;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "dropdown-menu",
        "Dropdown Menu",
        "Action",
        include_str!("overlays.rs"),
    )
    .example(Example::new(
        "dropdown-menu",
        "Dropdown Menu",
        "",
        overlays::example_dropdown_menu,
    ))
}
