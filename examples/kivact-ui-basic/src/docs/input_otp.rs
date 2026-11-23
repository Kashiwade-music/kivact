//! Catalog page of Input OTP: groups, patterns and completion.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("input-otp", "Input OTP", "Components", include_str!("input_otp.rs"))
        .description("Enters a short code, such as a verification code, with one box per character.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "basic",
            "Basic",
            "Pass the number of digits. One hidden input receives the typing; the boxes show the characters and the position.",
            basic,
        ))
        .example(Example::new(
            "groups",
            "Groups",
            "groups divides the boxes, and a - is put between the groups.",
            groups,
        ))
        .example(Example::new(
            "complete",
            "On complete",
            "pattern chooses the characters accepted, and on_complete receives the value when every digit is filled.",
            complete,
        ))
        .example(Example::new(
            "disabled",
            "Disabled",
            "A disabled input is drawn faint and cannot be typed in.",
            disabled,
        ))
}

// example: basic
fn basic() -> impl IntoView {
    let code = signal(String::new());
    basic::input_otp(code, 6usize).name("Verification code")
}
// end example

// example: groups
fn groups() -> impl IntoView {
    let code = signal(String::new());
    basic::input_otp(code, 6usize)
        .name("Verification code")
        .groups([3, 3])
}
// end example

// example: complete
fn complete() -> impl IntoView {
    let code = signal(String::new());
    let done = signal(String::new());
    column((
        basic::input_otp(code, 4usize)
            .name("Access code")
            .pattern(basic::OtpPattern::Alphanumeric)
            .on_complete(move |c: String| done.set(c)),
        basic::muted(move || format!("Completed with: {}", done.get())),
    ))
    .style(style().gap(8))
}
// end example

// example: disabled
fn disabled() -> impl IntoView {
    basic::input_otp(signal(String::from("123")), 6usize)
        .name("Code")
        .disabled(true)
}
// end example
