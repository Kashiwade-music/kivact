//! Role: Tests for the three text primitives on real windows.
//!
//! A shaped paragraph (`TextParagraph`: measured by a primitive's measure
//! function, laid out and drawn by its draw closure, and queried for
//! carets, hits and selection bands), the IME session of a node that edits
//! its own text (`NodeExt::ime_session`, `on_ime`) and the text runs a node
//! publishes for assistive technology (`NodeExt::semantics_text`). The
//! editor built from all three is the `kivact-text-primitives` fixture.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::support::mount;
use kivact::draw::{Point, Rect, ShapedText, TextParagraph, TextStyle};
use kivact::prelude::*;
use kivact::primitive::{
    Constraints, ImeEvent, ImeSession, NodeExt, Primitive, SemanticsText, SemanticsTextRun,
    TextCaret, TextRunId, primitive,
};
use kivact::style::Align;

/// The text style the tests shape in: a fixed size, so widths are stable.
fn style_14() -> TextStyle {
    TextStyle {
        size: 14.0,
        ..TextStyle::default()
    }
}

/// What a paragraph primitive remembers: the paragraph its measure
/// function made, and the one its draw closure laid out.
#[derive(Default)]
struct Seen {
    measured: Option<TextParagraph>,
    drawn: Option<TextParagraph>,
}

/// A primitive that measures `text` as a paragraph wrapped at `width` and
/// draws the paragraph it lays out in its draw closure.
fn paragraph_node(text: &'static str, width: Option<f32>, seen: Rc<RefCell<Seen>>) -> Primitive {
    let measured = seen.clone();
    primitive()
        .measure(text.to_string(), move |text: &String, c: Constraints| {
            let paragraph = c.measure_paragraph(text, &style_14(), width);
            let size = paragraph.size();
            measured.borrow_mut().measured = Some(paragraph);
            size
        })
        .draw(move |d, _| {
            let paragraph = d.layout_paragraph(text, &style_14(), width);
            d.paragraph(&paragraph, Point::new(0.0, 0.0), ColorToken::Text);
            seen.borrow_mut().drawn = Some(paragraph);
        })
        .test_id("p")
}

/// The measure function and the draw closure shape the same text in the
/// same style at the same width: one size, which is the node's, and one
/// shaping.
#[test]
fn a_paragraph_measures_and_lays_out_the_same_shaping() {
    let (screen, seen) = mount(|| {
        let seen = Rc::new(RefCell::new(Seen::default()));
        (
            seen.clone(),
            column(paragraph_node("Hello wrapping paragraph", Some(90.0), seen))
                .style(style().align_items(Align::Start)),
        )
    });
    let seen = seen.borrow();
    let (measured, drawn) = (
        seen.measured.as_ref().expect("measured"),
        seen.drawn.as_ref().expect("drawn"),
    );
    assert_eq!(measured.text(), drawn.text());
    assert_eq!(measured.size(), drawn.size());
    assert!(
        measured.lines().len() >= 2,
        "90 px wraps it: {:?}",
        measured.lines()
    );
    let rect = screen.get_by_test_id("p").rect().expect("laid out");
    assert!((rect.width - measured.size().width).abs() < 1.0, "{rect:?}");
    assert!(
        (rect.height - measured.size().height).abs() < 1.0,
        "{rect:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// `Draw::paragraph` draws the glyphs it measured, those `Draw::text` draws.
#[test]
fn a_paragraph_draws_the_pixels_of_the_text_it_measured() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                node()
                    .test_id("text")
                    .style(style().size(200, 32))
                    .draw(|d, _| d.text("Paragraph pixels", Point::new(0.0, 0.0), style_14())),
                node()
                    .test_id("paragraph")
                    .style(style().size(200, 32))
                    .draw(|d, _| {
                        let paragraph = d.layout_paragraph("Paragraph pixels", &style_14(), None);
                        d.paragraph(&paragraph, Point::new(0.0, 0.0), ColorToken::Text);
                    }),
            )),
        )
    });
    let (text, paragraph) = (
        screen.get_by_test_id("text").screenshot(),
        screen.get_by_test_id("paragraph").screenshot(),
    );
    assert_eq!(
        (text.width, text.height),
        (paragraph.width, paragraph.height)
    );
    // The same glyphs at the same places: no pixel differs by more than the
    // rasterizer's own rounding (two nodes drawing the same `Draw::text`
    // differ by one level in a few edge pixels too).
    let channel = |pixel: Option<u32>, shift: u32| (pixel.unwrap_or(0) >> shift) & 0xff;
    let far: Vec<_> = (0..text.height)
        .flat_map(|y| (0..text.width).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let (a, b) = (text.pixel(*x, *y), paragraph.pixel(*x, *y));
            [0, 8, 16]
                .into_iter()
                .any(|shift| channel(a, shift).abs_diff(channel(b, shift)) > 3)
        })
        .take(10)
        .collect();
    assert!(far.is_empty(), "pixels that differ: {far:?}");
    let blank = screen.get_by_test_id("paragraph").screenshot();
    let inked = (0..blank.height)
        .flat_map(|y| (0..blank.width).map(move |x| (x, y)))
        .filter(|(x, y)| blank.pixel(*x, *y) != blank.pixel(0, 0))
        .count();
    assert!(inked > 20, "the text left ink: {inked}");
}

