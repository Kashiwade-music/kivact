//! Role: Catalog page of Kbd: key caps, groups and use inside a button.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "kbd",
        name: "Kbd",
        description: "キーボードのキーやショートカットを表示する小さなキーキャップです。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact_ui_basic as basic;",
        source: include_str!("kbd.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "kivact::ui::kbd に文字を渡すと、一つのキーキャップになります。",
                default,
            ),
            Example::new(
                "group",
                "Group",
                "row で複数のキーを組み合わせて、ショートカットを表します。",
                group,
            ),
            Example::new(
                "in-button",
                "In a button",
                "ボタンの子として並べると、操作に対応するショートカットを示せます。",
                in_button,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    row((
        kivact::ui::kbd("Ctrl"),
        kivact::ui::kbd("Shift"),
        kivact::ui::kbd("Esc"),
        kivact::ui::kbd("⌘"),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: group
fn group() -> impl IntoView {
    column((
        row((kivact::ui::kbd("Ctrl"), kivact::ui::kbd("B"))).style(style().align_items(Align::Center).gap(4)),
        row((kivact::ui::kbd("Ctrl"), kivact::ui::kbd("Shift"), kivact::ui::kbd("P"))).style(style().align_items(Align::Center).gap(4)),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: in-button
fn in_button() -> impl IntoView {
    row((
        button("Accept")
            .variant(ButtonVariant::Outline)
            .trailing(kivact::ui::kbd("⏎")),
        button("Cancel")
            .variant(ButtonVariant::Outline)
            .trailing(kivact::ui::kbd("Esc")),
    ))
    .style(style().gap(8))
}
// end example
