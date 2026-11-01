//! Role: Tests for long documents in native editors and the line-level editing hooks.
//!
//! A native editor lays its text out one hard line at a time: a long
//! document is published to assistive technology and edited line by line,
//! the IME is given the lines around the caret, line colors run for the
//! lines drawn and carry their state from line to line, a behavior reads
//! the lines its request found, and the session is reported and written
//! without the text.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::interaction::text_edit_policy;
use kivact::prelude::*;
use kivact::primitive::{
    EditContext, EditOutcome, EditRequest, EditState, NodeRefExt, TextEditing, TextSession, part,
};
use kivact::test::Screen;

use crate::support::mount;

/// `lines` lines of about 30 bytes each.
fn document(lines: usize) -> String {
    (0..lines)
        .map(|i| format!("line {i} of a long document\n"))
        .collect()
}

/// The byte offset where line `line` of `text` starts.
fn line_start(text: &str, line: usize) -> usize {
    match line {
        0 => 0,
        _ => text.match_indices('\n').nth(line - 1).expect("the line").0 + 1,
    }
}

fn chars(text: &str, bytes: usize) -> u32 {
    text[..bytes].chars().count() as u32
}

/// An editor named "Doc" holding `text`, `height` high, with the standard
/// key policy, and its reference.
fn editor(text: String, height: u32) -> (Screen, NodeRef) {
    mount(move || {
        let editing = TextEditing::new(text.as_str())
            .multiline(true)
            .behavior(text_edit_policy);
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Doc")
            .focusable(true)
            .text_editing(editing);
        let reference = editor.node_ref();
        (
            reference,
            node().part(editor).style(style().size(400, height)),
        )
    })
}

