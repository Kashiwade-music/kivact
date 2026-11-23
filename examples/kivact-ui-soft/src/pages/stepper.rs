//! Role: The catalog page of Stepper.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "stepper",
        "Stepper",
        "Navigation",
        include_str!("navigation.rs"),
    )
    .example(Example::new(
        "stepper",
        "Stepper",
        "",
        navigation::example_stepper,
    ))
}
