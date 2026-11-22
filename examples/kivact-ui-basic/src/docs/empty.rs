//! Catalog page of EmptyState: icon media, outline and with actions.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("empty", "EmptyState", "Components", include_str!("empty.rs"))
        .description("A component that tells the user there is nothing to show. It centers a media, a title, a description and actions.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "With Icon as media_variant the icon fits in a 32 px square.",
            default,
        ))
        .example(Example::new(
            "outline",
            "Outline",
            "Add a border with the border of style.",
            outline,
        ))
        .example(Example::new("actions", "Actions", "Put the actions in the content.", actions))
}

// example: default
fn default() -> impl IntoView {
    kivact::ui::empty_state("No Projects Yet")
        .icon(icon("folder"))
        .variant(basic::EmptyMediaVariant::Icon)
        .description("Get started by creating your first project.")
        .style(style().width(420))
}
// end example

// example: outline
fn outline() -> impl IntoView {
    kivact::ui::empty_state("No results")
        .icon(icon("search"))
        .variant(basic::EmptyMediaVariant::Icon)
        .description("Try adjusting your search terms.")
        .style(style().border(1, basic::BORDER))
        .style(style().width(420))
}
// end example

// example: actions
fn actions() -> impl IntoView {
    kivact::ui::empty_state("Inbox zero")
        .icon(icon("inbox"))
        .variant(basic::EmptyMediaVariant::Icon)
        .description("You are all caught up.")
        .actions({
            row((
                button("Compose"),
                button("Refresh").variant(ButtonVariant::Outline),
            ))
            .style(style().gap(8))
        })
        .style(style().width(420))
}
// end example