/// Offsets are bytes, carets and hits agree, and a selection is one band per
/// line it touches.
#[test]
fn a_paragraph_answers_carets_hits_and_selection_bands() {
    let (_screen, seen) = mount(|| {
        let seen = Rc::new(RefCell::new(Seen::default()));
        (
            seen.clone(),
            column(paragraph_node("one two three four", Some(50.0), seen))
                .style(style().align_items(Align::Start)),
        )
    });
    let paragraph = seen.borrow().drawn.clone().expect("drawn");
    let text = paragraph.text().to_string();
    assert!(paragraph.lines().len() >= 3, "{:?}", paragraph.lines());
    // Every boundary hits itself, from the middle of its caret.
    for (offset, _) in text.char_indices().chain([(text.len(), ' ')]) {
        let caret = paragraph.caret(offset);
        let middle = Point::new(caret.x, caret.y + caret.height / 2.0);
        assert_eq!(paragraph.hit(middle), offset, "caret {caret:?}");
    }
    // Carets run along a line and drop to the next.
    assert_eq!(paragraph.caret(0).x, 0.0);
    assert!(paragraph.caret(3).x > paragraph.caret(1).x);
    let first_line_end = paragraph.lines()[0].range.end;
    assert!(paragraph.caret(first_line_end).y > paragraph.caret(0).y);
    // Past a line's end gives the line's end; above and below, the ends.
    let first = paragraph.lines()[0].rect;
    let at_end = paragraph.hit(Point::new(first.right() + 500.0, first.y + 1.0));
    assert!(
        at_end <= first_line_end && at_end >= first_line_end - 1,
        "{at_end} vs {first_line_end}"
    );
    assert_eq!(paragraph.hit(Point::new(1.0, -50.0)), 0);
    assert_eq!(
        paragraph.hit(Point::new(1000.0, paragraph.size().height + 50.0)),
        text.len()
    );
    // A multi-byte offset is clamped to the char boundary at or before it.
    // Selection bands: one per line the range touches, as high as the line.
    let bands = paragraph.rects(2..text.len());
    assert_eq!(bands.len(), paragraph.lines().len());
    for (band, line) in bands.iter().zip(paragraph.lines()) {
        assert_eq!((band.y, band.height), (line.rect.y, line.rect.height));
        assert!(band.width > 0.0);
    }
    assert_eq!(paragraph.rects(4..4), Vec::<Rect>::new());
    let inside = paragraph.rects(0..3);
    assert_eq!(inside.len(), 1);
    assert_eq!(inside[0].x, 0.0);
}

/// Offsets are UTF-8 bytes, clamped to the char boundary at or before them.
#[test]
fn paragraph_offsets_are_bytes_clamped_to_char_boundaries() {
    let (_screen, seen) = mount(|| {
        let seen = Rc::new(RefCell::new(Seen::default()));
        (seen.clone(), column(paragraph_node("aあb", None, seen)))
    });
    let paragraph = seen.borrow().drawn.clone().expect("drawn");
    // 'あ' is bytes 1..4: offsets 2 and 3 are inside it.
    assert_eq!(paragraph.caret(2), paragraph.caret(1));
    assert_eq!(paragraph.caret(3), paragraph.caret(1));
    assert!(paragraph.caret(4).x > paragraph.caret(1).x);
    assert_eq!(paragraph.caret(100), paragraph.caret(5));
    assert_eq!(paragraph.hit(Point::new(500.0, 1.0)), 5);
    assert_eq!(paragraph.rects(2..4), paragraph.rects(1..4));
}

