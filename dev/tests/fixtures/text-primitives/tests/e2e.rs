//! Role: Real-window tests of the one-line editor built from the text primitives.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::draw::Point;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{RenderOptions, Screen, WindowMode, render_with};
use kivact::view::{Key, Modifiers};
use kivact_text_primitives_fixture::line_editor;

fn render(initial: &'static str) -> (Screen, Signal<String>) {
    let slot: Rc<RefCell<Option<Signal<String>>>> = Rc::default();
    let out = slot.clone();
    let screen = render_with(
        RenderOptions {
            window: WindowMode::Real,
            gpu: cfg!(feature = "gpu"),
            ..Default::default()
        },
        move || {
            let text = signal(initial.to_string());
            *out.borrow_mut() = Some(text);
            column((
                line_editor("Title", text).test_id("editor"),
                node()
                    .test_id("other")
                    .role(Role::Button)
                    .name("other")
                    .focusable(true)
                    .style(style().size(60, 24)),
            ))
            .style(style().padding(20).gap(12).align_items(Align::Start))
        },
    );
    let text = slot.borrow().expect("setup ran");
    (screen, text)
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

/// The caret's window position now (as assistive technology sees it).
fn caret_x(screen: &Screen) -> f32 {
    screen
        .get_by_test_id("editor")
        .text_caret()
        .expect("the editor publishes its caret")
        .x
}

#[test]
fn typing_and_deleting_edit_the_text_and_publish_it() {
    let (screen, text) = render("");
    let editor = screen.get_by_test_id("editor");
    // The node is the text box, named for assistive technology, and
    // publishes an empty run before anything is typed.
    assert_eq!(editor.role(), Some(Role::TextBox));
    assert_eq!(editor.name(), "Title");
    assert_eq!(editor.text_runs(), Some(vec![String::new()]));
    editor.type_text("abc");
    assert_eq!(text.get(), "abc");
    assert_eq!(editor.text_runs(), Some(vec!["abc".to_string()]));
    assert_eq!(editor.text_selection(), Some((3, 3)));
    editor.press_key(Key::Backspace);
    assert_eq!(text.get(), "ab");
    editor.press_key(Key::ArrowLeft);
    editor.type_text("X");
    assert_eq!(text.get(), "aXb");
    editor.press_key(Key::Home);
    editor.press_key(Key::Delete);
    assert_eq!(text.get(), "Xb");
    assert_eq!(editor.text_runs(), Some(vec!["Xb".to_string()]));
    assert_eq!(editor.text_selection(), Some((0, 0)));
    // A change the app makes is shown and published too.
    text.set("hello".to_string());
    screen.run_until_idle();
    assert_eq!(editor.text_runs(), Some(vec!["hello".to_string()]));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn arrows_move_the_caret_by_cluster_and_shift_selects() {
    let (screen, text) = render("aあb");
    let editor = screen.get_by_test_id("editor");
    editor.focus();
    editor.press_key(Key::Home);
    let start = caret_x(&screen);
    editor.press_key(Key::ArrowRight);
    editor.press_key(Key::ArrowRight);
    // Past 'あ' (three bytes) the caret is at the scalar offset 2.
    assert_eq!(editor.text_selection(), Some((2, 2)));
    assert!(caret_x(&screen) > start);
    // Shift+arrows select; the band is drawn.
    let before = editor.screenshot();
    editor.press_key_with(Key::ArrowLeft, shift());
    editor.press_key_with(Key::ArrowLeft, shift());
    assert_eq!(editor.text_selection(), Some((2, 0)));
    assert_ne!(editor.screenshot().hash(), before.hash());
    // Typing replaces the selection.
    editor.type_text("Z");
    assert_eq!(text.get(), "Zb");
    // Ctrl+A selects everything, Delete removes it.
    editor.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(editor.text_selection(), Some((0, 2)));
    editor.press_key(Key::Delete);
    assert_eq!(text.get(), "");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn clicking_and_dragging_place_the_caret_and_select_by_hit_test() {
    let (screen, text) = render("hello world");
    let editor = screen.get_by_test_id("editor");
    editor.focus();
    // Where the caret is after k chars, read from the published caret.
    let mut x_at = Vec::new();
    editor.press_key(Key::Home);
    for _ in 0..=text.get().len() {
        x_at.push(caret_x(&screen));
        editor.press_key(Key::ArrowRight);
    }
    let y = {
        let r = editor.rect().expect("laid out");
        r.y + r.height / 2.0
    };
    // A click just right of the caret of offset 4 puts the caret there.
    screen.click_at(x_at[4] + 1.0, y);
    assert_eq!(editor.text_selection(), Some((4, 4)));
    // A click past the end is the end.
    let right = editor.rect().expect("laid out").right();
    screen.click_at(right - 3.0, y);
    assert_eq!(editor.text_selection(), Some((11, 11)));
    // A drag selects from where it started to where it ended.
    let at = editor.rect().expect("laid out");
    editor.drag(
        Point::new(x_at[2] + 1.0 - at.x, at.height / 2.0),
        Point::new(x_at[7] + 1.0 - at.x, at.height / 2.0),
    );
    assert_eq!(editor.text_selection(), Some((2, 7)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_ime_composes_commits_and_reconverts_through_the_session() {
    let (screen, text) = render("hello");
    let editor = screen.get_by_test_id("editor");
    editor.focus();
    editor.press_key(Key::Home);
    editor.press_key(Key::ArrowRight);
    // The IME is given the committed text and the caret's place.
    let ime = screen.ime_text().expect("a session while focused");
    assert_eq!((ime.text.as_str(), ime.selection.clone()), ("hello", 1..1));
    let caret_before = screen.ime_cursor_area().expect("candidate window place");

    editor.compose("にほ");
    // The composition is shown (and published) in the text, not committed.
    assert_eq!(text.get(), "hello");
    assert_eq!(editor.text_runs(), Some(vec!["hにほello".to_string()]));
    let caret_composing = screen.ime_cursor_area().expect("candidate window place");
    assert!(
        caret_composing.x > caret_before.x,
        "the candidate window follows the composition: {caret_before:?} {caret_composing:?}"
    );
    editor.commit("日本");
    assert_eq!(text.get(), "h日本ello");
    assert_eq!(editor.text_runs(), Some(vec!["h日本ello".to_string()]));
    assert_eq!(editor.text_selection(), Some((3, 3)));
    assert_eq!(screen.ime_text().expect("session").text, "h日本ello");

    // A composition that is cancelled leaves the text as it was.
    editor.compose("x");
    screen.ime_disabled();
    assert_eq!(editor.text_runs(), Some(vec!["h日本ello".to_string()]));

    // Reconversion: the IME takes back '日本' (chars 1..3), and the next
    // composition replaces it.
    editor.reconvert(1..3);
    assert_eq!(editor.text_selection(), Some((1, 3)));
    editor.compose("にほん");
    assert_eq!(text.get(), "hello");
    assert_eq!(editor.text_runs(), Some(vec!["hにほんello".to_string()]));
    editor.commit("二本");
    assert_eq!(text.get(), "h二本ello");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn only_the_focused_editor_takes_the_ime() {
    let (screen, _text) = render("abc");
    assert!(screen.ime_text().is_none());
    screen.get_by_test_id("editor").focus();
    assert!(screen.ime_text().is_some());
    screen.get_by_test_id("other").focus();
    assert!(screen.ime_text().is_none());
    assert!(screen.ime_cursor_area().is_none());
}

#[test]
fn movement_and_deletion_preserve_combining_and_emoji_clusters() {
    let (screen, text) = render("a\u{301}👩‍👩‍👧‍👦b");
    let editor = screen.get_by_test_id("editor");
    editor.focus();
    editor.press_key(Key::Home);
    editor.press_key(Key::ArrowRight);
    assert_eq!(editor.text_selection(), Some((2, 2)));
    editor.press_key(Key::Delete);
    assert_eq!(text.get(), "a\u{301}b");
    editor.press_key(Key::Backspace);
    assert_eq!(text.get(), "b");
    assert_eq!(editor.text_selection(), Some((0, 0)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
