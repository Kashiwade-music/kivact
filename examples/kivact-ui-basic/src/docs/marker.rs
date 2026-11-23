//! Catalog page of Marker: default, separator, border and icon.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("marker", "Marker", "Components", include_str!("marker.rs"))
        .description("A mark or a divider placed in running text. It is used for things such as the date divider in a conversation.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "variants",
            "Variants",
            "The MarkerVariant values Default, Separator and Border.",
            variants,
        ))
        .example(Example::new(
            "icon",
            "With icon",
            "icon adds a 14 px icon before the content.",
            with_icon,
        ))
}

// example: variants
fn variants() -> impl IntoView {
    column((
        basic::marker("Default"),
        basic::marker("Today").variant(basic::MarkerVariant::Separator),
        basic::marker("Border").variant(basic::MarkerVariant::Border),
    ))
    .style(style().gap(16).width(420))
}
// end example

// example: icon
fn with_icon() -> impl IntoView {
    column((
        basic::marker("Last edited 2 minutes ago").icon(icon("clock")),
        basic::marker("Pinned")
            .icon(icon("bookmark"))
            .variant(basic::MarkerVariant::Separator),
    ))
    .style(style().gap(16).width(420))
}
// end example
