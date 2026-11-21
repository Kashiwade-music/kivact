//! Role: The page of the Creative catalog.

use super::common::specs;
use super::feedback::*;
use catalog_shell::{Example, Page};
use kivact::prelude::{IntoView, signal};
use kivact::style::GridPlacement;
use kivact::style::style;

pub(crate) fn page() -> Page {
    Page::new(
        "dialogs-popovers",
        "Dialogs & Popovers",
        "Feedback",
        include_str!("dialogs_popovers.rs"),
    )
    .description("Windows, popovers, tooltips and banners.")
    .example(Example::new(
        "dialogs-popovers",
        "Dialogs & Popovers",
        "",
        demo,
    ))
}

// example: dialogs-popovers
fn demo() -> impl IntoView {
    let status = signal(String::new());
    specs((
        dialogs_metrics_and_descriptions(),
        dialogs_dialog(status),
        dialogs_popover(),
        dialogs_tooltip(),
        dialogs_banner().style(style().grid_column(GridPlacement::span(2)).min_height(250)),
    ))
}
// end example
