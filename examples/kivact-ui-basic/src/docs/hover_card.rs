//! Catalog page of Hover Card: a preview that opens on hover.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Placement;
use kivact_ui_basic as basic;
use std::time::Duration;

pub(crate) fn page() -> Page {
    Page::new("hover-card", "Hover Card", "Components", include_str!("hover_card.rs"))
        .description("A card with more detail, shown after a delay when the pointer rests on a link or the link takes focus. It stays open while the pointer is on the card, and closes a little after the pointer leaves. Escape and a press outside close it at once.")
        .usage("use kivact::geometry::Edge;\nuse kivact::prelude::*;\nuse kivact_ui_basic as basic;\nuse std::time::Duration;")
        .example(Example::new(
            "default",
            "Default",
            "Pass a function that makes the content of the card and attach it to the trigger. By default it opens after 600 ms and closes 300 ms after the pointer leaves.",
            default,
        ))
        .example(Example::new(
            "delay",
            "Delay and placement",
            "open_delay and close_delay set the waits; placement (kivact::view::Placement) chooses the side it opens on.",
            delay,
        ))
}

// example: default
fn default() -> impl IntoView {
    kivact::ui::hover_card(|| {
        column((
            text("@nextjs").style(style().font_size(14).font_weight(600)),
            text("The React Framework – created and maintained by @vercel."),
            row((
                icon("calendar"),
                basic::muted("Joined December 2021").style(style().font_size(12)),
            ))
            .style(style().gap(8).align_items(Align::Center)),
        ))
        .style(style().gap(8))
    })
    .attach(button("@nextjs").variant(ButtonVariant::Link))
}
// end example

// example: delay
fn delay() -> impl IntoView {
    kivact::ui::hover_card(|| text("Opens after 100 ms and closes right away."))
        .open_delay(Duration::from_millis(100))
        .close_delay(Duration::ZERO)
        .placement(Placement::End)
        .attach(button("Hover me").variant(ButtonVariant::Outline))
}
// end example
