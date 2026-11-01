//! Role: Tests for text editing and grapheme cluster handling.
//!
//! Text editing end to end on real windows: the caret and deletion step
//! over grapheme clusters, Shift selections and their collapse, the
//! clipboard chords and the normalisation of pasted line breaks, IME
//! composition (commit, cancel, keys swallowed while composing, commit on
//! blur, `on_composition_change` on each change of the composing state),
//! and which keys reach the handlers around a field.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::interaction::use_text_edit;
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, TestFonts, WindowMode, render, render_with};
use kivact::view::KeyEvent;
use kivact::view::Modifiers;

/// A family emoji: one grapheme of five scalars.
const FAMILY: &str = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

fn ctrl() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Modifiers::default()
    }
}

type Log = Rc<RefCell<Vec<String>>>;

#[test]
fn full_multiline_enter_reaches_the_parent_key_handler() {
    let log: Log = Rc::default();
    let keys = log.clone();
    let screen = render(move || {
        node()
            .on_key(move |event: KeyEvent| {
                keys.borrow_mut().push(format!("key {:?}", event.key));
            })
            .child(
                kivact::control::node().name("Limited").part(
                    kivact::interaction::use_text_edit(signal(String::from("x")))
                        .multiline(true)
                        .max_length(1usize)
                        .control(),
                ),
            )
    });
    screen
        .get_by_role(Role::TextBox, "Limited")
        .press_key(Key::End);
    log.borrow_mut().clear();
    screen
        .get_by_role(Role::TextBox, "Limited")
        .press_key(Key::Enter);
    assert_eq!(&*log.borrow(), &["key Enter"]);
}

#[test]
fn empty_backspace_reaches_the_parent_key_handler() {
    let (screen, _, log) = field_screen("", false);
    screen
        .get_by_role(Role::TextBox, "Name")
        .press_key(Key::Backspace);
    assert_eq!(&*log.borrow(), &["key Backspace"]);
}

/// A field bound to `value` inside a box that logs the keys reaching it;
/// the binding's writes and `on_submit` log too.
fn field_screen(initial: &str, area: bool) -> (Screen, Signal<String>, Log) {
    let log: Log = Rc::default();
    let out = log.clone();
    let initial = initial.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    let screen = render(move || {
        let value = signal(initial);
        tx.send(value).expect("the test waits");
        let (keys, changes, submits) = (out.clone(), out.clone(), out.clone());
        let edited = kivact::binding(value, move |v: String| {
            changes.borrow_mut().push(format!("change {v}"));
            value.set(v);
        });
        let field = use_text_edit(edited).multiline(area);
        column((
            node()
                .on_key(move |e: KeyEvent| {
                    keys.borrow_mut().push(format!("key {:?}", e.key));
                })
                .child(
                    node().name(if area { "Notes" } else { "Name" }).part(
                        field
                            .on_submit(move || submits.borrow_mut().push("submit".into()))
                            .control(),
                    ),
                ),
            node()
                .role(Role::Button)
                .name("Elsewhere")
                .focusable(true)
                .style(style().size(40, 20)),
        ))
    });
    let value = rx.recv().expect("mounted");
    (screen, value, log)
}