#[test]
fn a_long_document_is_published_and_edited_line_by_line() {
    // More than 64 KiB: shaped as one paragraph, its line ranges past
    // 64 KiB were wrong and the whole accessibility tree was refused.
    let text = document(12_000);
    let (screen, reference) = editor(text.clone(), 200);
    let field = screen.get_by_role(Role::TextBox, "Doc");
    assert_eq!(field.value().as_deref(), Some(text.as_str()));
    field.focus();
    // The caret goes to line 6000 without copying the text.
    let at = line_start(&text, 6000) + 5;
    reference.set_edit_session(TextSession::caret_at(at));
    screen.run_until_idle();
    let offset = chars(&text, at);
    assert_eq!(field.text_selection(), Some((offset, offset)));
    field.type_text("x");
    let mut expected = text.clone();
    expected.insert(at, 'x');
    assert_eq!(field.value().as_deref(), Some(expected.as_str()));
    assert_eq!(field.text_selection(), Some((offset + 1, offset + 1)));
    // The lines around it, read alone.
    let layout = reference.text_layout(5999..6002).unwrap();
    assert_eq!(layout.line_count, 12_001);
    assert_eq!(layout.lines.len(), 3);
    let row = "linex 6000 of a long document
"
    .len();
    assert_eq!(layout.lines[1].range, at - 5..at - 5 + row);
    assert!((layout.caret.y - layout.lines[1].rect.y).abs() < 1.0);
    let first = chars(&expected, line_start(&expected, 5999));
    assert_eq!(layout.carets[0].offset, first);
    assert_eq!(field.text_caret(), Some(layout.caret));
    // A line break moves the lines after it, down by a line.
    field.press_key(Key::Enter);
    expected.insert(at + 1, '\n');
    assert_eq!(field.value().as_deref(), Some(expected.as_str()));
    assert_eq!(field.text_selection(), Some((offset + 2, offset + 2)));
    let broken = reference.text_layout(5999..6003).unwrap();
    assert_eq!(broken.line_count, 12_002);
    let height = layout.lines[1].rect.y - layout.lines[0].rect.y;
    assert!((broken.lines[3].rect.y - layout.lines[2].rect.y - height).abs() < 0.5);
    assert!((broken.caret.y - layout.caret.y - height).abs() < 0.5);
    assert_eq!(field.text_caret(), Some(broken.caret));
    // Deleting it joins the lines again.
    field.press_key(Key::Backspace);
    expected.remove(at + 1);
    assert_eq!(field.value().as_deref(), Some(expected.as_str()));
    assert_eq!(reference.text_layout(0..0).unwrap().line_count, 12_001);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// About 100 KiB of words on one line, some of them not ASCII.
fn long_line() -> String {
    (0..20_000)
        .map(|i| {
            if i % 7 == 0 {
                "日本語の文 "
            } else {
                "word"
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn a_line_longer_than_64_kib_is_published_and_its_caret_moves() {
    // Shaped as one piece, its carets past 64 KiB were wrong; it is
    // shaped in pieces joined into one line.
    let text = long_line();
    assert!(text.len() > 100 * 1024 && !text.contains('\n'));
    let (screen, reference) = mount({
        let text = text.clone();
        move || {
            let editing = TextEditing::new(text.as_str()).behavior(text_edit_policy);
            let editor = part(|state| state)
                .role(Role::TextBox)
                .name("Long")
                .focusable(true)
                .text_editing(editing);
            let reference = editor.node_ref();
            (reference, node().part(editor).style(style().size(400, 40)))
        }
    });
    let field = screen.get_by_role(Role::TextBox, "Long");
    assert_eq!(field.value().as_deref(), Some(text.as_str()));
    let count = text.chars().count() as u32;
    // One caret per scalar, in order and without a jump where the pieces
    // meet.
    let layout = reference.text_layout(0..1).unwrap();
    assert_eq!(layout.line_count, 1);
    assert_eq!(layout.carets.len() as u32, count + 1);
    let gaps = layout.carets.windows(2).map(|w| {
        assert_eq!(w[1].offset, w[0].offset + 1);
        w[1].rect.x - w[0].rect.x
    });
    let widest = gaps.fold(0.0f32, |m, gap| {
        assert!(gap >= -0.01, "carets go back: {gap}");
        m.max(gap)
    });
    assert!(widest < 40.0, "a jump of {widest}");
    // The caret goes to the end and back, scrolled into the box.
    field.focus();
    field.press_key(Key::End);
    assert_eq!(field.text_selection(), Some((count, count)));
    let bounds = field.rect().unwrap();
    let caret = field.text_caret().unwrap();
    assert!(
        caret.x >= bounds.x && caret.x <= bounds.right(),
        "{caret:?}"
    );
    assert_eq!(
        field.text_caret(),
        Some(reference.text_layout(0..0).unwrap().caret)
    );
    field.press_key(Key::Home);
    field.press_key(Key::ArrowRight);
    field.press_key(Key::ArrowRight);
    assert_eq!(field.text_selection(), Some((2, 2)));
    // Typing past the first 64 KiB edits the right place.
    let at = text.char_indices().nth(70_000).unwrap().0;
    reference.set_edit_session(TextSession::caret_at(at));
    screen.run_until_idle();
    assert_eq!(field.text_selection(), Some((70_000, 70_000)));
    field.type_text("Z");
    let mut expected = text.clone();
    expected.insert(at, 'Z');
    assert_eq!(field.value().as_deref(), Some(expected.as_str()));
    assert_eq!(field.text_selection(), Some((70_001, 70_001)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_line_longer_than_64_kib_among_short_ones_keeps_the_runs_in_a_row() {
    let long = long_line();
    let text = format!("first\n{long}\nlast");
    let (screen, reference) = editor(text.clone(), 200);
    let field = screen.get_by_role(Role::TextBox, "Doc");
    field.focus();
    // The caret on the last line, after the long one: the published runs
    // run on from one to the next.
    let at = text.len() - "last".len();
    reference.set_edit_session(TextSession::caret_at(at));
    screen.run_until_idle();
    let offset = chars(&text, at);
    assert_eq!(field.text_selection(), Some((offset, offset)));
    // Back over the line break, to the long line's end, and into it.
    field.press_key(Key::ArrowLeft);
    assert_eq!(field.text_selection(), Some((offset - 1, offset - 1)));
    let inside = 6 + long.char_indices().nth(90_000).unwrap().0;
    reference.set_edit_session(TextSession::caret_at(inside));
    screen.run_until_idle();
    let offset = chars(&text, inside);
    assert_eq!(field.text_selection(), Some((offset, offset)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_ime_is_given_the_lines_around_the_caret_of_a_long_document() {
    let text = document(2000);
    let (screen, reference) = editor(text.clone(), 200);
    let field = screen.get_by_role(Role::TextBox, "Doc");
    field.focus();
    let line = line_start(&text, 1000);
    reference.set_edit_session(TextSession::caret_at(line + 5));
    screen.run_until_idle();
    let ime = screen.ime_text().unwrap();
    // Whole lines around the caret, a few thousand bytes, not the text.
    assert!(
        ime.text.len() > 2000 && ime.text.len() < 4200,
        "{}",
        ime.text.len()
    );
    assert!(ime.text.starts_with("line ") && ime.text.ends_with('\n'));
    let all: Vec<char> = text.chars().collect();
    let shown: String = all[ime.start..ime.start + ime.text.chars().count()]
        .iter()
        .collect();
    assert_eq!(shown, ime.text);
    let caret = chars(&text, line + 5) as usize - ime.start;
    assert_eq!(ime.selection, caret..caret);
    // A composition in the middle is in the excerpt's offsets too.
    field.compose("ka");
    let ime = screen.ime_text().unwrap();
    assert_eq!(ime.composition, Some(caret..caret + 2));
    assert_eq!(ime.char_rects.len(), 3);
    field.commit("か");
    // A reconversion of a word of the middle line replaces that word.
    let word = chars(&text, line) as usize;
    field.reconvert(word..word + 4);
    assert_eq!(
        reference.edit_session().unwrap().selection(),
        line..line + 4
    );
    field.commit("LINE");
    let value = field.value().unwrap();
    assert!(value[line..].starts_with("LINE か1000 of a long document\n"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn line_colors_run_for_the_drawn_lines_and_pass_their_state_on() {
    let calls: Rc<RefCell<Vec<(String, bool)>>> = Rc::default();
    let seen = calls.clone();
    let text = document(1000);
    let (screen, reference) = mount(move || {
        // A block comment opened on a line stays open on the next ones.
        let editing = TextEditing::new(text.as_str()).multiline(true).highlight(
            (),
            move |_: &(), line: &str, open: &bool| {
                seen.borrow_mut().push((line.to_string(), *open));
                let open = match (line.contains("/*"), line.contains("*/")) {
                    (true, _) => true,
                    (_, true) => false,
                    _ => *open,
                };
                (Vec::new(), open)
            },
        );
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Colored")
            .focusable(true)
            .text_editing(editing);
        let reference = editor.node_ref();
        (reference, node().part(editor).style(style().size(300, 100)))
    });
    screen.screenshot();
    let drawn = calls.borrow().len();
    assert!((1..20).contains(&drawn), "only the drawn lines: {drawn}");
    let field = screen.get_by_role(Role::TextBox, "Colored");
    field.focus();
    reference.set_edit_session(TextSession::caret_at(0));
    screen.run_until_idle();
    // An edit of the first line colors it again; the lines after it start
    // from the same state and keep their colors.
    field.type_text("a");
    screen.screenshot();
    assert_eq!(
        calls.borrow()[drawn..],
        [(String::from("aline 0 of a long document"), false)]
    );
    // Opening a comment there colors the drawn lines after it again, from
    // the open state, and no others.
    reference.set_edit_session(TextSession::caret_at(0));
    field.type_text("/*");
    screen.screenshot();
    let after = calls.borrow()[drawn + 1..].to_vec();
    assert_eq!(
        after[0],
        (String::from("/*aline 0 of a long document"), false)
    );
    assert!(after.len() == drawn, "{after:?}");
    assert!(after[1..].iter().all(|(_, open)| *open), "{after:?}");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_behavior_reads_the_lines_its_request_found() {
    type Seen = Rc<RefCell<Vec<(Option<usize>, Option<usize>, Option<std::ops::Range<usize>>)>>>;
    let seen: Seen = Rc::default();
    let record = seen.clone();
    let behavior = move |state: &mut EditState, request: EditRequest<'_>, cx: &EditContext<'_>| {
        let EditRequest::Key { key, .. } = request else {
            return EditOutcome::NotEditing;
        };
        match key {
            // Down a line, to the caret nearest the same x.
            Key::ArrowDown => {
                let caret = state.caret();
                let line = cx.line_of(state, caret).unwrap();
                record.borrow_mut().push((
                    cx.line_count(state),
                    Some(line),
                    cx.line_range(state, line),
                ));
                let x = cx.x_of(state, caret).unwrap();
                let to = cx.offset_at(state, line + 1, x).unwrap();
                state.move_to(to, false);
                EditOutcome::Done { changed: false }
            }
            // After a change, the lines are not of the text any more.
            Key::Char('z') => {
                state.replace(0..0, "z");
                record
                    .borrow_mut()
                    .push((cx.line_count(state), cx.line_of(state, 0), None));
                EditOutcome::Done { changed: true }
            }
            _ => EditOutcome::NotEditing,
        }
    };
    let (screen, reference) = mount(move || {
        let editing = TextEditing::new("ab\r\nabcdef\nx")
            .multiline(true)
            .behavior(behavior);
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Lines")
            .focusable(true)
            .text_editing(editing);
        let reference = editor.node_ref();
        (reference, node().part(editor).style(style().size(300, 120)))
    });
    let field = screen.get_by_role(Role::TextBox, "Lines");
    field.focus();
    reference.set_edit_session(TextSession::caret_at(2));
    screen.run_until_idle();
    field.press_key(Key::ArrowDown);
    assert_eq!(reference.edit_session().unwrap().caret(), 6);
    field.press_key(Key::ArrowDown);
    assert_eq!(reference.edit_session().unwrap().caret(), 12);
    field.press_key(Key::Char('z'));
    assert_eq!(
        *seen.borrow(),
        [
            (Some(3), Some(0), Some(0..2)),
            (Some(3), Some(1), Some(4..10)),
            (None, None, None),
        ]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn session_reports_follow_moves_edits_and_writes() {
    let sessions: Rc<RefCell<Vec<TextSession>>> = Rc::default();
    let seen = sessions.clone();
    let (screen, (reference, disabled, read_only)) = mount(move || {
        let disabled = signal(false);
        let read_only = signal(false);
        let editing = TextEditing::new("abc\ndef")
            .multiline(true)
            .read_only(read_only)
            .behavior(text_edit_policy)
            .on_selection(move |session| seen.borrow_mut().push(session));
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Session")
            .focusable(true)
            .text_editing(editing);
        let reference = editor.node_ref();
        (
            (reference, disabled, read_only),
            node()
                .part(editor)
                .disabled(disabled)
                .style(style().size(200, 80)),
        )
    });
    let take = || std::mem::take(&mut *sessions.borrow_mut());
    // The first session, once laid out.
    assert_eq!(take(), [TextSession::caret_at(7)]);
    let field = screen.get_by_role(Role::TextBox, "Session");
    field.focus();
    assert_eq!(take(), []);
    field.press_key(Key::ArrowLeft);
    assert_eq!(take(), [TextSession::caret_at(6)]);
    field.type_text("x");
    assert_eq!(take(), [TextSession::caret_at(7)]);
    // A written session is reported, clamped into the text.
    reference.set_edit_session(TextSession::select(0, 3));
    screen.run_until_idle();
    assert_eq!(take(), [TextSession::select(0, 3)]);
    assert_eq!(field.text_selection(), Some((0, 3)));
    reference.set_edit_session(TextSession::caret_at(100));
    screen.run_until_idle();
    assert_eq!(take(), [TextSession::caret_at(8)]);
    reference.set_edit_session(TextSession::caret_at(8));
    screen.run_until_idle();
    assert_eq!(take(), []);
    // A read-only editor takes a selection; a disabled one nothing.
    read_only.set(true);
    screen.run_until_idle();
    reference.set_edit_session(TextSession::select(4, 2));
    screen.run_until_idle();
    assert_eq!(take(), [TextSession::select(4, 2)]);
    disabled.set(true);
    screen.run_until_idle();
    reference.set_edit_session(TextSession::caret_at(0));
    screen.run_until_idle();
    assert_eq!(take(), []);
    assert_eq!(reference.edit_session().unwrap(), TextSession::select(4, 2));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_editor_in_a_scroll_container_draws_the_lines_scrolled_into_view() {
    use kivact::primitive::TextHighlight;
    use kivact::style::rgb;
    // One colored word, on line 4000 of 5000.
    let text: String = (0..5000)
        .map(|i| if i == 4000 { "WORD\n" } else { "x\n" })
        .collect();
    let (screen, reference) = mount(move || {
        let editing = TextEditing::new(text.as_str()).multiline(true).highlight(
            (),
            |_: &(), line: &str, _: &()| {
                let spans = line
                    .find("WORD")
                    .map(|at| TextHighlight {
                        range: at..at + 4,
                        color: rgb(0xD01010),
                    })
                    .into_iter()
                    .collect();
                (spans, ())
            },
        );
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Tall")
            .focusable(true)
            .text_editing(editing);
        let reference = editor.node_ref();
        (
            reference,
            scroll(node().part(editor).style(style().font_size(22)))
                .test_id("scroller")
                .style(style().size(300, 200)),
        )
    });
    let lines = reference.text_layout(0..2).unwrap().lines;
    let height = lines[1].rect.y - lines[0].rect.y;
    screen
        .get_by_test_id("scroller")
        .scroll(0.0, 4000.0 * height - 60.0);
    let layout = reference.text_layout(4000..4001).unwrap();
    let word = layout.lines[0].rect;
    let view = screen.get_by_test_id("scroller").rect().unwrap();
    assert!(word.y > view.y && word.y + word.height < view.y + view.height);
    let scale = screen.scale();
    let px = |v: f32| (v * scale).round().max(0.0) as u32;
    let shot = screen
        .screenshot()
        .crop(px(word.x), px(word.y), px(word.width), px(word.height));
    let red = shot.pixels.iter().any(|p| {
        let (r, g, b) = (p >> 16, (p >> 8) & 0xff, p & 0xff);
        r > 160 && g < 90 && b < 90
    });
    assert!(red, "the scrolled-in line is drawn with its color");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A panic in the line-color function draws that line without colors, is
/// reported once, and leaves the other lines colored.
#[test]
fn a_panicking_highlight_draws_the_line_uncolored_and_is_reported_once() {
    use kivact::primitive::TextHighlight;
    use kivact::style::rgb;
    let (screen, reference) = mount(|| {
        let editing = TextEditing::new(
            "WORD
WORD BOOM
",
        )
        .multiline(true)
        .highlight((), |_: &(), line: &str, _: &()| {
            if line.contains("BOOM") {
                panic!("the highlight failed");
            }
            let spans = line
                .find("WORD")
                .map(|at| TextHighlight {
                    range: at..at + 4,
                    color: rgb(0xD01010),
                })
                .into_iter()
                .collect();
            (spans, ())
        });
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Colored")
            .focusable(true)
            .text_editing(editing);
        let reference = editor.node_ref();
        (
            reference,
            node()
                .part(editor)
                .style(style().size(300, 120).font_size(22)),
        )
    });
    screen.run_until_idle();
    let lines = reference.text_layout(0..15).unwrap().lines;
    let scale = screen.scale();
    let px = |v: f32| (v * scale).round().max(0.0) as u32;
    let shot = screen.screenshot();
    let has_red = |rect: kivact::draw::Rect| {
        shot.crop(px(rect.x), px(rect.y), px(rect.width), px(rect.height))
            .pixels
            .iter()
            .any(|p| {
                let (r, g, b) = (p >> 16, (p >> 8) & 0xff, p & 0xff);
                r > 160 && g < 90 && b < 90
            })
    };
    assert!(has_red(lines[0].rect), "the first line is colored");
    let second = lines[1].rect;
    assert!(!has_red(second), "the failing line is drawn uncolored");
    assert_eq!(screen.errors().len(), 1, "{:?}", screen.errors());
    assert!(format!("{:?}", screen.errors()[0]).contains("the highlight failed"));
}

#[test]
fn a_panicking_behavior_leaves_the_key_unhandled_and_is_reported_once() {
    use kivact::view::Modifiers;
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    let (screen, reference) = mount(|| {
        let editing = TextEditing::new("abc").behavior(
            |state: &mut EditState, request: EditRequest<'_>, cx: &EditContext<'_>| {
                if let EditRequest::Key {
                    key: Key::Char('p'),
                    ..
                } = request
                {
                    // A change, then a panic: the change is taken back.
                    state.replace(0..0, "p");
                    panic!("the behavior failed");
                }
                text_edit_policy(state, request, cx)
            },
        );
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Fragile")
            .focusable(true)
            .text_editing(editing);
        let reference = editor.node_ref();
        (reference, node().part(editor).style(style().size(200, 40)))
    });
    let field = screen.get_by_role(Role::TextBox, "Fragile");
    field.press_key_with(Key::Char('p'), ctrl);
    assert_eq!(reference.edit_state().unwrap().value(), "abc");
    assert_eq!(field.value().as_deref(), Some("abc"));
    assert_eq!(screen.errors().len(), 1);
    assert!(format!("{:?}", screen.errors()[0]).contains("the behavior failed"));
    // Once: the next panic is not reported again; other keys still work.
    field.press_key_with(Key::Char('p'), ctrl);
    assert_eq!(screen.errors().len(), 1);
    field.type_text("x");
    assert_eq!(field.value().as_deref(), Some("abcx"));
}

#[test]
fn a_value_written_while_composing_is_taken_when_the_composition_ends() {
    use kivact::primitive::TextEditEvent;
    let copy = Rc::new(RefCell::new(String::from("abc")));
    let seen = copy.clone();
    let (screen, value) = mount(move || {
        let value: Signal<Rc<str>> = signal(Rc::from("abc"));
        // The copy follows the reported changes only.
        let editing = TextEditing::new(value).on_edit(move |event: TextEditEvent| {
            assert!(event.apply(&mut seen.borrow_mut()), "{event:?}");
        });
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Held")
            .focusable(true)
            .text_editing(editing);
        (value, node().part(editor).style(style().size(200, 40)))
    });
    let field = screen.get_by_role(Role::TextBox, "Held");
    field.compose("ka");
    assert_eq!(*copy.borrow(), "abcka");
    // The application replaces the value while the IME composes: it waits.
    value.set(Rc::from("new value"));
    screen.run_until_idle();
    assert_eq!(field.value().as_deref(), Some("abcka"));
    // The commit ends the composition: the value is taken, and reported.
    field.commit("か");
    assert_eq!(field.value().as_deref(), Some("new value"));
    assert_eq!(*copy.borrow(), "new value");
    // A cancelled composition takes it too.
    field.compose("x");
    value.set(Rc::from("other"));
    screen.run_until_idle();
    field.compose("");
    assert_eq!(field.value().as_deref(), Some("other"));
    assert_eq!(*copy.borrow(), "other");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_multiline_value_ending_in_a_line_feed_keeps_one_last_empty_line() {
    // Editing the line before the last empty one split that empty line
    // again, so the editor showed one more line after every edit.
    let text = String::from("one\ntwo\nthree\n");
    let (screen, reference) = editor(text.clone(), 200);
    let field = screen.get_by_role(Role::TextBox, "Doc");
    field.focus();
    assert_eq!(reference.text_layout(0..0).unwrap().line_count, 4);
    let end = text.len() - 1;
    reference.set_edit_session(TextSession::caret_at(end));
    screen.run_until_idle();
    field.type_text("x");
    assert_eq!(field.value().as_deref(), Some("one\ntwo\nthreex\n"));
    let layout = reference.text_layout(0..8).unwrap();
    assert_eq!(layout.line_count, 4);
    assert_eq!(layout.lines.len(), 4);
    assert_eq!(layout.lines[2].range, 8..15);
    assert_eq!(layout.lines[3].range, 15..15);
    field.press_key(Key::Backspace);
    field.press_key(Key::Backspace);
    assert_eq!(field.value().as_deref(), Some("one\ntwo\nthre\n"));
    assert_eq!(reference.text_layout(0..0).unwrap().line_count, 4);
    // The last line itself, then the line before it again.
    reference.set_edit_session(TextSession::caret_at(13));
    screen.run_until_idle();
    field.type_text("y");
    assert_eq!(field.value().as_deref(), Some("one\ntwo\nthre\ny"));
    assert_eq!(reference.text_layout(0..0).unwrap().line_count, 4);
    field.press_key(Key::Enter);
    reference.set_edit_session(TextSession::caret_at(13));
    screen.run_until_idle();
    field.type_text("z");
    assert_eq!(field.value().as_deref(), Some("one\ntwo\nthre\nzy\n"));
    assert_eq!(reference.text_layout(0..0).unwrap().line_count, 5);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_state_of_the_same_text_from_another_revision_keeps_its_selection() {
    let text = String::from("hello world");
    let (screen, reference) = editor(text.clone(), 40);
    let field = screen.get_by_role(Role::TextBox, "Doc");
    field.focus();
    // A state made apart from the editor's (another revision) holding the
    // same text: only its selection is new.
    let mut state = EditState::new(&text);
    state.set_session(TextSession::select(0, 5));
    reference.set_edit_state(state);
    screen.run_until_idle();
    assert_eq!(reference.edit_session().unwrap().selection(), 0..5);
    assert_eq!(field.text_selection(), Some((0, 5)));
    field.type_text("H");
    assert_eq!(field.value().as_deref(), Some("H world"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_pieces_of_a_long_line_meet_without_a_gap() {
    // A line past 16 KiB is shaped in pieces joined end to end: each caret
    // steps as far as the same character's caret does in the first piece.
    for unit in ["word ", "abcdefghij"] {
        let text = unit.repeat(20 * 1024 / unit.len());
        let (screen, reference) = mount({
            let text = text.clone();
            move || {
                let editing = TextEditing::new(text.as_str()).behavior(text_edit_policy);
                let editor = part(|state| state)
                    .role(Role::TextBox)
                    .name("Long")
                    .focusable(true)
                    .text_editing(editing);
                let reference = editor.node_ref();
                (reference, node().part(editor).style(style().size(400, 40)))
            }
        });
        let layout = reference.text_layout(0..1).unwrap();
        let steps: Vec<f32> = layout
            .carets
            .windows(2)
            .map(|w| w[1].rect.x - w[0].rect.x)
            .collect();
        assert_eq!(steps.len(), text.len());
        for (i, step) in steps.iter().enumerate() {
            let first = steps[i % unit.len()];
            assert!(
                (step - first).abs() < 0.05,
                "{unit:?}: the caret at {i} steps {step}, not {first}"
            );
        }
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn the_ime_excerpt_of_a_long_line_is_cut_between_graphemes() {
    // The IME is given 2048 bytes on each side of the caret of a long
    // line; the cut moves back to a grapheme boundary rather than split
    // an accent from its letter, or a CR from its LF.
    let text = format!("{}e\u{301}{}\r\nz", "a".repeat(3000), "b".repeat(3000));
    let (screen, reference) = editor(text.clone(), 200);
    let field = screen.get_by_role(Role::TextBox, "Doc");
    field.focus();
    let accent = 3001;
    reference.set_edit_session(TextSession::caret_at(accent - 2048));
    screen.run_until_idle();
    let ime = screen.ime_text().unwrap();
    assert_eq!(ime.start, 0);
    assert_eq!(ime.text, "a".repeat(3000));
    let lf = text.find('\n').unwrap();
    reference.set_edit_session(TextSession::caret_at(lf - 2048));
    screen.run_until_idle();
    let ime = screen.ime_text().unwrap();
    assert!(
        ime.text.ends_with('b'),
        "{:?}",
        &ime.text[ime.text.len() - 4..]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn masked_hit_uses_original_combining_grapheme_boundaries() {
    let (screen, reference) = mount(|| {
        let editing = TextEditing::new("a\u{301}b").password(true);
        let editor = part(|state| state)
            .role(Role::TextBox)
            .name("Secret")
            .focusable(true)
            .text_editing(editing);
        let reference = editor.node_ref();
        (reference, node().part(editor).style(style().size(200, 40)))
    });
    let field = screen.get_by_role(Role::TextBox, "Secret");
    field.focus();
    let layout = reference.text_layout(0..1).expect("masked layout");
    let interior = layout.carets[1].rect;
    screen.click_at(interior.x + 0.1, interior.y + interior.height / 2.0);
    let caret = reference.edit_session().expect("session").caret();
    assert!(
        caret == 0 || caret == "a\u{301}".len(),
        "hit split grapheme at {caret}"
    );
    assert_eq!(field.value(), None);
    assert_eq!(field.text_runs(), None);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
