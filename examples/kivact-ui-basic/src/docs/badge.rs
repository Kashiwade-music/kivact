//! Catalog page of Badge: variants, with icon and as a pressable badge.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("badge", "Badge", "Components", include_str!("badge.rs"))
        .description("A small badge that shows a status or a label. Pass on_press to make it a badge that is pressed like a link.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "variants",
            "Variants",
            "Choose the look with BadgeVariant.",
            variants,
        ))
        .example(Example::new(
            "with-icon",
            "With icon",
            "icon and trailing_icon add a 12 px icon.",
            with_icon,
        ))
        .example(Example::new(
            "pressable",
            "Pressable",
            "With on_press the badge takes focus, and Enter or Space presses it too.",
            pressable,
        ))
}

// example: variants
fn variants() -> impl IntoView {
    row((
        kivact::ui::badge("Badge"),
        kivact::ui::badge("Secondary").variant(basic::BadgeVariant::Secondary),
        kivact::ui::badge("Destructive").variant(basic::BadgeVariant::Destructive),
        kivact::ui::badge("Outline").variant(basic::BadgeVariant::Outline),
        kivact::ui::badge("Ghost").variant(basic::BadgeVariant::Ghost),
        kivact::ui::badge("Link").variant(basic::BadgeVariant::Link),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: with-icon
fn with_icon() -> impl IntoView {
    row((
        kivact::ui::badge("Verified")
            .variant(basic::BadgeVariant::Secondary)
            .icon(icon("badge-check").style(style().size(12, 12))),
        kivact::ui::badge("Open")
            .variant(basic::BadgeVariant::Outline)
            .trailing_icon(icon("arrow-up-right").style(style().size(12, 12))),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: pressable
fn pressable() -> impl IntoView {
    let count = signal(0);
    row((
        kivact::ui::badge(move || format!("Clicked {}", count.get()))
            .variant(basic::BadgeVariant::Outline)
            .on_press(move || count.update(|n| *n += 1)),
        kivact::ui::badge("Reset")
            .variant(basic::BadgeVariant::Ghost)
            .on_press(move || count.set(0)),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example
