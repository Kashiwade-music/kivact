//! Role: The catalog page of Syntax Theme.
use super::utilities;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "syntax-theme",
        "Syntax Theme",
        "Utility",
        include_str!("utilities.rs"),
    )
    .example(Example::new(
        "syntax-theme",
        "Syntax Theme",
        "",
        utilities::example_syntax_theme,
    ))
}
