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

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use kivact::control::canvas;
use kivact::control::{Span, rich_text, span};
use kivact::draw::{Point, Rect, Size};
use kivact::draw::{Shadow, linear_gradient};
use kivact::draw::{TextMetrics, TextStyle};
use kivact::prelude::*;
use kivact::primitive::{Constraints, primitive};
use kivact::style::{Align, Color, pct};
use kivact::style::{
    FontKerning, LineHeight, TextAlign, TextDecorationLine, TextOverflow, TextTransform, em,
};
use kivact::test::{RenderOptions, Screen, Screenshot, TestFonts, render, render_with};
use kivact::view::Modifiers;

use crate::paint::{BLACK, BLUE, GREEN, RED, WHITE, at, canvas_of, distance, pixel};
use crate::support::mount;

static INTER: &[u8] = include_bytes!("../../../../../crates/arvane/tests/fonts/InterVariable.ttf");

#[test]
fn available_font_families_include_registered_names_sorted_without_duplicates() {
    static TUFFY: &[u8] = include_bytes!("../../../../../crates/arvane/assets/fonts/Tuffy.ttf");
    let (screen, families) = crate::support::mount_with(
        RenderOptions {
            fonts: TestFonts::Only(vec![Arc::from(INTER), Arc::from(TUFFY), Arc::from(TUFFY)]),
            ..RenderOptions::from_env()
        },
        || {
            let families = kivact::platform::font_families();
            (families, text("font families"))
        },
    );
    assert!(families.iter().any(|name| name == "Tuffy"), "{families:?}");
    assert!(
        families.iter().any(|name| name == "Inter Variable"),
        "{families:?}"
    );
    assert!(
        families.windows(2).all(|names| names[0] < names[1]),
        "{families:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn inter() -> RenderOptions {
    RenderOptions {
        fonts: TestFonts::Only(vec![Arc::from(INTER)]),
        ..RenderOptions::from_env()
    }
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
fn word_spacing_adds_its_space_after_every_space_in_px_or_em() {
    let screen = render(|| {
        texts((
            text("a b c d")
                .style(style().font_size(20))
                .test_id("plain"),
            text("a b c d")
                .style(style().font_size(20).word_spacing(6))
                .test_id("px"),
            text("a b c d")
                .style(style().font_size(20).word_spacing(em(0.3)))
                .test_id("em"),
            rich_text(vec![span("a b "), span("c d").word_spacing(6)])
                .style(style().font_size(20))
                .test_id("span"),
        ))
    });
    let plain = rect(&screen, "plain").width;
    // Three spaces, 6 px each (0.3em of 20 px is 6 px too).
    for id in ["px", "em"] {
        let grown = rect(&screen, id).width - plain;
        assert!((grown - 18.0).abs() <= 1.0, "{id}: grown by {grown}");
    }
    // The span widens the one space inside it.
    let grown = rect(&screen, "span").width - plain;
    assert!((grown - 6.0).abs() <= 1.0, "span: grown by {grown}");
    assert!(screen.errors().is_empty());
}

#[test]
fn font_kerning_none_widens_a_kerned_pair() {
    let screen = render_with(inter(), || {
        texts((
            text("AVAV").style(style().font_size(32)).test_id("auto"),
            text("AVAV")
                .style(style().font_size(32).font_kerning(FontKerning::Normal))
                .test_id("normal"),
            text("AVAV")
                .style(style().font_size(32).font_kerning(FontKerning::None))
                .test_id("none"),
            rich_text(vec![span("AV"), span("AV").font_kerning(FontKerning::None)])
                .style(style().font_size(32))
                .test_id("span"),
        ))
    });
    let auto = rect(&screen, "auto").width;
    let none = rect(&screen, "none").width;
    assert!(none > auto + 2.0, "Inter kerns AV: {none} vs {auto}");
    assert!((rect(&screen, "normal").width - auto).abs() <= 1.0);
    let span = rect(&screen, "span").width;
    assert!(
        span > auto + 1.0 && span < none - 1.0,
        "one pair of two: {auto} < {span} < {none}"
    );
    assert!(screen.errors().is_empty());
}

/// The x of the rightmost pixel near `color` in every line of `r` (lines
/// are the runs of rows holding some, apart from each other).
fn line_rights(shot: &Screenshot, r: Rect, color: u32) -> Vec<u32> {
    let mut lines: Vec<Vec<u32>> = Vec::new();
    for row in rows_of(shot, r, color) {
        match lines.last_mut() {
            Some(line) if line.last().is_some_and(|last| row <= last + 1) => line.push(row),
            _ => lines.push(vec![row]),
        }
    }
    let (x0, x1) = (r.x.max(0.0) as u32, (r.x + r.width) as u32);
    lines
        .iter()
        .map(|rows| {
            (x0..x1)
                .rev()
                .find(|x| {
                    rows.iter()
                        .any(|y| distance(pixel(shot, *x, *y), color) <= 40)
                })
                .expect("a pixel in the line")
        })
        .collect()
}

#[test]
fn justified_text_aligns_every_line_but_the_last_to_the_right_edge() {
    let paragraph =
        "Justified text spreads the spaces of each line so that the lines end at the same edge";
    let screen = render_with(inter(), move || {
        let aligned = |left, align| {
            at(left, 20, 220, 300)
                .font_size(16)
                .line_height(LineHeight::px(30))
                .color(Color::BLACK)
                .text_align(align)
        };
        canvas_of((
            text(paragraph)
                .style(aligned(20, TextAlign::Justify))
                .test_id("justify"),
            text(paragraph)
                .style(aligned(300, TextAlign::Start))
                .test_id("start"),
            rich_text(vec![
                span("Justified text spreads the spaces of "),
                span("each line so that the lines end at the same edge").weight(700),
            ])
            .style(aligned(560, TextAlign::Justify))
            .test_id("rich"),
        ))
    });
    let shot = screen.screenshot();
    for (id, edge) in [("justify", 240u32), ("rich", 780)] {
        let rights = line_rights(&shot, rect(&screen, id), BLACK);
        assert!(rights.len() >= 3, "{id}: several lines: {rights:?}");
        let (last, rest) = rights.split_last().expect("lines");
        for right in rest {
            assert!(
                edge.abs_diff(*right) <= 3,
                "{id}: a line ends at the edge {edge}: {rights:?}"
            );
        }
        assert!(
            *last + 20 < edge,
            "{id}: the last line stays short: {rights:?}"
        );
    }
    // Start-aligned lines are ragged: some end well before the edge.
    let rights = line_rights(&shot, rect(&screen, "start"), BLACK);
    assert!(
        rights.iter().any(|right| right + 3 < 520),
        "start-aligned lines are ragged: {rights:?}"
    );
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

#[test]
fn tabular_figures_and_a_weight_between_the_named_ones() {
    let screen = render_with(inter(), || {
        texts((
            text("1111").style(style().font_size(20)).test_id("ones"),
            text("0000").style(style().font_size(20)).test_id("zeros"),
            text("1111")
                .style(style().font_size(20).tabular_nums())
                .test_id("tabular-ones"),
            text("0000")
                .style(style().font_size(20).font_feature_settings([("tnum", 1)]))
                .test_id("tabular-zeros"),
            text("Weight")
                .style(style().font_size(30).weight(400))
                .test_id("400"),
            text("Weight")
                .style(style().font_size(30).weight(650))
                .test_id("650"),
            text("Weight")
                .style(style().font_size(30).weight(700))
                .test_id("700"),
        ))
    });
    let w = |id: &str| rect(&screen, id).width;
    assert!((w("ones") - w("zeros")).abs() >= 2.0, "proportional digits");
    assert_eq!(w("tabular-ones"), w("tabular-zeros"), "tabular digits");
    assert!(
        w("400") < w("650") && w("650") < w("700"),
        "the wght axis: {} < {} < {}",
        w("400"),
        w("650"),
        w("700")
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn an_invalid_feature_tag_is_dropped_and_reported() {
    let screen = render(|| text("x").style(style().font_feature_settings([("toolong", 1)])));
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].to_string().contains("feature tag"),
        "{}",
        errors[0]
    );
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
    let beyond = Rect::new(right as f32, cut.y, 100.0, cut.height);
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
fn a_line_clamp_is_as_high_as_its_lines() {
    let screen = render(|| {
        texts((
            text(LONG)
                .style(style().width(120).font_size(14).line_clamp(2))
                .test_id("clamped"),
            text(LONG)
                .style(style().width(120).font_size(14))
                .test_id("all"),
            text("x").style(style().font_size(14)).test_id("line"),
        ))
    });
    let line = rect(&screen, "line").height;
    assert_eq!(rect(&screen, "clamped").height, 2.0 * line);
    assert!(rect(&screen, "all").height > 2.0 * line);
    assert_eq!(screen.get_by_test_id("clamped").name(), LONG);
}

#[test]
fn case_conversion_draws_the_converted_text_and_keeps_the_name() {
    let screen = render(|| {
        texts((
            text("label")
                .style(
                    style()
                        .font_size(18)
                        .text_transform(TextTransform::Uppercase),
                )
                .test_id("upper"),
            text("LABEL")
                .style(style().font_size(18))
                .test_id("written"),
        ))
    });
    assert_eq!(rect(&screen, "upper").width, rect(&screen, "written").width);
    assert_eq!(screen.get_by_test_id("upper").name(), "label");
    assert!(screen.get_by_text("label").exists());
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

#[test]
fn a_marker_band_is_a_gradient_behind_the_glyphs() {
    let screen = render(|| {
        canvas_of(
            rich_text([
                span("Hi "),
                span("teacher").background(
                    linear_gradient(180)
                        .stop(Color::TRANSPARENT, pct(60))
                        .stop(rgb(GREEN), pct(60)),
                ),
            ])
            .style(at(20, 20, 300, 40).font_size(20).color(Color::BLACK))
            .test_id("marker"),
        )
    });
    let shot = screen.screenshot();
    let r = rect(&screen, "marker");
    let green = rows_of(&shot, r, GREEN);
    let glyphs = rows_of(&shot, r, BLACK);
    assert!(!green.is_empty());
    // The band's lower 40 %: it starts in the glyphs' lower half.
    let middle = (glyphs[0] + glyphs[glyphs.len() - 1]) / 2;
    assert!(
        green[0] > middle,
        "the lower part only: {green:?}, glyphs {glyphs:?}"
    );
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

#[test]
fn draw_measure_text_agrees_with_text_elements_and_sizes_primitives() {
    let measured: Rc<Cell<TextMetrics>> = Rc::default();
    let out = measured.clone();
    let (screen, ()) = mount(move || {
        let label = TextStyle {
            size: 16.0,
            letter_spacing: em(0.05),
            ..TextStyle::default()
        };
        let chip_style = label.clone();
        let view = texts((
            text("Measure me")
                .style(style().font_size(16).letter_spacing(em(0.05)))
                .test_id("element"),
            canvas(move |d, _| {
                out.set(d.measure_text("Measure me", &label));
                d.text("Measure me", Point::new(0.0, 0.0), label.clone());
            })
            .style(style().size(200, 30)),
            primitive()
                .measure(
                    String::from("Measure me"),
                    move |s: &String, c: Constraints| {
                        let m = c.measure_text(s, &chip_style);
                        Size::new(m.width + 16.0, m.height + 8.0)
                    },
                )
                .test_id("chip"),
        ));
        ((), view)
    });
    let m = measured.get();
    let element = rect(&screen, "element");
    assert!(m.width > 0.0 && m.baseline > 0.0 && m.baseline < m.height);
    assert!(
        (element.width - (m.width.ceil() + 0.5)).abs() <= 0.5,
        "the element's width is the measured one (rounded up): {element:?} vs {m:?}"
    );
    assert!(
        (element.height - (m.height.ceil() + 0.5)).abs() <= 0.5,
        "the element's height is the measured one (rounded up): {element:?} vs {m:?}"
    );
    let chip = rect(&screen, "chip");
    assert!(
        (chip.width - (m.width + 16.0)).abs() <= 1.0,
        "{chip:?} vs {m:?}"
    );
    assert!(
        (chip.height - (m.height + 8.0)).abs() <= 1.0,
        "{chip:?} vs {m:?}"
    );
}

#[test]
fn a_text_field_draws_its_caret_selection_and_placeholder_colors() {
    let screen = render(|| {
        canvas_of(
            kivact::control::node()
                .name("Name")
                .part(
                    kivact::interaction::use_text_edit(kivact::signal(String::new()))
                        .placeholder("Type here")
                        .control(),
                )
                .style(
                    at(20, 20, 200, 32)
                        .font_size(16)
                        .color(Color::BLACK)
                        .placeholder_color(rgb(RED))
                        .caret_color(rgb(BLUE))
                        .selection_color(rgb(GREEN)),
                )
                .test_id("field"),
        )
    });
    let r = rect(&screen, "field");
    assert!(
        count(&screen.screenshot(), r, RED) > 0,
        "the placeholder's red"
    );
    screen.get_by_test_id("field").click();
    screen.type_text("abc");
    let shot = screen.screenshot();
    assert_eq!(count(&shot, r, RED), 0, "no placeholder");
    assert!(count(&shot, r, BLUE) > 0, "the blue caret");
    screen.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    assert!(
        count(&screen.screenshot(), r, GREEN) > 0,
        "the green selection"
    );
}

#[test]
fn a_span_family_that_is_not_installed_is_reported_once() {
    let screen = render(|| {
        column((
            rich_text([span("a"), span("b").font_family("No Such Family")]),
            rich_text([span("c").font_family("No Such Family")]),
        ))
    });
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].to_string().contains("No Such Family"),
        "{}",
        errors[0]
    );
}

#[test]
fn a_line_height_number_is_a_multiple_and_a_px_length_is_its_own() {
    let screen = render(|| {
        texts((
            text("Line")
                .style(style().font_size(20).line_height(2))
                .test_id("scale"),
            text("Line")
                .style(style().font_size(20).line_height(LineHeight::px(40)))
                .test_id("px"),
            text("Line")
                .style(style().font_size(20).line_height(LineHeight::Normal))
                .test_id("normal"),
            text("Line").style(style().font_size(20)).test_id("default"),
        ))
    });
    let (scale, px) = (rect(&screen, "scale").height, rect(&screen, "px").height);
    assert!((scale - 40.0).abs() <= 0.5, "twice the size: {scale}");
    assert_eq!(scale, px, "2 times 20 px is 40 px");
    // An explicit `Normal` is a value: the font size times 1.3, rounded,
    // not the theme's.
    let normal = rect(&screen, "normal").height;
    assert!((normal - 26.0).abs() <= 0.5, "normal: {normal}");
    // Text that sets none inherits the theme's (a multiple of 1.35).
    let unset = rect(&screen, "default").height;
    let themed = 20.0 * 1.35;
    assert!((unset - themed).abs() <= 0.5, "unset: {unset}");
    assert!(screen.errors().is_empty());
}

/// The three texts of the theme test: bare, with another text property
/// only, and in a container that sets a line height of its own.
fn theme_line_heights(line_height: LineHeight) -> [f32; 4] {
    let theme = Theme::light();
    let typography = kivact::style::theme::Typography {
        line_height,
        ..theme.text().clone()
    };
    let theme = theme.with_text(typography);
    let screen = render_with(
        RenderOptions {
            theme,
            ..RenderOptions::from_env()
        },
        || {
            texts((
                text("Line").test_id("bare"),
                text("Line")
                    .style(style().color(Color::BLACK))
                    .test_id("coloured"),
                text("Line")
                    .style(style().line_height(3))
                    .test_id("explicit"),
                column(
                    text("Line")
                        .style(style().color(Color::BLACK))
                        .test_id("inherited"),
                )
                .style(style().line_height(LineHeight::px(30))),
            ))
        },
    );
    assert!(screen.errors().is_empty());
    ["bare", "coloured", "explicit", "inherited"].map(|id| rect(&screen, id).height)
}

#[test]
fn the_themes_line_height_reaches_every_text_and_a_style_wins() {
    let size = Theme::light().text().size;
    for scale in [1.0, 2.0] {
        let [bare, coloured, explicit, inherited] = theme_line_heights(LineHeight::Scale(scale));
        // Text that sets none, also one that sets another text property.
        assert!((bare - size * scale).abs() <= 0.5, "bare: {bare}");
        assert_eq!(bare, coloured, "another text property keeps the theme's");
        // A style's own value, and a container's reaching its texts, win.
        assert!((explicit - size * 3.0).abs() <= 0.5, "explicit: {explicit}");
        assert!((inherited - 30.0).abs() <= 0.5, "inherited: {inherited}");
    }
    // A px line height of the theme is the same for any size.
    let [bare, ..] = theme_line_heights(LineHeight::px(25));
    assert!((bare - 25.0).abs() <= 0.5, "px: {bare}");
}

#[test]
fn draw_text_takes_the_inherited_line_height_unless_it_sets_one() {
    let measured: Rc<Cell<[f32; 3]>> = Rc::default();
    let out = measured.clone();
    let (screen, ()) = mount(move || {
        let view = column((
            text("Line").style(style().font_size(20)).test_id("element"),
            canvas(move |d, _| {
                let base = TextStyle {
                    size: 20.0,
                    ..TextStyle::default()
                };
                let with = |line_height| TextStyle {
                    line_height: Some(line_height),
                    ..base.clone()
                };
                out.set([
                    d.measure_text("Line", &base).height,
                    d.measure_text("Line", &with(LineHeight::px(40))).height,
                    d.measure_text("Line", &with(LineHeight::Normal)).height,
                ]);
            })
            .style(style().size(200, 30)),
        ));
        ((), view)
    });
    let [inherited, px, normal] = measured.get();
    // Unset: what a text element gets (the theme's), not the shaper's.
    assert_eq!(inherited, rect(&screen, "element").height);
    assert!((px - 40.0).abs() <= 0.5, "px: {px}");
    assert!((normal - 26.0).abs() <= 0.5, "normal: {normal}");
}

#[test]
fn draw_text_takes_word_spacing_and_kerning() {
    let measured: Rc<Cell<[f32; 6]>> = Rc::default();
    let out = measured.clone();
    let (screen, ()) = mount_with_inter(move || {
        let view = texts((
            text("a b c")
                .style(style().font_size(20).word_spacing(em(0.5)))
                .test_id("element"),
            canvas(move |d, _| {
                let base = TextStyle {
                    size: 20.0,
                    ..TextStyle::default()
                };
                let spaced = TextStyle {
                    word_spacing: em(0.5),
                    ..base.clone()
                };
                let kerned = |font_kerning| TextStyle {
                    size: 32.0,
                    font_kerning,
                    ..TextStyle::default()
                };
                let justify = TextStyle {
                    align: TextAlign::Justify,
                    ..base.clone()
                };
                out.set([
                    d.measure_text("a b c", &base).width,
                    d.measure_text("a b c", &spaced).width,
                    d.measure_text("AV", &kerned(FontKerning::Auto)).width,
                    d.measure_text("AV", &kerned(FontKerning::None)).width,
                    d.measure_text("a b c", &justify).width,
                    d.measure_text("AV", &kerned(FontKerning::Normal)).width,
                ]);
            })
            .style(style().size(200, 30)),
        ));
        ((), view)
    });
    let [plain, spaced, auto, none, justify, normal] = measured.get();
    // Two spaces, 0.5em of 20 px each; a text element agrees.
    assert!((spaced - plain - 20.0).abs() <= 0.5, "{plain} vs {spaced}");
    assert!((rect(&screen, "element").width - (spaced.ceil() + 0.5)).abs() <= 0.5);
    assert!(none > auto + 1.0, "Inter kerns AV: {none} vs {auto}");
    assert!((normal - auto).abs() <= 0.01);
    // A line drawn alone is the last: it measures as it does at the start.
    assert_eq!(justify, plain);
    assert!(screen.errors().is_empty());
}

/// A theme whose text is `color` (red or blue, to count on the frame).
fn text_theme(color: u32) -> kivact::style::Theme {
    let theme = kivact::style::Theme::light();
    let colors = kivact::style::theme::Palette {
        text: kivact::style::rgb(color),
        surface: Color::WHITE,
        ..theme.colors().clone()
    };
    theme.with_colors(colors)
}

/// Each text property is inherited on its own, as in CSS: a text that sets
/// only a weight keeps its parent's color, size and letter spacing, and
/// one under no colored ancestor takes the App theme's text color.
#[test]
fn a_text_that_sets_only_a_weight_keeps_each_other_property_of_its_parent() {
    let (screen, ()) = crate::support::mount_with(
        RenderOptions {
            theme: text_theme(BLUE),
            ..RenderOptions::from_env()
        },
        || {
            let view = texts((
                column(text("Weight").style(style().weight(700)).test_id("child")).style(
                    style()
                        .color(kivact::style::rgb(RED))
                        .font_size(24)
                        .letter_spacing(3),
                ),
                text("Weight")
                    .style(
                        style()
                            .weight(700)
                            .color(kivact::style::rgb(RED))
                            .font_size(24)
                            .letter_spacing(3),
                    )
                    .test_id("own"),
                text("Weight")
                    .style(style().weight(700).font_size(24))
                    .test_id("theme"),
            ));
            ((), view)
        },
    );
    let (child, own, theme) = (
        rect(&screen, "child"),
        rect(&screen, "own"),
        rect(&screen, "theme"),
    );
    // The same size (the height rounds by the place, 0.4 px apart).
    assert_eq!(child.width, own.width);
    assert!(
        (child.height - own.height).abs() <= 1.0,
        "{child:?} {own:?}"
    );
    let shot = screen.screenshot();
    let (red, blue) = (count(&shot, child, RED), count(&shot, child, BLUE));
    assert!(red > 30 && blue == 0, "child: {red} red, {blue} blue");
    let (red, blue) = (count(&shot, theme, RED), count(&shot, theme, BLUE));
    assert!(blue > 30 && red == 0, "theme: {red} red, {blue} blue");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A text that sets only a weight follows its parent's color through a
/// theme switch: the parent takes the theme's text color, and the text
/// inherits it again.
#[test]
fn a_text_that_sets_only_a_weight_follows_a_theme_switch() {
    let (screen, blue) = crate::support::mount(|| {
        let blue = signal(false);
        let view = kivact::view::provide_theme(
            move || text_theme(if blue.get() { BLUE } else { RED }),
            || {
                column(
                    text("Theme")
                        .style(style().weight(700).font_size(24))
                        .test_id("label"),
                )
                .style(
                    style()
                        .align_items(Align::Start)
                        .background(kivact::style::ColorToken::Surface)
                        .color(kivact::style::ColorToken::Text),
                )
            },
        );
        (blue, view)
    });
    let label = rect(&screen, "label");
    let colors = |screen: &Screen| {
        let shot = screen.screenshot();
        (count(&shot, label, RED), count(&shot, label, BLUE))
    };
    let (red, blue_px) = colors(&screen);
    assert!(red > 30 && blue_px == 0, "light: {red} red, {blue_px} blue");
    blue.set(true);
    screen.run_until_idle();
    let (red, blue_px) = colors(&screen);
    assert!(
        blue_px > 30 && red == 0,
        "switched: {red} red, {blue_px} blue"
    );
    assert_eq!(rect(&screen, "label"), label);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A state transition captures the text color it shows when its paint is
/// written, and the ancestor's text style of the same revision is written
/// before it: the capture reads the new inherited color, not the one the
/// engine resolved for the node before (so nothing fades from the old
/// color).
#[test]
fn a_state_capture_sees_the_text_style_written_before_it_in_one_revision() {
    use kivact::motion::{Easing, Transition};
    use kivact::style::TransitionProp;
    let (screen, flip) = crate::support::mount(|| {
        let flip = signal(false);
        let view = column(
            node()
                .child(
                    text("Captured")
                        .style(style().font_size(24))
                        .test_id("label"),
                )
                .style(move || {
                    style()
                        .size(200, 60)
                        .background(if flip.get() {
                            kivact::style::rgb(GREEN)
                        } else {
                            Color::WHITE
                        })
                        .hover(style().color(kivact::style::rgb(RED)))
                        .transition(
                            TransitionProp::Paint,
                            Transition::tween(400, Easing::Linear),
                        )
                }),
        )
        .style(move || {
            style()
                .align_items(Align::Start)
                .background(Color::WHITE)
                .color(kivact::style::rgb(if flip.get() { BLUE } else { BLACK }))
        });
        (flip, view)
    });
    let label = rect(&screen, "label");
    let shot = screen.screenshot();
    assert!(count(&shot, label, BLACK) > 30, "starts black");
    flip.set(true);
    screen.run_until_idle();
    let shot = screen.screenshot();
    let (black, blue) = (count(&shot, label, BLACK), count(&shot, label, BLUE));
    assert!(blue > 30 && black == 0, "{black} black, {blue} blue");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
