//! Role: Catalog page of Input: basic, label, disabled, invalid and with a button.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "input",
        name: "Input",
        description: "一行のテキストを入力する欄です。標準の kivact::ui::text_input に Lyra の見た目を当てたもので、値は呼び出し側の signal が持ちます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("input.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "ラベル（アクセシブルな名前）、signal、placeholder を渡します。入力した値は signal から読めます。",
                basic,
            ),
            Example::new(
                "label",
                "With label",
                "basic::label の target に入力欄を渡すと、ラベルを押して入力欄にフォーカスを移せます。",
                with_label,
            ),
            Example::new(
                "disabled",
                "Disabled",
                "disabled の入力欄は薄く表示され、編集できません。",
                disabled,
            ),
            Example::new(
                "invalid",
                "Invalid",
                "invalid を true にすると、枠が destructive の色になり、外側に薄い destructive の ring が付きます。支援技術にも値が不正であることが伝わります。",
                invalid,
            ),
            Example::new(
                "with-button",
                "With button",
                "入力欄とボタンを横に並べた例です。",
                with_button,
            ),
            Example::new(
                "field",
                "In a field",
                "basic::field で包むと、ラベル、説明、エラーを縦に並べます。",
                in_field,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let email = signal(String::new());
    column((
        text_input("Email", email)
            .placeholder("Email")
            .style(style().width(320)),
        basic::muted(move || format!("Value: {}", email.get())),
    ))
    .style(style().gap(8))
}
// end example

// example: label
fn with_label() -> impl IntoView {
    let field = node_ref();
    let name = signal(String::new());
    column((
        basic::label("Display name").target(field),
        text_input("Display name", name)
            .placeholder("Ada Lovelace")
            .with_node(|node| node.node_ref(field))
            .style(style().width(320)),
    ))
    .style(style().gap(8))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    text_input("Email", signal(String::new()))
        .placeholder("Email")
        .disabled(true)
        .style(style().width(320))
}
// end example

// example: invalid
fn invalid() -> impl IntoView {
    let username = signal(String::from("ada lovelace"));
    column((
        text_input("Username", username)
            .invalid(true)
            .style(style().width(320)),
        text("Usernames cannot contain spaces.").style(move || {
            style().font_size(12).color(
                kivact::style::use_theme()
                    .get()
                    .ext::<basic::Tokens>()
                    .destructive,
            )
        }),
    ))
    .style(style().gap(8))
}
// end example

// example: with-button
fn with_button() -> impl IntoView {
    let email = signal(String::new());
    row((
        text_input("Email", email)
            .placeholder("Email")
            .style(style().width(240)),
        button("Subscribe").variant(ButtonVariant::Outline),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let email = signal(String::new());
    kivact::ui::field(
        "Email",
        text_input("Email", email).placeholder("you@example.com"),
    )
    .hint("We will never share your email.")
    .style(style().width(320))
}
// end example
