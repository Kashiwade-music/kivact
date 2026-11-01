//! Role: Code block tests: copy button, highlight colors, selection, copy and long code.

use super::{assert_clean, distance};
use kivact::draw::Point;
use kivact::prelude::*;
use kivact::style::{Align, Color, ThemeMode};
use kivact::test::Screen;
use kivact::view::Modifiers;
use kivact_ui_basic as basic;
use std::time::Duration;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .padding(16)
            .gap(12)
            .grow(1)
            .align_items(Align::Start),
    )
}

/// The 23 px line of the default size.
const LINE: f32 = 23.0;

fn ctrl() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Modifiers::default()
    }
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

/// The color nearest `color` drawn on line `line` of the code text box
/// `name`, and how far it is.
fn nearest_on_line(screen: &Screen, name: &str, line: usize, color: u32) -> (u32, u32) {
    let rect = screen
        .get_by_role(Role::TextBox, name)
        .rect()
        .expect("laid out");
    let scale = screen.scale();
    let shot = screen.screenshot();
    let top = ((rect.y + line as f32 * LINE) * scale) as u32;
    let (left, right) = (
        (rect.x * scale) as u32,
        ((rect.x + rect.width) * scale) as u32,
    );
    let mut best = (0, u32::MAX);
    for y in top..top + (LINE * scale) as u32 {
        for x in left..right.min(shot.width) {
            if let Some(p) = shot.pixel(x, y) {
                let d = distance(p, color);
                if d < best.1 {
                    best = (p, d);
                }
            }
        }
    }
    best
}

/// `color` as `0xRRGGBB`.
fn rgb(color: Color) -> u32 {
    let [r, g, b, _] = color.components();
    let channel = |v: f32| (v * 255.0).round() as u32;
    channel(r) << 16 | channel(g) << 8 | channel(b)
}

#[track_caller]
fn assert_drawn(screen: &Screen, name: &str, line: usize, color: u32, what: &str) {
    let (p, d) = nearest_on_line(screen, name, line, color);
    assert!(
        d <= 24,
        "{what} on line {line}: nearest {p:06x} to {color:06x}"
    );
}

#[test]
fn code_block_reports_copy_requests_and_shows_copied_for_two_seconds() {
    let (screen, copies) = kivact::test::render_with_handle(crate::options(), || {
        let copies = signal(0usize);
        (
            copies,
            page(
                basic::code_block(basic::code_lines("let a = 1;\n\nlet b = 2;\n"))
                    .title("main.rs")
                    .line_numbers(true)
                    .on_copy(move || copies.update(|n| *n += 1))
                    .footer(move || text("Rust"))
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

#[test]
fn the_code_is_a_read_only_text_box_after_the_copy_button_in_the_tab_order() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page((
                basic::code_block(basic::rust_lines("let a = 1;"))
                    .title("main.rs")
                    .on_copy(|| {}),
                basic::code_block(basic::code_lines("cargo run")),
                kivact::ui::button("After"),
            )),
        )
    });
    let code = screen.get_by_role(Role::TextBox, "main.rs");
    assert!(code.read_only());
    // Without a title the box is called by what it holds.
    let untitled = screen.get_by_role(Role::TextBox, "Code");
    assert_eq!(untitled.value().as_deref(), Some("cargo run"));
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "Copy").is_focused());
    screen.press_tab(false);
    assert!(code.is_focused());
    // Typing, deleting and pasting change nothing; Tab leaves the code.
    code.type_text("x");
    code.press_key(Key::Backspace);
    screen.set_clipboard("pasted");
    code.press_key_with(Key::Char('v'), ctrl());
    assert_eq!(code.value().as_deref(), Some("let a = 1;"));
    screen.press_tab(false);
    assert!(untitled.is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert_clean(&screen);
}