/// A host that lists no shaping-cluster boundaries still gets grapheme
/// stops from `next_boundary`, `prev_boundary` and `hit`.
#[test]
fn a_paragraph_without_cluster_boundaries_stops_at_every_grapheme() {
    let (_screen, seen) = mount(|| {
        let seen = Rc::new(RefCell::new(Seen::default()));
        (seen.clone(), column(paragraph_node("abc", None, seen)))
    });
    let drawn = seen.borrow().drawn.clone().expect("drawn");
    let shaped = ShapedText {
        size: drawn.size(),
        carets: (0..=3)
            .map(|offset| TextCaret {
                offset: offset as u32,
                rect: drawn.caret(offset),
            })
            .collect(),
        cluster_boundaries: Vec::new(),
        baseline: drawn.baseline(),
        lines: drawn.lines().to_vec(),
        ..ShapedText::default()
    };
    let rebuilt = TextParagraph::from_shaped("abc", Arc::new(shaped), None);
    assert_eq!(rebuilt.next_boundary(0), 1);
    assert_eq!(rebuilt.prev_boundary(3), 2);
    let line = rebuilt.lines()[0].rect;
    let at = Point::new(rebuilt.caret(2).x, line.y + line.height / 2.0);
    assert_eq!(rebuilt.hit(at), 2);
}

/// A focusable node whose IME session is `session` and whose IME events go
/// to `events`.
fn session_node(id: &'static str, session: ImeSession, events: Rc<RefCell<Vec<ImeEvent>>>) -> Node {
    node()
        .test_id(id)
        .role(Role::TextBox)
        .name(id)
        .focusable(true)
        .style(style().size(200, 24))
        .ime_session(session)
        .on_ime(move |event| events.borrow_mut().push(event))
}

