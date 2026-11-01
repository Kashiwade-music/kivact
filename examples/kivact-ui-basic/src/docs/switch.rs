//! Role: Catalog page of Switch: basic, description, disabled and settings.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "switch",
        name: "Switch",
        description: "オンとオフを切り替えるスイッチです。標準の kivact::ui::switch に Lyra の見た目を当てたものです。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("switch.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "ラベルと signal を渡します。クリックと Space で切り替わります。",
                basic,
            ),
            Example::new(
                "description",
                "With description",
                "ラベルの下に補足の文章を添える例です。",
                description,
            ),
            Example::new(
                "disabled",
                "Disabled",
                "disabled のスイッチは薄く表示され、操作できません。",
                disabled,
            ),
            Example::new(
                "invalid",
                "Invalid",
                "invalid を true にすると、枠が destructive の色になり、薄い ring が付きます。",
                invalid,
            ),
            Example::new(
                "settings",
                "Settings list",
                "複数のスイッチを並べた設定の例です。状態はそれぞれ別の signal が持ちます。",
                settings,
            ),
            Example::new(
                "field",
                "In a field",
                "basic::field の Horizontal で、ラベルと説明を左、スイッチを右に置きます。",
                in_field,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let airplane = signal(false);
    column((
        switch("Airplane mode", airplane),
        text(move || {
            if airplane.get() {
                "Radios are off".to_string()
            } else {
                "Radios are on".to_string()
            }
        })
        .style(style().font_size(12)),
    ))
    .style(style().gap(12))
}
// end example

// example: description
fn description() -> impl IntoView {
    let emails = signal(true);
    column((
        switch("Marketing emails", emails),
        basic::muted("Receive emails about new products and features.")
            .style(style().padding_left(44)),
    ))
    .style(style().gap(4))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    column((
        switch("Disabled off", signal(false)).disabled(true),
        switch("Disabled on", signal(true)).disabled(true),
    ))
    .style(style().gap(12))
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    let agreed = signal(false);
    switch("Share usage data", agreed).invalid(move || !agreed.get())
}
// end example

// example: settings
fn settings() -> impl IntoView {
    column((
        switch("Share across devices", signal(true)),
        switch("Notify me about replies", signal(false)),
        switch("Use compact density", signal(false)),
    ))
    .style(style().gap(12))
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let on = signal(false);
    kivact::ui::field("Share across devices", switch("Share across devices", on))
        .hint("Focus is shared across devices.")
        .style(style().direction(kivact::style::Direction::Row))
        .style(style().width(420))
}
// end example
