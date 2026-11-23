//! Role: The catalog page of Carousel.
use super::collections;
use catalog_shell::{Example, Page};

pub(crate) fn page() -> Page {
    Page::new(
        "carousel",
        "Carousel",
        "Container",
        include_str!("collections.rs"),
    )
    .example(Example::new(
        "carousel",
        "Carousel",
        "",
        collections::example_carousel,
    ))
}
