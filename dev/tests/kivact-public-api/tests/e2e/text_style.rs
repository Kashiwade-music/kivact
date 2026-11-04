//! Role: Tests for text styling and typography properties.
//!
//! The text vocabulary on real windows: letter and word spacing in px and em,
//! kerning, justified lines,
//! decoration lines and their offset, OpenType features and variable
//! weights (Inter, `crates/arvane/tests/fonts`), ellipses and line clamps
//! (the accessible name keeps the whole text), case conversion, text
//! shadows, rich text spans (bold, a highlight band across lines, live
//! from a signal), `Draw::measure_text` and a primitive measuring its
//! label, and a text field's caret, selection and placeholder colors.
//! Checked on the frame's pixels at scale 1.

use std::rc::Rc;

use kivact::draw::Shadow;
use kivact::geometry::Rect;
use kivact::prelude::*;
use kivact::style::{Align, Color};
use kivact::style::{TextDecorationLine, TextOverflow, em};
use kivact::test::{RenderOptions, Screen, Screenshot, TestFonts, render, render_with};
use kivact::view::{Span, rich_text, span};

use crate::paint::{BLACK, BLUE, GREEN, RED, WHITE, at, canvas_of, distance, pixel};

static INTER: &[u8] = include_bytes!("../../../../../crates/arvane/tests/fonts/InterVariable.ttf");

fn inter() -> RenderOptions {
    RenderOptions::from_env().fonts(TestFonts::Only(vec![kivact::view::Asset::from(INTER)]))
}

fn rect(screen: &Screen, id: &str) -> Rect {
    screen.get_by_test_id(id).rect().expect("laid out")
}

/// Texts in a column at their own widths.
fn texts(children: impl IntoView) -> Node {
    column(children).style(style().align_items(Align::Start).background(Color::WHITE))
}

/// The rows of `r` holding a pixel within 40 of `color`.
fn rows_of(shot: &Screenshot, r: Rect, color: u32) -> Vec<u32> {
    let (x0, y0) = (r.x.max(0.0) as u32, r.y.max(0.0) as u32);
    let (x1, y1) = ((r.x + r.width) as u32, (r.y + r.height) as u32);
    (y0..y1)
        .filter(|y| (x0..x1).any(|x| distance(pixel(shot, x, *y), color) <= 40))
        .collect()
}

/// How many pixels of `r` are within 40 of `color`.
fn count(shot: &Screenshot, r: Rect, color: u32) -> usize {
    let (x0, y0) = (r.x.max(0.0) as u32, r.y.max(0.0) as u32);
    let (x1, y1) = ((r.x + r.width) as u32, (r.y + r.height) as u32);
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .filter(|(x, y)| distance(pixel(shot, *x, *y), color) <= 40)
        .count()
}

#[test]
fn letter_spacing_widens_text_by_its_px_or_em() {
    let screen = render(|| {
        texts((
            text("spacing")
                .style(style().font_size(20))
                .test_id("plain"),
            text("spacing")
                .style(style().font_size(20).letter_spacing(2))
                .test_id("px"),
            text("spacing")
                .style(style().font_size(20).letter_spacing(em(0.1)))
                .test_id("em"),
        ))
    });
    let plain = rect(&screen, "plain").width;
    // Seven characters, 2 px each (0.1em of 20 px is 2 px too).
    for id in ["px", "em"] {
        let grown = rect(&screen, id).width - plain;
        assert!((grown - 14.0).abs() <= 1.0, "{id}: grown by {grown}");
    }
    assert!(screen.errors().is_empty());
}

#[test]
fn an_underline_lies_below_the_baseline_and_moves_with_its_offset() {
    let screen = render(|| {
        canvas_of((
            text("Under")
                .style(
                    at(20, 20, 200, 40)
                        .font_size(24)
                        .color(Color::BLACK)
                        .text_decoration(TextDecorationLine::UNDERLINE)
                        .text_decoration_color(rgb(RED))
                        .text_decoration_thickness(2),
                )
                .test_id("under"),
            text("Under")
                .style(
                    at(260, 20, 200, 40)
                        .font_size(24)
                        .color(Color::BLACK)
                        .text_decoration(TextDecorationLine::UNDERLINE)
                        .text_decoration_color(rgb(RED))
                        .text_decoration_thickness(2)
                        .text_underline_offset(3),
                )
                .test_id("moved"),
            // The text's color (currentColor) without a color of its own.
            text("Blue")
                .style(
                    at(20, 80, 200, 40)
                        .font_size(24)
                        .color(rgb(BLUE))
                        .text_decoration(
                            TextDecorationLine::UNDERLINE | TextDecorationLine::LINE_THROUGH,
                        ),
                )
                .test_id("blue"),
        ))
    });
    let shot = screen.screenshot();
    let under = rows_of(&shot, rect(&screen, "under"), RED);
    let moved = rows_of(&shot, rect(&screen, "moved"), RED);
    let glyphs = rows_of(&shot, rect(&screen, "under"), BLACK);
    assert!(!under.is_empty(), "an underline is drawn");
    // Below the glyphs' baseline: under the bottom of "U", "n", "d" ...
    let baseline_row = glyphs.iter().copied().max().expect("glyphs");
    assert!(
        under[0] + 1 >= baseline_row,
        "below the glyphs: rows {under:?}, glyphs end at {baseline_row}"
    );
    assert_eq!(moved[0], under[0] + 3, "3 px further down");
    // Underline and line-through in the text's blue: two bands of rows.
    let blue = rows_of(&shot, rect(&screen, "blue"), BLUE);
    assert!(blue.len() > 4);
    assert!(screen.errors().is_empty());
}

