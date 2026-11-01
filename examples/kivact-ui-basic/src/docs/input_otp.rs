//! Role: Catalog page of Input OTP: groups, patterns and completion.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "input-otp",
        name: "Input OTP",
        description: "認証コードなどの短い文字列を、一文字ずつの枠で入力する部品です。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("input_otp.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "basic",
                "Basic",
                "桁数を渡します。入力は一つの隠れた入力欄が受け取り、枠は文字と入力位置を表示します。",
                basic,
            ),
            Example::new(
                "groups",
                "Groups",
                "groups で枠を区切り、区切りの間に - を置きます。",
                groups,
            ),
            Example::new(
                "complete",
                "On complete",
                "pattern で受け付ける文字を選び、on_complete で全桁そろったときに値を受け取ります。",
                complete,
            ),
            Example::new(
                "disabled",
                "Disabled",
                "disabled の間は薄く表示され、入力できません。",
                disabled,
            ),
        ],
    }
}

// example: basic
fn basic() -> impl IntoView {
    let code = signal(String::new());
    basic::input_otp("Verification code", code, 6)
}
// end example

// example: groups
fn groups() -> impl IntoView {
    let code = signal(String::new());
    basic::input_otp("Verification code", code, 6).groups(&[3, 3])
}
// end example

// example: complete
fn complete() -> impl IntoView {
    let code = signal(String::new());
    let done = signal(String::new());
    column((
        basic::input_otp("Access code", code, 4)
            .pattern(basic::OtpPattern::Alphanumeric)
            .on_complete(move |c: String| done.set(c)),
        basic::muted(move || format!("Completed with: {}", done.get())),
    ))
    .style(style().gap(8))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    basic::input_otp("Code", signal(String::from("123")), 6).disabled(true)
}
// end example
