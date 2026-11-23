//! Role: The catalog page of Toolbar.
use super::controls;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("toolbar", "Toolbar", "Action", include_str!("controls.rs")).example(Example::new(
        "toolbar",
        "Toolbar",
        "",
        controls::example_toolbar,
    ))
}
