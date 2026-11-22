//! Role: The catalog page of Step.
use super::navigation;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new("step", "Step", "Navigation", include_str!("navigation.rs")).example(Example::new(
        "step",
        "Step",
        "",
        navigation::example_stepper,
    ))
}
