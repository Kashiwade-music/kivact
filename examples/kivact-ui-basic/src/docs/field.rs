//! Role: Catalog page of Field: label, description, error, orientation and sets.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "field",
        name: "Field",
        description: "ラベル、説明、エラーを入力部品とまとめるフォームの区画です。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("field.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "field(label, control) に入力部品を渡し、hint を付けます。縦並びが既定です。",
                basic,
            ),
            Example::new(
                "error",
                "Error",
                "error に Some を渡すと、メッセージを Role::Alert で表示し、ラベルを destructive の色にします。",
                error,
            ),
            Example::new(
                "horizontal",
                "Horizontal",
                "標準 field に style の direction を指定して横に並べます。豊かな説明や応答する配置は、呼出元の row・column で合成します。",
                horizontal,
            ),
            Example::new(
                "set",
                "Field set",
                "field_set と field_group で複数の field をまとめます。field_separator は区切り線です。",
                set,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let username = signal(String::new());
    kivact::ui::field(
        "Username",
        text_input("Username", username).placeholder("shadcn"),
    )
    .hint("Choose a unique username for your account.")
    .style(style().width(320))
}
// end example

// example: error
fn error() -> impl IntoView {
    let email = signal(String::from("ada@"));
    let message = memo(move || {
        (!email.get().contains("@example.com")).then(|| "Use an @example.com address.".to_string())
    });
    kivact::ui::field(
        "Email",
        text_input("Email", email).invalid(move || message.get().is_some()),
    )
    .error(message)
    .style(style().width(320))
}
// end example

// example: horizontal
fn horizontal() -> impl IntoView {
    let on = signal(true);
    kivact::ui::field("Marketing emails", switch("Marketing emails", on))
        .hint("Receive news and offers.")
        .style(style().direction(kivact::style::Direction::Row))
        .style(style().width(420))
}
// end example

// example: set
fn set() -> impl IntoView {
    let (name, city) = (signal(String::new()), signal(String::new()));
    basic::field_set(basic::field_group((
        kivact::ui::field("Name", text_input("Name", name)),
        basic::field_separator().text("Or"),
        kivact::ui::field("City", text_input("City", city)).hint("Where you live."),
    )))
    .legend("Address")
    .description("Used to ship your order.")
    .style(style().width(320))
}
// end example
