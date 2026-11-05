//! Role: Catalog page of Badge: variants, with icon and as a pressable badge.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "badge",
        name: "Badge",
        description: "状態やラベルを小さく表示する部品です。on_press を渡すとリンクのように押せるバッジになります。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("badge.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "variants",
                "Variants",
                "BadgeVariant で見た目を選びます。",
                variants,
            ),
            Example::new(
                "with-icon",
                "With icon",
                "icon と trailing_icon で 12 px のアイコンを添えます。",
                with_icon,
            ),
            Example::new(
                "pressable",
                "Pressable",
                "on_press を渡すとフォーカスでき、Enter や Space でも押せます。",
                pressable,
            ),
        ],
    }
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
            .icon(icon("badge-check").style(style().size(12,12))),
        kivact::ui::badge("Open")
            .variant(basic::BadgeVariant::Outline)
            .trailing_icon(icon("arrow-up-right").style(style().size(12,12))),
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
