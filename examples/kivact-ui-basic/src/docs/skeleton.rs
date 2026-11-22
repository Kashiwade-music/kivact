//! Catalog page of Skeleton: an avatar row and a card placeholder.

use catalog_shell::{Example, Page};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page::new("skeleton", "Skeleton", "Components", include_str!("skeleton.rs"))
        .description("A placeholder that shows the shape of content while it loads. The caller sets its size with style. It does not pulse when reduced motion is on.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "avatar",
            "Avatar",
            "A placeholder of a round icon and two lines of text.",
            avatar,
        ))
        .example(Example::new(
            "card",
            "Card",
            "A placeholder of a card with an image and two lines of text.",
            card,
        ))
}

// example: avatar
fn avatar() -> impl IntoView {
    row((
        kivact::ui::skeleton().style(style().size(40, 40).border_radius(20)),
        column((
            kivact::ui::skeleton().style(style().size(250, 16)),
            kivact::ui::skeleton().style(style().size(200, 16)),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(16).align_items(Align::Center))
}
// end example

// example: card
fn card() -> impl IntoView {
    column((
        kivact::ui::skeleton().style(style().size(250, 125)),
        column((
            kivact::ui::skeleton().style(style().size(250, 16)),
            kivact::ui::skeleton().style(style().size(200, 16)),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(12))
}
// end example
