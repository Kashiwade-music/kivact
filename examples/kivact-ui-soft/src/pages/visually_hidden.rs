//! Role: The catalog page of Visually Hidden.
use super::content;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "visually-hidden",
        "Visually Hidden",
        "Utility",
        include_str!("content.rs"),
    )
    .example(Example::new(
        "visually-hidden",
        "Visually Hidden",
        "",
        content::example_visually_hidden,
    ))
}
