//! Role: Code block tests: copy button, highlight colors, selection, copy and long code.

use crate::support::assert_clean;
use kivact::prelude::*;
use kivact::style::Align;
use kivact_ui_basic as basic;
use std::time::Duration;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::ColorName::Background)
            .padding(16)
            .gap(12)
            .grow(1)
            .align_items(Align::Start),
    )
}

/// The 23 px line of the default size.
const LINE: f64 = 23.0;

#[test]
fn code_block_reports_copy_requests_and_shows_copied_for_two_seconds() {
    let (screen, copies) = kivact::test::render_with_handle(crate::support::options(), || {
        let copies = signal(0usize);
        (
            copies,
            page(
                kivact::ui::code_block("let a = 1;\n\nlet b = 2;".to_string())
                    .title("main.rs")
                    .line_numbers(true)
                    .on_copy(move || copies.update(|n| *n += 1))
                    .footer(text("Rust"))
                    .test_id("code")
                    .style(style().width(400)),
            ),
        )
    });
    screen.get_by_text("main.rs");
    screen.get_by_text("Rust");
    // One number per line, the blank second line as high as the others.
    let numbers = screen.get_by_text("1\n2\n3").rect().expect("laid out");
    let code = screen.get_by_role(Role::TextBox, "main.rs");
    let rect = code.rect().expect("laid out");
    assert_eq!(rect.height, 3.0 * LINE);
    assert_eq!(numbers.y, rect.y);
    assert_eq!(numbers.height, rect.height);
    assert_eq!(code.value().as_deref(), Some("let a = 1;\n\nlet b = 2;"));
    screen.get_by_role(Role::Button, "Copy").click();
    assert_eq!(copies.get_untracked(), 1);
    screen.get_by_role(Role::Button, "Copied");
    screen.get_by_role(Role::Status, "Copied");
    screen.advance(Duration::from_millis(1900));
    screen.get_by_role(Role::Button, "Copied");
    screen.advance(Duration::from_millis(200));
    screen
        .get_by_role(Role::Button, "Copy")
        .press_key(Key::Enter);
    assert_eq!(copies.get_untracked(), 2);
    assert_clean(&screen);
}
