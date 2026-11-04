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

use crate::support::mount;
use kivact::draw::{InlineBox, InlineParagraph, Point, Rect, Size, TextParagraph, TextStyle};
use kivact::prelude::*;
use kivact::primitive::{Constraints, Primitive, primitive};
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

#[test]
fn measured_inline_boxes_wrap_and_share_the_text_baseline_in_draw() {
    type Pair = (Option<InlineParagraph>, Option<InlineParagraph>);
    let (screen, seen) = mount(|| {
        let seen = Rc::new(RefCell::new(Pair::default()));
        let measure = seen.clone();
        let draw = seen.clone();
        let item = InlineBox {
            id: 42,
            offset: 6,
            size: Size::new(60.0, 32.0),
        };
        let text = "First word after";
        let width = Some(68.0);
        let view = primitive()
            .measure(text.to_owned(), move |text: &String, c: Constraints| {
                let result = c
                    .measure_inline_paragraph(text, &style_14(), width, &[item])
                    .expect("host supports inline paragraphs");
                let size = result.paragraph().size();
                measure.borrow_mut().0 = Some(result);
                size
            })
            .draw(move |d, _| {
                let result = d
                    .layout_inline_paragraph(text, &style_14(), width, &[item])
                    .expect("same inline layout during drawing");
                d.paragraph(result.paragraph(), Point::default(), ColorToken::Text);
                for placement in result.boxes() {
                    d.fill_rect(placement.rect, ColorToken::Accent);
                }
                draw.borrow_mut().1 = Some(result);
            });
        (seen, column(view).style(style().align_items(Align::Start)))
    });
    let seen = seen.borrow();
    let measured = seen.0.as_ref().expect("measured");
    let drawn = seen.1.as_ref().expect("drawn");
    assert_eq!(measured.boxes(), drawn.boxes());
    assert_eq!(measured.paragraph().lines(), drawn.paragraph().lines());
    let item = measured.boxes()[0];
    assert_eq!(item.id, 42);
    assert!(item.line > 0, "the 60 px object cannot fit after First");
    let line = &measured.paragraph().lines()[item.line];
    assert!((item.rect.bottom() - line.baseline).abs() < 0.01);
    assert!(line.rect.height >= 32.0);
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

/// Atomic controls stay mounted and occupy the shaped line after a hard break.
#[test]
fn retained_inline_control_receives_pointer_input() {
    let (screen, pressed) = mount(|| {
        let pressed = Rc::new(std::cell::Cell::new(false));
        let flag = Rc::clone(&pressed);
        let item = node()
            .role(Role::Button)
            .name("Inline action")
            .focusable(true)
            .style(style().size(32, 18))
            .on_press(move || flag.set(true));
        (
            pressed,
            column(kivact::primitive::inline_view(
                "First line\n ",
                style_14(),
                [(11, item.into_view())],
            ))
            .style(style().width(150))
            .test_id("paragraph-container"),
        )
    });
    let container = screen.get_by_test_id("paragraph-container").rect().unwrap();
    let action = screen.get_by_role(Role::Button, "Inline action");
    assert!(action.rect().unwrap().y > container.y + 8.0);
    action.click();
    assert!(pressed.get());
}
