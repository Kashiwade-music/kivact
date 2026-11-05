//! Role: Catalog page of Tooltip: default, sides, icon buttons and keyboard shortcut.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "tooltip",
        name: "Tooltip",
        description: "要素へのホバーやフォーカスで短い補足を表示します。標準の kivact::ui::tooltip に Lyra の見た目（反転した配色と、対象を指す矢印）を当てたもので、attach で対象の要素に取り付けます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact::view::Placement;\nuse kivact_ui_basic as basic;",
        source: include_str!("tooltip.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "tooltip に文字を渡し、attach で対象を渡します。ホバーまたはフォーカスで表示されます。",
                default,
            ),
            Example::new(
                "sides",
                "Sides",
                "placement で表示する側を選びます。",
                sides,
            ),
            Example::new(
                "icon-button",
                "Icon button",
                "名前だけを持つアイコンボタンは、ツールチップで意味を補います。",
                icon_button,
            ),
            Example::new(
                "shortcut",
                "With keyboard shortcut",
                "content で表示内容を差し替えて、kivact::ui::kbd のキーキャップを並べます。",
                shortcut,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    tooltip("Add to library").attach(button("Hover").variant(ButtonVariant::Outline))
}
// end example

// example: sides
fn sides() -> impl IntoView {
    let side = |label: &'static str, placement: Placement| {
        tooltip(format!("Add to {}", label.to_lowercase()))
            .placement(placement)
            .attach(button(label).variant(ButtonVariant::Outline))
    };
    row((
        side("Above", Placement::Above),
        side("Below", Placement::Below),
        side("Start", Placement::Start),
        side("End", Placement::End),
    ))
    .style(style().gap(8))
}
// end example

// example: icon-button
fn icon_button() -> impl IntoView {
    row((
        tooltip("Settings").attach(
            button("Settings")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("settings")),
        ),
        tooltip("Notifications").attach(
            button("Notifications")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("bell")),
        ),
        tooltip("Delete").attach(
            button("Delete")
                .variant(ButtonVariant::Outline)
                .hide_label(true)
                .icon(icon("trash-2")),
        ),
    ))
    .style(style().gap(8))
}
// end example

// example: shortcut
fn shortcut() -> impl IntoView {
    tooltip("Save")
        .content(|| {
            row((
                text("Save"),
                row((kivact::ui::kbd("Ctrl"), kivact::ui::kbd("S"))),
            ))
            .style(style().gap(8).align_items(kivact::style::Align::Center))
        })
        .attach(button("Save").variant(ButtonVariant::Outline))
}
// end example