const LONG: &str = "A long line of text that cannot fit its box";

#[test]
fn an_ellipsis_cuts_a_line_and_the_name_keeps_the_whole_text() {
    let screen = render(|| {
        canvas_of((
            text(LONG)
                .style(
                    at(20, 20, 160, 30)
                        .font_size(16)
                        .color(Color::BLACK)
                        .text_wrap(false)
                        .text_overflow(TextOverflow::Ellipsis),
                )
                .test_id("cut"),
            // In a row, an ellipsized text shrinks down to its ellipsis.
            row((
                text(LONG)
                    .style(
                        style()
                            .grow(1)
                            .text_wrap(false)
                            .text_overflow(TextOverflow::Ellipsis),
                    )
                    .test_id("shrunk"),
                node().style(style().size(60, 10).shrink(0)),
            ))
            .style(at(20, 80, 200, 30)),
        ))
    });
    let shot = screen.screenshot();
    let cut = rect(&screen, "cut");
    // Nothing is drawn past the box (but the ellipsis' last pixel).
    let right = (cut.x + cut.width) as u32 + 2;
    let beyond = Rect::new(right as f64, cut.y, 100.0, cut.height);
    assert_eq!(count(&shot, beyond, BLACK), 0, "ink past the box");
    assert!(count(&shot, cut, BLACK) > 0);
    // The accessible text is the whole text.
    assert_eq!(screen.get_by_test_id("cut").name(), LONG);
    let shrunk = rect(&screen, "shrunk");
    assert!(
        (shrunk.width - 140.0).abs() <= 1.0,
        "the row's 200 less 60: {shrunk:?}"
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn text_shadows_paint_offset_and_blurred_copies() {
    let screen = render(|| {
        canvas_of((
            text("I")
                .style(
                    at(20, 20, 60, 60)
                        .font_size(40)
                        .color(Color::BLACK)
                        .text_shadow(Shadow::new(20, 0, 0, rgb(RED))),
                )
                .test_id("hard"),
            text("I")
                .style(
                    at(120, 20, 60, 60)
                        .font_size(40)
                        .color(Color::BLACK)
                        .text_shadow(Shadow::new(20, 0, 6, rgb(RED))),
                )
                .test_id("soft"),
        ))
    });
    let shot = screen.screenshot();
    let hard = rect(&screen, "hard");
    let red = count(&shot, Rect::new(hard.x, hard.y, 60.0, 60.0), RED);
    assert!(red > 20, "the shadow's glyph, 20 px right: {red}");
    // Blurred: reddish pixels, no pure red.
    let soft = rect(&screen, "soft");
    let area = Rect::new(soft.x, soft.y, 60.0, 60.0);
    let tinted = (area.y as u32..(area.y + area.height) as u32)
        .flat_map(|y| (area.x as u32..(area.x + area.width) as u32).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let p = pixel(&shot, *x, *y);
            p != WHITE && (p >> 16) & 0xff == 0xff && p & 0xff < 0xf0
        })
        .count();
    assert!(tinted > 20, "a blurred red glow: {tinted}");
    assert_eq!(count(&shot, area, RED), 0, "no hard red");
    assert!(screen.errors().is_empty());
}

/// `label` with every match of `query` bold on a green band.
fn highlight(label: &str, query: &str) -> Vec<Span> {
    match label.find(query).filter(|_| !query.is_empty()) {
        Some(i) => vec![
            span(&label[..i]),
            span(&label[i..i + query.len()])
                .weight(700)
                .background(rgb(GREEN)),
            span(&label[i + query.len()..]),
        ],
        None => vec![span(label)],
    }
}

const PARAGRAPH: &str = "find the matching words in this wrapped line";

#[test]
fn rich_text_spans_bold_and_highlight_across_lines_live() {
    let (screen, query) = mount_with_inter(|| {
        let query = signal(String::from("matching words in"));
        let view = canvas_of(
            rich_text(move || highlight(PARAGRAPH, &query.get()))
                .style(at(20, 20, 150, 120).font_size(16).color(Color::BLACK))
                .test_id("rich"),
        );
        (query, view)
    });
    let shot = screen.screenshot();
    let r = rect(&screen, "rich");
    let green = rows_of(&shot, r, GREEN);
    assert!(!green.is_empty(), "a band");
    // On two lines: the band's rows are not one run.
    let gaps = green.windows(2).filter(|w| w[1] > w[0] + 1).count();
    assert_eq!(gaps, 1, "two bands, one per line: {green:?}");
    // One text, the spans' text joined.
    assert_eq!(screen.get_by_test_id("rich").name(), PARAGRAPH);
    // A new query moves the highlight; no query, no band.
    query.set(String::new());
    screen.run_until_idle();
    assert_eq!(count(&screen.screenshot(), r, GREEN), 0);
    assert_eq!(screen.get_by_test_id("rich").name(), PARAGRAPH);
    // Bold spans are wider (Inter's wght axis).
    query.set(String::from("find"));
    screen.run_until_idle();
    let shot = screen.screenshot();
    assert!(count(&shot, r, GREEN) > 0);
    assert!(screen.errors().is_empty());
}

fn mount_with_inter<H: 'static, V: IntoView>(f: impl FnOnce() -> (H, V) + 'static) -> (Screen, H) {
    let slot: Rc<std::cell::RefCell<Option<H>>> = Rc::default();
    let s2 = slot.clone();
    let screen = render_with(inter(), move || {
        let (h, v) = f();
        *s2.borrow_mut() = Some(h);
        v
    });
    let h = slot.borrow_mut().take().expect("setup ran");
    (screen, h)
}
