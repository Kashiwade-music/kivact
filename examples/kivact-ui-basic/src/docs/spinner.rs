//! Role: Catalog page of Spinner: default, in a button and with text.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "spinner",
        name: "Spinner",
        description: "処理の待ち状態を示す、回転する Lucide の loader-circle です。Role::Status で「Loading」と名付けられ、動きを減らす設定では止まります。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact_ui_basic as basic;",
        source: include_str!("spinner.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "basic::spinner() をそのまま置きます。色は文字色に従います。",
                default,
            ),
            Example::new(
                "in-button",
                "In a button",
                "ボタンの loading はスピナーを自動で表示します。",
                in_button,
            ),
            Example::new(
                "with-text",
                "With text",
                "スピナーと説明の文字を横に並べた例です。",
                with_text,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    basic::spinner()
}
// end example

// example: in-button
fn in_button() -> impl IntoView {
    row((
        button("Loading").busy(true),
        button("Please wait")
            .variant(ButtonVariant::Outline)
            .busy(true),
        button("Deleting").variant(ButtonVariant::Danger).busy(true),
    ))
    .style(style().gap(8))
}
// end example

// example: with-text
fn with_text() -> impl IntoView {
    row((
        basic::spinner(),
        text("Processing payment...").style(style().font_size(14)),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example
