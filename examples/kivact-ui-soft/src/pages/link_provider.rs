//! Role: The catalog page of Link Provider.
use super::utilities;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "link-provider",
        "Link Provider",
        "Utility",
        include_str!("utilities.rs"),
    )
    .example(Example::new(
        "link-provider",
        "Link Provider",
        "",
        utilities::example_link_provider,
    ))
}