/// While a node with an IME session has the focus the window allows the
/// IME, places its candidate window at the session's caret and answers its
/// queries from the session; the composition, commits and reconversion
/// ranges come back as `ImeEvent`s, in bytes of the session's text.
#[test]
fn a_node_with_an_ime_session_is_the_ime_target_while_it_has_the_focus() {
    let (screen, (text, selection, caret, events)) = mount(|| {
        let text = signal("héllo".to_string());
        let selection = signal(0..0);
        let caret = signal(Rect::new(10.0, 4.0, 1.0, 16.0));
        let events = Rc::new(RefCell::new(Vec::new()));
        (
            (text, selection, caret, events.clone()),
            column((
                session_node("editor", ImeSession::new(text, selection, caret), events),
                node()
                    .test_id("other")
                    .role(Role::Button)
                    .name("other")
                    .focusable(true)
                    .style(style().size(60, 24)),
            ))
            .style(style().align_items(Align::Start)),
        )
    });
    let editor = screen.get_by_test_id("editor");
    // No focus, no IME.
    assert!(screen.ime_text().is_none());
    assert!(screen.ime_cursor_area().is_none());
    editor.focus();
    let at = editor.rect().expect("laid out");
    let ime = screen.ime_text().expect("the session's text");
    assert_eq!(ime.text, "héllo");
    assert_eq!(ime.selection, 0..0);
    assert_eq!(ime.composition, None);
    let area = screen
        .ime_cursor_area()
        .expect("the candidate window's place");
    assert_eq!((area.x, area.y), (at.x + 10.0, at.y + 4.0));
    assert_eq!(ime.char_rects.last().copied(), Some(area));

    // The composition and the commit arrive as the IME sent them.
    editor.compose("にほ");
    editor.commit("日本");
    assert_eq!(
        *events.borrow(),
        vec![
            ImeEvent::Preedit {
                text: "にほ".into(),
                selection: Some(6..6),
            },
            ImeEvent::Commit("日本".into()),
        ]
    );
    events.borrow_mut().clear();

    // The session follows the node's own state: its text, selection and caret.
    text.set("héllo日本".to_string());
    selection.set(1..3);
    caret.set(Rect::new(50.0, 4.0, 1.0, 16.0));
    screen.run_until_idle();
    let ime = screen.ime_text().expect("the session's text");
    assert_eq!(ime.text, "héllo日本");
    assert_eq!(ime.selection, 1..2, "chars of the text: 'é'");
    let area = screen.ime_cursor_area().expect("moved");
    assert_eq!((area.x, area.y), (at.x + 50.0, at.y + 4.0));

    // A reconversion range is taken in the text the IME was given and
    // reaches the node in bytes ('é' is two).
    editor.reconvert(5..7);
    assert_eq!(*events.borrow(), vec![ImeEvent::Selection(6..12)]);

    // The focus leaving ends it.
    screen.get_by_test_id("other").focus();
    assert!(screen.ime_text().is_none());
    assert!(screen.ime_cursor_area().is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A password session gives the IME no surrounding text, a disabled node is
/// no IME target, and the end of the IME session reaches the node.
#[test]
fn an_ime_session_can_hide_its_text_and_a_disabled_node_has_none() {
    let (screen, (events, disabled)) = mount(|| {
        let events = Rc::new(RefCell::new(Vec::new()));
        let disabled = signal(false);
        (
            (events.clone(), disabled),
            column((
                session_node(
                    "secret",
                    ImeSession::new("hunter2".to_string(), 3..3, Rect::new(8.0, 2.0, 1.0, 16.0))
                        .password(true),
                    events.clone(),
                ),
                session_node(
                    "plain",
                    ImeSession::new("abc".to_string(), 0..0, Rect::new(0.0, 0.0, 1.0, 16.0)),
                    events,
                )
                .disabled(disabled),
            )),
        )
    });
    let secret = screen.get_by_test_id("secret");
    secret.focus();
    let ime = screen.ime_text().expect("the caret's box");
    assert_eq!(ime.text, "");
    assert!(screen.ime_cursor_area().is_some());
    secret.compose("x");
    assert!(matches!(
        events.borrow().last(),
        Some(ImeEvent::Preedit { .. })
    ));
    // The platform ending the session without a commit.
    screen.ime_disabled();
    assert_eq!(events.borrow().last(), Some(&ImeEvent::End));
    // A disabled node takes no IME, and a focus it already has is not one.
    screen.get_by_test_id("plain").focus();
    assert_eq!(screen.ime_text().expect("plain's").text, "abc");
    disabled.set(true);
    screen.run_until_idle();
    // (The focus may move on to the other node, which has a session too.)
    assert!(screen.ime_text().is_none_or(|ime| ime.text != "abc"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Lines of text as hand-made runs, with the selection `(anchor, caret)` in
/// chars of all of it and the caret rectangle at `caret_x`.
fn runs(lines: &[&str], selection: (u32, u32), caret_x: f32) -> SemanticsText {
    let runs: Vec<Arc<SemanticsTextRun>> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let count = line.chars().count() as u32;
            let y = i as f32 * 16.0;
            Arc::new(SemanticsTextRun {
                // Every node numbers its runs from 0: the host keeps them apart.
                id: TextRunId(i as u64),
                value: (*line).to_string(),
                bounds: Rect::new(2.0, 2.0 + y, 8.0 * count as f32, 16.0),
                carets: (0..=count)
                    .map(|c| TextCaret {
                        offset: c,
                        rect: Rect::new(2.0 + 8.0 * c as f32, 2.0 + y, 1.0, 16.0),
                    })
                    .collect(),
            })
        })
        .collect();
    SemanticsText {
        runs: runs.into(),
        start: 0,
        selection,
        caret: Rect::new(caret_x, 2.0, 1.0, 16.0),
    }
}

/// A node publishes its own text for assistive technology: the runs, the
/// selection and the caret (in the node's coordinates, moved into the
/// window's), kept apart from another node's runs numbered the same; a text
/// that cannot be published is reported.
#[test]
fn a_node_publishes_its_own_text_runs() {
    let (screen, shown) = mount(|| {
        let shown = signal(runs(&["abc\n", "de"], (1, 5), 10.0));
        let editor = |id: &'static str| {
            node()
                .test_id(id)
                .role(Role::TextBox)
                .name(id)
                .focusable(true)
                .style(style().size(200, 40))
        };
        (
            shown,
            column((
                editor("first").semantics_text(runs(&["one"], (0, 0), 2.0)),
                editor("second").semantics_text(shown),
            ))
            .style(style().align_items(Align::Start)),
        )
    });
    let (first, second) = (
        screen.get_by_test_id("first"),
        screen.get_by_test_id("second"),
    );
    assert_eq!(first.text_runs(), Some(vec!["one".to_string()]));
    assert_eq!(
        second.text_runs(),
        Some(vec!["abc\n".to_string(), "de".to_string()])
    );
    assert_eq!(second.text_selection(), Some((1, 5)));
    let at = second.rect().expect("laid out");
    let caret = second.text_caret().expect("caret");
    assert_eq!((caret.x, caret.y), (at.x + 10.0, at.y + 2.0));

    // Reactive: a local authored snapshot must not rebuild its static sibling.
    let before = screen.stats();
    shown.set(runs(&["abcd"], (4, 2), 34.0));
    screen.run_until_idle();
    assert_eq!(second.text_runs(), Some(vec!["abcd".to_string()]));
    assert_eq!(second.text_selection(), Some((4, 2)));
    let after = screen.stats();
    assert_eq!(after.a11y_constructed - before.a11y_constructed, 1);
    assert_eq!(after.a11y_published - before.a11y_published, 1);

    // A selection outside the runs cannot be published: reported, and the
    // node publishes no text (the tree stays valid).
    shown.set(runs(&["abcd"], (9, 9), 34.0));
    screen.run_until_idle();
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        *errors[0].kind(),
        kivact::diagnostics::ErrorKind::Unsupported
    );
    assert_eq!(second.text_runs(), None);
    assert_eq!(first.text_runs(), Some(vec!["one".to_string()]));
}

/// A paragraph's text for assistive technology: one run per line, tiling
/// the text, with carets in run-local chars and the selection in chars.
#[test]
fn a_paragraph_makes_the_text_runs_of_its_lines() {
    let (screen, seen) = mount(|| {
        let seen = Rc::new(RefCell::new(Seen::default()));
        (
            seen.clone(),
            column(paragraph_node("one あ two three", Some(50.0), seen).role(Role::TextBox))
                .style(style().align_items(Align::Start)),
        )
    });
    let paragraph = seen.borrow().drawn.clone().expect("drawn");
    let text = paragraph.semantics_text(Point::new(4.0, 2.0), 4, 8);
    let values: Vec<&str> = text.runs.iter().map(|run| run.value.as_str()).collect();
    assert!(values.len() >= 3, "{values:?}");
    assert_eq!(values.concat(), "one あ two three");
    // Chars, not bytes: the anchor is before 'あ' (4), the caret after it.
    assert_eq!(text.selection, (4, 6));
    assert_eq!(text.start, 0);
    for run in text.runs.iter() {
        let count = run.value.chars().count() as u32;
        assert_eq!(run.carets.len() as u32, count + 1);
        assert_eq!(run.carets.last().map(|c| c.offset), Some(count));
        assert!(run.bounds.x >= 4.0 && run.bounds.y >= 2.0);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn paragraphs_shape_in_setup_and_values_with_shared_native_carets() {
    use kivact::primitive::{NodeRefExt, TextEditing, part, shape_paragraph};
    const TEXT: &str = "a\u{301} אבג xyz";
    let (screen, (setup, computed, reference)) = mount(|| {
        let setup = shape_paragraph(TEXT, &style_14(), None);
        let computed = memo(|| shape_paragraph(TEXT, &style_14(), None));
        let editing = part(|state| state)
            .role(Role::TextBox)
            .name("Shared")
            .focusable(true)
            .text_editing(TextEditing::new(TEXT));
        let reference = editing.node_ref();
        (
            (setup, computed, reference),
            node()
                .part(editing)
                .style(style().size(300, 40).font_size(14)),
        )
    });
    let paragraph = computed.get();
    assert_eq!(setup.size(), paragraph.size());
    assert_eq!(setup.next_boundary(0), "a\u{301}".len());
    assert_eq!(setup.prev_boundary(usize::MAX), TEXT.len() - 1);
    assert_eq!(setup.next_boundary(usize::MAX), TEXT.len());
    assert_eq!(setup.prev_boundary(0), 0);
    let layout = reference.text_layout(0..1).expect("native layout");
    let origin = layout.carets[0].rect;
    for (index, (byte, _)) in TEXT
        .char_indices()
        .chain(std::iter::once((TEXT.len(), '\0')))
        .enumerate()
    {
        let native = layout.carets[index].rect;
        let primitive = paragraph.caret(byte);
        assert!(
            (native.x - origin.x - primitive.x).abs() < 0.01,
            "{native:?} {primitive:?}"
        );
        assert!((native.height - primitive.height).abs() < 0.01);
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
