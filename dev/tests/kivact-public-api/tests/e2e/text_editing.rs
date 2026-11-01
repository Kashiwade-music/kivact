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
use kivact::test::{Screen, render};
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