#[test]
fn initial_bound_input_keeps_its_caret_at_the_end_when_focused_by_tab() {
    for area in [false, true] {
        let (screen, value, _) = field_screen("abc", area);
        screen.press_key(Key::Tab);
        screen.type_text("X");
        assert_eq!(value.get_untracked(), "abcX");
        let name = if area { "Notes" } else { "Name" };
        assert_eq!(value_of(&screen, Role::TextBox, name), "abcX");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

fn value_of(screen: &Screen, role: Role, name: &str) -> String {
    screen.get_by_role(role, name).value().unwrap_or_default()
}

#[test]
fn caret_and_deletion_step_over_graphemes_and_selections_replace() {
    let (screen, value, log) = field_screen(&format!("a{FAMILY}e\u{301}🇯🇵"), false);
    let f = screen.get_by_role(Role::TextBox, "Name");
    f.press_key(Key::End);
    f.press_key(Key::Backspace);
    assert_eq!(
        value.get_untracked(),
        format!("a{FAMILY}e\u{301}"),
        "the whole flag"
    );
    f.press_key(Key::ArrowLeft);
    f.type_text("X");
    assert_eq!(
        value.get_untracked(),
        format!("a{FAMILY}Xe\u{301}"),
        "over e + accent"
    );
    f.press_key(Key::Home);
    f.press_key(Key::ArrowRight);
    f.press_key(Key::Delete);
    assert_eq!(value.get_untracked(), "aXe\u{301}", "the whole family");

    value.set("abcd".into());
    screen.run_until_idle();
    log.borrow_mut().clear();
    f.press_key(Key::End);
    f.press_key_with(Key::ArrowLeft, shift());
    f.press_key_with(Key::ArrowLeft, shift());
    assert!(
        log.borrow().is_empty(),
        "caret keys change nothing and stay in the field: {:?}",
        log.borrow()
    );
    f.type_text("Z");
    assert_eq!(
        value.get_untracked(),
        "abZ",
        "typing replaces the selection"
    );
    // A plain arrow collapses a selection to its edge.
    f.press_key_with(Key::ArrowLeft, shift());
    f.press_key_with(Key::ArrowLeft, shift());
    f.press_key(Key::ArrowLeft);
    f.type_text("X");
    assert_eq!(value.get_untracked(), "aXbZ");
    f.press_key_with(Key::Char('a'), ctrl());
    f.type_text("x");
    assert_eq!(value.get_untracked(), "x", "Ctrl+A selects everything");
    // Chords the field does not edit with go on to the handlers around it.
    log.borrow_mut().clear();
    f.press_key_with(Key::Char('s'), ctrl());
    f.press_key(Key::Enter);
    assert_eq!(
        *log.borrow(),
        ["key Char('s')", "submit"],
        "{:?}",
        log.borrow()
    );
    assert_eq!(value_of(&screen, Role::TextBox, "Name"), "x");
}

#[test]
fn clipboard_chords_and_pasted_line_breaks() {
    let (screen, value, _log) = field_screen("abc", false);
    let f = screen.get_by_role(Role::TextBox, "Name");
    screen.set_clipboard("kept");
    f.press_key_with(Key::Char('c'), ctrl());
    assert_eq!(
        screen.clipboard().as_deref(),
        Some("kept"),
        "nothing selected, nothing copied"
    );
    f.press_key_with(Key::Char('a'), ctrl());
    f.press_key_with(Key::Char('c'), ctrl());
    assert_eq!(screen.clipboard().as_deref(), Some("abc"));
    f.press_key_with(Key::Char('x'), ctrl());
    assert_eq!(
        (
            screen.clipboard().as_deref(),
            value.get_untracked().as_str()
        ),
        (Some("abc"), "")
    );
    screen.set_clipboard("a\r\nb\rc\t d");
    f.press_key_with(Key::Char('v'), ctrl());
    assert_eq!(
        value.get_untracked(),
        "a  b c\t d",
        "one line: breaks become spaces"
    );

    let (screen, value, _log) = field_screen("", true);
    let area = screen.get_by_role(Role::TextBox, "Notes");
    screen.set_clipboard("a\r\nb\rc\t d");
    area.press_key_with(Key::Char('v'), ctrl());
    assert_eq!(value.get_untracked(), "a\nb\rc\t d", "CRLF becomes LF");
    area.press_key(Key::Enter);
    area.type_text("e");
    assert_eq!(
        value.get_untracked(),
        "a\nb\rc\t d\ne",
        "Enter breaks the line"
    );
}

#[test]
fn ime_composition_commits_cancels_swallows_keys_and_ends_on_blur() {
    let (screen, value, log) = field_screen("a🦀日", false);
    let f = screen.get_by_role(Role::TextBox, "Name");
    f.press_key(Key::End);
    f.press_key(Key::Backspace);
    assert_eq!(value.get_untracked(), "a🦀");
    f.compose("日本");
    assert_eq!(
        value.get_untracked(),
        "a🦀",
        "a composition is not the value yet"
    );
    f.commit("日本語");
    assert_eq!(value.get_untracked(), "a🦀日本語");
    f.type_text("X");
    assert_eq!(
        value.get_untracked(),
        "a🦀日本語X",
        "the caret follows the commit"
    );

    // Cancelling (an empty preedit) removes the composition.
    f.compose("仮");
    f.compose("");
    assert_eq!(value_of(&screen, Role::TextBox, "Name"), "a🦀日本語X");
    assert_eq!(value.get_untracked(), "a🦀日本語X");

    // Keys go to the IME while composing: nothing reaches the handlers.
    log.borrow_mut().clear();
    f.compose("に");
    for key in [Key::Enter, Key::Escape, Key::ArrowLeft, Key::Char('a')] {
        screen.press_key(key);
    }
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());
    assert_eq!(value.get_untracked(), "a🦀日本語X");

    // Moving the focus away commits the composition.
    f.compose("にほ");
    screen.get_by_role(Role::Button, "Elsewhere").click();
    assert_eq!(value.get_untracked(), "a🦀日本語Xにほ");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// IME reconversion (Windows answers the IME's `WM_IME_REQUEST` from what
/// the host gives it): the host tells the IME the focused field's text,
/// selection and composition in step with every edit, and where the
/// composed characters are; a range the IME takes for reconversion becomes
/// the selection, which the reconversion's composition then replaces.
#[test]
fn ime_reconversion_replaces_the_range_the_ime_takes() {
    let (screen, value, _log) = field_screen("", false);
    assert_eq!(screen.ime_text(), None, "no field has the focus");
    let f = screen.get_by_role(Role::TextBox, "Name");
    // A crab first: its two UTF-16 units are one char.
    f.commit("🦀にほんごをべんきょう");
    let ime = screen.ime_text().expect("the field has the focus");
    assert_eq!(ime.text, "🦀にほんごをべんきょう");
    assert_eq!((ime.selection, ime.composition), (11..11, None));
    assert_eq!(ime.char_rects.len(), 1, "the caret's box");
    let field = f.rect().expect("laid out");
    assert!(
        ime.area.x >= field.x && ime.area.x + ime.area.width <= field.x + field.width,
        "the text area lies in the field: {:?} {field:?}",
        ime.area
    );

    // The IME takes the phrase at the caret: "にほんごを".
    f.press_key(Key::Home);
    f.press_key(Key::ArrowRight);
    f.press_key(Key::ArrowRight);
    f.reconvert(1..6);
    assert_eq!(screen.ime_text().expect("focused").selection, 1..6);
    f.compose("日本語を");
    let ime = screen.ime_text().expect("focused");
    assert_eq!(ime.text, "🦀日本語をべんきょう");
    assert_eq!(ime.composition, Some(1..5));
    assert_eq!(ime.char_rects.len(), 5, "four composed chars and the caret");
    assert!(
        ime.char_rects
            .windows(2)
            .all(|pair| pair[0].x < pair[1].x && pair[0].y == pair[1].y),
        "boxes run left to right on one line: {:?}",
        ime.char_rects
    );
    // A reconversion range while composing is ignored.
    f.reconvert(0..1);
    assert_eq!(screen.ime_text().expect("focused").composition, Some(1..5));
    f.commit("日本語を");
    assert_eq!(value.get_untracked(), "🦀日本語をべんきょう");
    assert_eq!(screen.ime_text().expect("focused").selection, 5..5);

    // The whole value selected, reconverted and replaced.
    f.press_key_with(Key::Char('a'), ctrl());
    assert_eq!(screen.ime_text().expect("focused").selection, 0..10);
    f.reconvert(0..10);
    f.compose("蟹日本語を勉強");
    f.commit("蟹日本語を勉強");
    assert_eq!(value.get_untracked(), "蟹日本語を勉強");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// In a multiline field the IME's character boxes follow the caret onto
/// the line it composes on, and a reconversion range spanning a line
/// break replaces exactly that range.
#[test]
fn ime_boxes_follow_the_line_of_a_multiline_field() {
    let (screen, value, _log) = field_screen("first line\nsecond", true);
    let area = screen.get_by_role(Role::TextBox, "Notes");
    area.press_key(Key::Home);
    let top = screen.ime_text().expect("focused").char_rects[0];
    area.press_key(Key::End);
    let first = screen.ime_text().expect("focused").char_rects[0];
    assert!(
        first.y > top.y,
        "the end is on the second line: {top:?} {first:?}"
    );
    area.compose("xyz");
    let ime = screen.ime_text().expect("focused");
    assert_eq!(ime.composition, Some(17..20));
    assert_eq!(
        ime.char_rects.len(),
        4,
        "three composed chars and the caret"
    );
    assert!(
        ime.char_rects.iter().all(|r| r.y == first.y),
        "on the caret's line: {first:?} {:?}",
        ime.char_rects
    );
    area.commit("xyz");
    assert_eq!(value.get_untracked(), "first line\nsecondxyz");
    // "line\nsecond" spans the break.
    area.reconvert(6..17);
    area.commit("LINE");
    assert_eq!(value.get_untracked(), "first LINExyz");
}

/// A text field or area of the given height without padding, alone in the
/// window, focused.
fn tall_input(area: bool, height: u32) -> Screen {
    let screen = render(move || {
        node()
            .name("Tall")
            .part(
                use_text_edit(signal(String::from("abcdefghij")))
                    .multiline(area)
                    .control(),
            )
            .style(style().width(200).height(height).padding(0))
    });
    screen.get_by_role(Role::TextBox, "Tall").focus();
    screen
}

/// A single-line field puts its line in the middle of its content box
/// (the caret, the IME's boxes and the text area are one line high around
/// the same centre), and a content box lower than the line overflows it
/// evenly. A text area keeps its text at the top.
#[test]
fn single_line_input_centers_its_line_in_the_content_box() {
    for height in [48, 8] {
        let screen = tall_input(false, height);
        let ime = screen.ime_text().expect("focused");
        let caret = *ime.char_rects.last().expect("the caret");
        let middle = ime.area.y + ime.area.height / 2.0;
        assert!(
            (caret.y + caret.height / 2.0 - middle).abs() < 0.5,
            "the line is centered in {:?}: {caret:?}",
            ime.area
        );
    }
    let screen = tall_input(true, 100);
    let ime = screen.ime_text().expect("focused");
    let caret = *ime.char_rects.last().expect("the caret");
    assert!(
        (caret.y - ime.area.y).abs() < 0.5,
        "a text area starts at the top of {:?}: {caret:?}",
        ime.area
    );
}

/// The pointer lands on the characters where they are painted, centered or
/// not: a click on a character's right half places the caret after it.
#[test]
fn clicking_a_centered_line_places_the_caret_under_the_pointer() {
    let screen = tall_input(false, 48);
    let field = screen.get_by_role(Role::TextBox, "Tall");
    field.press_key(Key::Home);
    let mut stops = Vec::new();
    for _ in 0..=10 {
        let ime = screen.ime_text().expect("focused");
        stops.push(*ime.char_rects.last().expect("the caret"));
        field.press_key(Key::ArrowRight);
    }
    for (at, pair) in stops.windows(2).enumerate() {
        let (from, to) = (pair[0], pair[1]);
        // A hair before the boundary after `at + 1` characters, level with
        // the middle of the painted line.
        let x = to.x - (to.x - from.x) * 0.25;
        screen.click_at(x, from.y + from.height / 2.0);
        let ime = screen.ime_text().expect("focused");
        assert_eq!(ime.selection, at + 1..at + 1, "x {x} over character {at}");
    }
    // The box above the line still belongs to the field: the nearest
    // boundary on that side of the line.
    let first = stops[0];
    screen.click_at(
        first.x + 0.5,
        screen
            .get_by_role(Role::TextBox, "Tall")
            .rect()
            .expect("shown")
            .y
            + 2.0,
    );
    assert_eq!(screen.ime_text().expect("focused").selection, 0..0);
}

#[test]
fn hidden_and_widened_inputs_paint_their_retained_value() {
    let gpu = std::env::var("KIVACT_TEST_GPU").is_ok_and(|value| value == "1");
    let screen = render_with(
        RenderOptions {
            width: 480.,
            height: 240.,
            window: WindowMode::Real,
            fonts: TestFonts::System,
            gpu,
            ..Default::default()
        },
        move || {
            let hidden = signal(true);
            let width = signal(320.0f32);
            column((
                row((
                    node()
                        .role(Role::Button)
                        .name("Reveal")
                        .focusable(true)
                        .style(style().size(80, 24))
                        .child(text("Reveal"))
                        .on_press(move || hidden.set(false)),
                    node()
                        .role(Role::Button)
                        .name("Narrow")
                        .focusable(true)
                        .style(style().size(80, 24))
                        .child(text("Narrow"))
                        .on_press(move || width.set(40.)),
                    node()
                        .role(Role::Button)
                        .name("Widen")
                        .focusable(true)
                        .style(style().size(80, 24))
                        .child(text("Widen"))
                        .on_press(move || width.set(320.)),
                )),
                kivact::control::node()
                    .name("Retained value")
                    .part(
                        kivact::interaction::use_text_edit(signal(String::from(
                            "Saved output name",
                        )))
                        .control(),
                    )
                    .hidden(hidden)
                    .style(move || {
                        style()
                            .width(width.get())
                            .height(30)
                            .background(rgb(0x202020))
                            .color(rgb(0xffffff))
                    }),
            ))
            .style(style().padding(10).gap(8))
        },
    );
    screen.get_by_role(Role::Button, "Reveal").click();
    let field = screen.get_by_role(Role::TextBox, "Retained value");
    assert_eq!(field.value(), Some("Saved output name".into()));
    let full = field.screenshot();
    assert!(
        full.pixels
            .iter()
            .filter(|&&p| ((p >> 16) & 255) > 100 && ((p >> 8) & 255) > 100 && (p & 255) > 100)
            .count()
            > 15
    );
    screen.get_by_role(Role::Button, "Narrow").click();
    field.press_key(Key::End);
    screen.get_by_role(Role::Button, "Widen").click();
    assert_eq!(
        field.screenshot(),
        full,
        "Widening must restore the full value after horizontal caret scrolling"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    assert!(screen.request_close());
}

/// `on_composition_change` reports `true` when a composition starts and
/// `false` when it ends (commit, cancel, the blur that commits it), once
/// per change: further preedits and plain typing report nothing. At the
/// end the binding already holds the committed text. A text area reports
/// the same.
#[test]
fn on_composition_change_reports_each_change_of_the_composing_state() {
    for multiline in [false, true] {
        let log: Rc<RefCell<Vec<(bool, String)>>> = Rc::default();
        let seen = log.clone();
        let screen = render(move || {
            let value = signal(String::new());
            let seen = seen.clone();
            let field = use_text_edit(value).multiline(multiline);
            column((
                node()
                    .name("Draft")
                    .part(
                        field
                            .on_composition_change(move |composing| {
                                seen.borrow_mut().push((composing, value.get_untracked()));
                            })
                            .control(),
                    )
                    .style(style().size(200, 60)),
                node()
                    .role(Role::Button)
                    .name("elsewhere")
                    .focusable(true)
                    .style(style().size(40, 20)),
            ))
        });
        let take = || std::mem::take(&mut *log.borrow_mut());
        let field = screen.get_by_role(Role::TextBox, "Draft");
        field.type_text("a");
        assert_eq!(take(), [], "{multiline}: plain typing does not compose");
        field.compose("か");
        field.compose("かな");
        assert_eq!(
            take(),
            [(true, "a".to_string())],
            "{multiline}: one start for several preedits; the binding keeps the committed text"
        );
        field.commit("仮名");
        assert_eq!(
            take(),
            [(false, "a仮名".to_string())],
            "{multiline}: commit"
        );
        field.compose("x");
        field.compose("");
        assert_eq!(
            take(),
            [(true, "a仮名".to_string()), (false, "a仮名".to_string())],
            "{multiline}: a cancelled composition ends too"
        );
        field.compose("ん");
        screen.get_by_role(Role::Button, "elsewhere").focus();
        assert_eq!(
            take(),
            [(true, "a仮名".to_string()), (false, "a仮名ん".to_string())],
            "{multiline}: the blur commits the composition as it stands"
        );
        assert!(
            screen.errors().is_empty(),
            "{multiline}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn headless_editor_can_attach_again_after_its_control_is_disposed() {
    let (screen, value) = crate::support::mount(|| {
        let value = signal(String::new());
        let editor = use_text_edit(value);
        let open = signal(true);
        (
            value,
            column((
                node()
                    .role(Role::Button)
                    .name("Toggle editor")
                    .focusable(true)
                    .style(style().size(100, 30))
                    .on_press(move || open.update(|value| *value = !*value)),
                show(open, move || {
                    node()
                        .name("Reusable editor")
                        .part(editor.control())
                        .style(style().size(180, 36))
                }),
            )),
        )
    });
    screen
        .get_by_role(Role::TextBox, "Reusable editor")
        .type_text("a");
    assert_eq!(value.get_untracked(), "a");
    screen.get_by_role(Role::Button, "Toggle editor").click();
    assert!(
        screen
            .query_by_role(Role::TextBox, "Reusable editor")
            .is_none()
    );
    screen.get_by_role(Role::Button, "Toggle editor").click();
    screen
        .get_by_role(Role::TextBox, "Reusable editor")
        .type_text("b");
    assert_eq!(value.get_untracked(), "ab");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn lowering_the_user_edit_limit_preserves_the_existing_controlled_value() {
    let (screen, value) = crate::support::mount(|| {
        let value = signal(String::from("abcd"));
        let limit = signal(4usize);
        (
            value,
            column((
                node()
                    .name("Limited editor")
                    .part(use_text_edit(value).max_length(limit).control())
                    .style(style().size(180, 36)),
                node()
                    .role(Role::Button)
                    .name("Lower limit")
                    .focusable(true)
                    .style(style().size(100, 30))
                    .on_press(move || limit.set(2)),
            )),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Limited editor");
    field.focus();
    screen.get_by_role(Role::Button, "Lower limit").click();
    field.focus();
    field.press_key(Key::ArrowLeft);
    assert_eq!(value.get_untracked(), "abcd");
    assert_eq!(field.value().as_deref(), Some("abcd"));
    field.press_key(Key::Backspace);
    assert_eq!(value.get_untracked(), "abd");
    assert_eq!(field.value().as_deref(), Some("abd"));
    field.type_text("Z");
    assert_eq!(value.get_untracked(), "abd");
    assert_eq!(field.value().as_deref(), Some("abd"));
    field.press_key_with(Key::Char('a'), ctrl());
    field.type_text("XY");
    assert_eq!(value.get_untracked(), "XY");
    assert_eq!(field.value().as_deref(), Some("XY"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[path = "text_editing/forward_keys.rs"]
mod forward_keys;