#[test]
fn a_part_of_the_code_is_selected_by_pointer_and_keyboard_and_copied_with_ctrl_c() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            page(
                basic::code_block(basic::rust_lines("let a = 1;\nlet b = 2;"))
                    .title("main.rs")
                    .style(style().width(400)),
            ),
        )
    });
    let code = screen.get_by_role(Role::TextBox, "main.rs");
    // A drag from the start of the first line to the start of the second
    // selects the first line and its break.
    code.drag(
        Point {
            x: 0.5,
            y: LINE / 2.0,
        },
        Point {
            x: 0.5,
            y: LINE * 1.5,
        },
    );
    assert!(code.is_focused(), "the press focuses the code");
    assert_eq!(code.text_selection(), Some((0, 11)));
    code.press_key_with(Key::Char('c'), ctrl());
    assert_eq!(screen.clipboard().as_deref(), Some("let a = 1;\n"));
    // On the second line, Home and Shift+End select it without its break.
    code.press_key(Key::Home);
    code.press_key_with(Key::End, shift());
    assert_eq!(code.text_selection(), Some((11, 21)));
    code.press_key_with(Key::Char('c'), ctrl());
    assert_eq!(screen.clipboard().as_deref(), Some("let b = 2;"));
    // Shift+ArrowLeft and Shift+ArrowUp select back from the end of the
    // code into the line above.
    code.press_key_with(Key::End, ctrl());
    code.press_key_with(Key::ArrowLeft, shift());
    code.press_key_with(Key::ArrowLeft, shift());
    code.press_key_with(Key::ArrowUp, shift());
    assert_eq!(code.text_selection(), Some((21, 8)));
    code.press_key_with(Key::Char('c'), ctrl());
    assert_eq!(
        screen.clipboard().as_deref(),
        Some(
            "1;
let b = 2;"
        )
    );
    // Ctrl+A selects the whole code.
    code.press_key_with(Key::Char('a'), ctrl());
    code.press_key_with(Key::Char('c'), ctrl());
    assert_eq!(
        screen.clipboard().as_deref(),
        Some("let a = 1;\nlet b = 2;")
    );
    assert_clean(&screen);
}

#[test]
fn rust_code_is_drawn_in_the_themes_code_colors_in_both_schemes() {
    let source = "/* A block\n   comment */ fn main() {\n    let s: &'static str = \"hi\";\n    println!(\"{}\", 42);\n    Vec::<u8>::new();\n}";
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let colors = basic::CodeColors::new(mode);
        let (screen, ()) = kivact::test::render_with_handle(
            kivact::test::RenderOptions {
                theme: basic::theme(mode),
                ..crate::options()
            },
            move || {
                (
                    (),
                    page(
                        basic::code_block(basic::rust_lines(source))
                            .title("main.rs")
                            .style(style().width(500)),
                    ),
                )
            },
        );
        let at = |line, color: Color, what: &str| {
            assert_drawn(
                &screen,
                "main.rs",
                line,
                rgb(color),
                &format!("{what} in {mode:?}"),
            );
        };
        // The comment opened on the first line colors the second too.
        at(0, colors.comment, "the comment");
        at(1, colors.comment, "the comment's second line");
        at(1, colors.keyword, "fn after it");
        at(1, colors.function, "main");
        at(2, colors.keyword, "let");
        at(2, colors.lifetime, "'static");
        at(2, colors.string, "the string");
        at(3, colors.macro_call, "println!");
        at(3, colors.number, "42");
        at(4, colors.type_name, "Vec");
        at(2, colors.type_name, "str");
        // The plain closing brace is in the foreground, no tone color.
        let (_, far) = nearest_on_line(&screen, "main.rs", 5, rgb(colors.keyword));
        assert!(far > 24, "a plain line has no keyword color in {mode:?}");
        assert_clean(&screen);
    }
}

#[test]
fn a_long_listing_scrolls_and_draws_the_highlighted_lines_it_shows() {
    let source = (1..=2000)
        .map(|n| format!("let value_{n} = {n}; // line {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), move || {
        (
            (),
            page(
                basic::code_block(basic::rust_lines(&source))
                    .title("long.rs")
                    .line_numbers(true)
                    .max_height(200.0)
                    .test_id("code")
                    .style(style().width(500)),
            ),
        )
    });
    let colors = basic::CodeColors::new(ThemeMode::Light);
    let keyword = rgb(colors.keyword);
    let code = screen.get_by_role(Role::TextBox, "long.rs");
    assert_eq!(code.rect().expect("laid out").height, 2000.0 * LINE);
    let block = screen.get_by_test_id("code").rect().expect("laid out");
    assert!(
        block.height < 260.0,
        "the block keeps its max height: {block:?}"
    );
    assert_drawn(&screen, "long.rs", 0, keyword, "let");
    // Far down the listing, the lines shown are drawn and highlighted.
    screen.get_by_test_id("code").scroll(0.0, 1000.0 * LINE);
    let top = code.rect().expect("laid out").y;
    let first = ((block.y - top) / LINE).ceil() as usize;
    assert!(first > 900, "scrolled down to line {first}");
    assert_drawn(&screen, "long.rs", first + 1, keyword, "let far down");
    assert_clean(&screen);
}
