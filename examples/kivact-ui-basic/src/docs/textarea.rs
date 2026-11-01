//! Role: Catalog page of Textarea: basic, label, disabled and with a button.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "textarea",
        name: "Textarea",
        description: "複数行のテキストを入力する欄です。標準の kivact::ui::text_area に Lyra の見た目を当てたものです。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("textarea.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "ラベル、signal、placeholder を渡します。Enter で改行し、高さは 64 px から行数に合わせて伸びます。",
                basic,
            ),
            Example::new(
                "label",
                "With label",
                "basic::label の target に入力欄を渡して、ラベルから入力欄へフォーカスを移します。",
                with_label,
            ),
            Example::new(
                "disabled",
                "Disabled",
                "disabled の入力欄は薄く表示され、編集できません。",
                disabled,
            ),
            Example::new(
                "with-button",
                "With button",
                "入力欄の下に送信ボタンを置いた例です。文字数は signal から数えます。",
                with_button,
            ),
            Example::new(
                "field",
                "In a field",
                "basic::field でラベルと説明を付けます。",
                in_field,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    text_area("Message", signal(String::new()))
        .placeholder("Type your message here.")
        .style(style().width(360))
}
// end example

// example: label
fn with_label() -> impl IntoView {
    let field = node_ref();
    column((
        basic::label("Your message").target(field),
        text_area("Your message", signal(String::new()))
            .placeholder("Type your message here.")
            .node_ref(field)
            .style(style().width(360)),
        basic::muted("Your message will be copied to the support team."),
    ))
    .style(style().gap(8))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    text_area("Message", signal(String::new()))
        .placeholder("Type your message here.")
        .disabled(true)
        .style(style().width(360))
}
// end example

// example: with-button
fn with_button() -> impl IntoView {
    let message = signal(String::new());
    column((
        text_area("Your message", message)
            .placeholder("Type your message here.")
            .style(style().width(360)),
        basic::muted(move || format!("{} characters", message.with(|text| text.chars().count()))),
        button("Send message"),
    ))
    .style(style().gap(8))
}
// end example

// example: field
fn in_field() -> impl IntoView {
    let bio = signal(String::new());
    kivact::ui::field(
        "Bio",
        text_area("Bio", bio).placeholder("Tell us about yourself"),
    )
    .hint("You can mention people with @.")
    .style(style().width(320))
}
// end example
