//! Role: The catalog page of Internationalization Provider.
use super::utilities;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "internationalization-provider",
        "Internationalization Provider",
        "Utility",
        include_str!("utilities.rs"),
    )
    .example(Example::new(
        "internationalization-provider",
        "Internationalization Provider",
        "",
        utilities::example_internationalization_provider,
    ))
}
