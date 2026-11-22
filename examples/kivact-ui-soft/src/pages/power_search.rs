//! Role: The catalog page of Power Search.
use super::forms;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "power-search",
        "Power Search",
        "Form Controls",
        include_str!("forms.rs"),
    )
    .example(Example::new(
        "power-search",
        "Power Search",
        "",
        forms::example_power_search,
    ))
}
