//! Role: Text style contract tests.
//!
//! The text vocabulary at the shaping level: letter and word spacing,
//! kerning, justified lines, OpenType
//! features and variable weights, decorations, ellipses and line clamps,
//! case conversion, and rich text spans (their glyphs, colors, bands and
//! decorations, and fallback per glyph within a span), measured exactly
//! as they are shaped.
//!
//! Features and weights use Inter (`tests/fonts/InterVariable.ttf`, SIL
//! Open Font License, from the Inter project): a variable font with a
//! `wght` axis, proportional default digits and `tnum`.

use std::sync::Arc;

use arvane::{Engine, ErrorKind};
use ui_types::{
    DecorationLines, FontFeature, FontKerning, Rgba, RichText, ShapedText, SpanStyle,
    TextDecoration, TextOverflow, TextSpan, TextStyle, TextTransform, TextWrap,
};

use crate::support::engine;

static INTER: &[u8] = include_bytes!("../fonts/InterVariable.ttf");

fn inter() -> Engine {
    Engine::with_fonts_only(&[Arc::from(INTER)]).expect("Inter is usable")
}

fn style(font_size: f32) -> TextStyle {
    TextStyle {
        font_size,
        ..TextStyle::default()
    }
}

const RED: Rgba = Rgba {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

/// Measuring gives what shaping draws (glyphs and carets alike).
#[track_caller]
fn shaped<'t>(
    engine: &mut Engine,
    text: impl Into<RichText<'t>>,
    style: &TextStyle,
    width: Option<f32>,
) -> Arc<ShapedText> {
    let text = text.into();
    let measured = engine.measure_text(text, style, width).expect("measure");
    let glyphs = engine.shape_glyphs(text, style, width).expect("shape");
    let shaped = engine.shape(text, style, width).expect("shape");
    assert_eq!(measured, glyphs.size, "measured as drawn: {:?}", text.text);
    assert_eq!(shaped.size, glyphs.size);
    assert_eq!(shaped.runs, glyphs.runs);
    glyphs
}

fn glyph_count(shaped: &ShapedText) -> usize {
    shaped.runs.iter().map(|r| r.glyphs.len()).sum()
}

#[test]
fn letter_spacing_adds_its_space_after_every_character() {
    let mut engine = engine();
    let plain = shaped(&mut engine, "spacing", &style(16.0), None);
    let spaced = TextStyle {
        letter_spacing: 3.0,
        ..style(16.0)
    };
    let wide = shaped(&mut engine, "spacing", &spaced, None);
    let grown = wide.size.width - plain.size.width;
    assert!(
        (grown - 7.0 * 3.0).abs() < 0.1,
        "seven characters, 3 px each: grown by {grown}"
    );
    // A negative spacing tightens.
    let tight = TextStyle {
        letter_spacing: -1.0,
        ..style(16.0)
    };
    assert!(shaped(&mut engine, "spacing", &tight, None).size.width < plain.size.width);
}

#[test]
fn word_spacing_adds_its_space_after_every_space() {
    let mut engine = engine();
    let text = "one two  three";
    let plain = shaped(&mut engine, text, &style(16.0), None);
    let spaced = TextStyle {
        word_spacing: 5.0,
        ..style(16.0)
    };
    let wide = shaped(&mut engine, text, &spaced, None);
    let grown = wide.size.width - plain.size.width;
    assert!(
        (grown - 3.0 * 5.0).abs() < 0.1,
        "three spaces, 5 px each: grown by {grown}"
    );
    // A span sets its own; the paragraph's stays elsewhere.
    let spans = [TextSpan {
        range: 0..8,
        style: SpanStyle {
            word_spacing: Some(5.0),
            ..SpanStyle::default()
        },
    }];
    let part = shaped(
        &mut engine,
        RichText {
            text,
            spans: &spans,
        },
        &style(16.0),
        None,
    );
    let grown = part.size.width - plain.size.width;
    assert!(
        (grown - 2.0 * 5.0).abs() < 0.1,
        "two spaces in the span: grown by {grown}"
    );
    // A negative spacing tightens.
    let tight = TextStyle {
        word_spacing: -2.0,
        ..style(16.0)
    };
    assert!(shaped(&mut engine, text, &tight, None).size.width < plain.size.width);
}

#[test]
fn turning_kerning_off_widens_a_kerned_pair() {
    let mut engine = inter();
    let width = |engine: &mut Engine, kerning: FontKerning| {
        shaped(
            engine,
            "AV",
            &TextStyle {
                kerning,
                ..style(32.0)
            },
            None,
        )
        .size
        .width
    };
    let auto = width(&mut engine, FontKerning::Auto);
    let normal = width(&mut engine, FontKerning::Normal);
    let none = width(&mut engine, FontKerning::None);
    assert!(
        none > auto + 1.0,
        "Inter kerns AV: {none} without, {auto} with"
    );
    assert!(
        (auto - normal).abs() < 0.01,
        "auto kerns: {auto} vs {normal}"
    );
    // A span turns it off for its range only.
    let spans = [TextSpan {
        range: 0..2,
        style: SpanStyle {
            kerning: Some(FontKerning::None),
            ..SpanStyle::default()
        },
    }];
    let span = shaped(
        &mut engine,
        RichText {
            text: "AV",
            spans: &spans,
        },
        &style(32.0),
        None,
    );
    assert!((span.size.width - none).abs() < 0.01);
    // `font-feature-settings` wins over `font-kerning`.
    let feature = shaped(
        &mut engine,
        "AV",
        &TextStyle {
            kerning: FontKerning::None,
            features: vec![FontFeature::new(*b"kern", 1)],
            ..style(32.0)
        },
        None,
    )
    .size
    .width;
    assert!((feature - auto).abs() < 0.01, "{feature} vs {auto}");
}

#[test]
fn justified_lines_span_the_width_except_the_last() {
    let mut engine = engine();
    let text = "Justified text fills each line of the paragraph to its width but the last one";
    let justified = TextStyle {
        align: ui_types::TextAlign::Justify,
        ..style(14.0)
    };
    let width = 180.0;
    let shaped_text = shaped(&mut engine, text, &justified, Some(width));
    let lines = &shaped_text.lines;
    assert!(lines.len() >= 3, "wraps: {}", lines.len());
    let (last, rest) = lines.split_last().expect("lines");
    for line in rest {
        assert!(
            line.rect.x.abs() < 0.01 && (line.rect.x + line.rect.width - width).abs() < 0.5,
            "a line spans the width: {:?}",
            line.rect
        );
    }
    let ragged = shaped(&mut engine, text, &style(14.0), Some(width));
    assert!(
        rest.iter()
            .zip(&ragged.lines)
            .any(|(j, r)| j.rect.width > r.rect.width + 1.0),
        "spaces widened"
    );
    assert!(last.rect.x.abs() < 0.01, "the last starts at the start");
    assert!(
        (last.rect.width - ragged.lines.last().expect("line").rect.width).abs() < 0.01,
        "and keeps its natural width"
    );
    // Again at another width: justification does not stick to the layout.
    let narrower = shaped(&mut engine, text, &justified, Some(150.0));
    for line in &narrower.lines[..narrower.lines.len() - 1] {
        assert!((line.rect.x + line.rect.width - 150.0).abs() < 0.5);
    }
    let start = shaped(&mut engine, text, &style(14.0), Some(width));
    assert_eq!(start.runs, ragged.runs, "start alignment unchanged");
    // Without a width there is one line: the last.
    let alone = shaped(&mut engine, text, &justified, None);
    assert_eq!(alone.lines.len(), 1);
    assert_eq!(
        alone.size,
        shaped(&mut engine, text, &style(14.0), None).size
    );
}

#[test]
fn tabular_figures_give_every_digit_one_width() {
    let mut engine = inter();
    let ones = shaped(&mut engine, "1111", &style(20.0), None).size.width;
    let zeros = shaped(&mut engine, "0000", &style(20.0), None).size.width;
    assert!(
        (ones - zeros).abs() > 1.0,
        "Inter's default digits are proportional: {ones} vs {zeros}"
    );
    let tabular = TextStyle {
        features: vec![FontFeature::new(*b"tnum", 1)],
        ..style(20.0)
    };
    let ones = shaped(&mut engine, "1111", &tabular, None).size.width;
    let zeros = shaped(&mut engine, "0000", &tabular, None).size.width;
    assert!(
        (ones - zeros).abs() < 0.01,
        "tnum: equal widths, {ones} vs {zeros}"
    );
    // Another feature changes glyphs, not the text: `ss01` (open digits).
    let plain = shaped(&mut engine, "1469", &style(20.0), None);
    let alternate = shaped(
        &mut engine,
        "1469",
        &TextStyle {
            features: vec![FontFeature::new(*b"ss01", 1)],
            ..style(20.0)
        },
        None,
    );
    let ids = |s: &ShapedText| -> Vec<u32> {
        s.runs
            .iter()
            .flat_map(|r| r.glyphs.iter().map(|g| g.id))
            .collect()
    };
    assert_ne!(ids(&plain), ids(&alternate), "ss01 picks other glyphs");
}

#[test]
fn a_weight_between_the_named_ones_selects_the_variable_axis() {
    let mut engine = inter();
    let at = |engine: &mut Engine, weight: u16| {
        let s = shaped(
            engine,
            "Weight",
            &TextStyle {
                weight,
                ..style(24.0)
            },
            None,
        );
        let coords = s.runs[0].normalized_coords.clone();
        (s.size.width, coords)
    };
    let (regular, regular_coords) = at(&mut engine, 400);
    let (between, between_coords) = at(&mut engine, 650);
    let (bold, bold_coords) = at(&mut engine, 700);
    assert!(
        regular < between && between < bold,
        "wider as heavier: {regular} < {between} < {bold}"
    );
    assert!(!between_coords.is_empty(), "the wght axis is set");
    assert_ne!(between_coords, regular_coords);
    assert_ne!(between_coords, bold_coords);
}

#[test]
fn decorations_lie_under_over_and_through_the_glyphs() {
    let mut engine = engine();
    let decorated = |lines, offset| TextStyle {
        decoration: TextDecoration {
            lines,
            underline_offset: offset,
            ..TextDecoration::default()
        },
        ..style(20.0)
    };
    let under = shaped(
        &mut engine,
        "Under",
        &decorated(DecorationLines::UNDERLINE, 0.0),
        None,
    );
    let baseline = under.baseline;
    assert!(baseline > 0.0 && baseline < under.size.height);
    let [line] = under.decorations[..] else {
        panic!("one underline: {:?}", under.decorations);
    };
    assert_eq!(line.line, DecorationLines::UNDERLINE);
    assert!(line.rect.y >= baseline, "below the baseline: {line:?}");
    assert!(line.rect.height > 0.0);
    assert!(
        (line.rect.width - under.size.width).abs() < 0.5,
        "as wide as the text: {line:?}"
    );
    assert_eq!(line.color, TextStyle::default().color, "the text's color");
    // `text-underline-offset: 3px` moves it 3 px further down.
    let moved = shaped(
        &mut engine,
        "Under",
        &decorated(DecorationLines::UNDERLINE, 3.0),
        None,
    );
    assert!((moved.decorations[0].rect.y - line.rect.y - 3.0).abs() < 0.01);
    // Over and through: above the baseline, the overline at the top.
    let both = shaped(
        &mut engine,
        "Both",
        &decorated(
            DecorationLines::OVERLINE | DecorationLines::LINE_THROUGH,
            0.0,
        ),
        None,
    );
    let over = both
        .decorations
        .iter()
        .find(|d| d.line == DecorationLines::OVERLINE)
        .expect("an overline");
    let through = both
        .decorations
        .iter()
        .find(|d| d.line == DecorationLines::LINE_THROUGH)
        .expect("a line-through");
    assert!(over.rect.y < through.rect.y && through.rect.y < both.baseline);
    // Decorations change no extent and no shaping.
    assert_eq!(
        both.size,
        shaped(&mut engine, "Both", &style(20.0), None).size
    );
}

#[test]
fn an_ellipsis_ends_a_line_cut_to_fit() {
    let mut engine = engine();
    let text = "A long line that does not fit its box";
    let full = shaped(&mut engine, text, &style(16.0), None);
    let one_line = TextStyle {
        wrap: TextWrap::NoWrap,
        overflow: TextOverflow::Ellipsis,
        ..style(16.0)
    };
    let cut = shaped(&mut engine, text, &one_line, Some(120.0));
    assert!(cut.size.width <= 120.01, "fits: {:?}", cut.size);
    assert_eq!(cut.size.height, full.size.height, "one line");
    assert!(glyph_count(&cut) < glyph_count(&full), "glyphs dropped");
    let last = cut.runs.last().expect("the ellipsis");
    assert_eq!(last.glyphs.len(), 1, "one glyph: …");
    // The ellipsis follows the kept glyphs.
    let before = cut.runs[..cut.runs.len() - 1]
        .iter()
        .flat_map(|r| r.glyphs.iter())
        .map(|g| g.x)
        .fold(f32::MIN, f32::max);
    assert!(last.glyphs[0].x > before);
    // Wide enough: nothing is cut.
    let whole = shaped(&mut engine, text, &one_line, Some(full.size.width + 1.0));
    assert_eq!(glyph_count(&whole), glyph_count(&full));
    assert_eq!(whole.size, full.size);
    // A centered cut line is centered by its cut width.
    let centered = shaped(
        &mut engine,
        text,
        &TextStyle {
            align: ui_types::TextAlign::Center,
            ..one_line.clone()
        },
        Some(120.0),
    );
    let left = centered.runs[0].glyphs[0].x;
    assert!(
        (0.0..2.0).contains(&left),
        "the cut line fills the width: {left}"
    );
}

#[test]
fn a_line_clamp_shows_its_lines_and_ends_the_last_in_an_ellipsis() {
    let mut engine = engine();
    let text = "One paragraph of several words that wraps onto four or five lines at this width";
    let wrapped = shaped(&mut engine, text, &style(14.0), Some(140.0));
    let line = shaped(&mut engine, "One", &style(14.0), None).size.height;
    assert!(wrapped.size.height > 3.0 * line);
    let clamped = TextStyle {
        max_lines: Some(2),
        ..style(14.0)
    };
    let two = shaped(&mut engine, text, &clamped, Some(140.0));
    assert!((two.size.height - 2.0 * line).abs() < 0.01, "two lines");
    assert!(two.size.width <= 140.01);
    assert!(glyph_count(&two) < glyph_count(&wrapped));
    // Clamped at the lines it has: no ellipsis.
    let short = shaped(&mut engine, "Short", &clamped, Some(140.0));
    assert_eq!(glyph_count(&short), 5);
}

#[test]
fn case_conversion_shapes_the_converted_text() {
    let mut engine = engine();
    let upper = TextStyle {
        transform: TextTransform::Uppercase,
        ..style(16.0)
    };
    let converted = shaped(&mut engine, "label", &upper, None);
    let written = shaped(&mut engine, "LABEL", &style(16.0), None);
    assert_eq!(converted.size, written.size);
    assert_eq!(converted.runs, written.runs);
    let capitalized = TextStyle {
        transform: TextTransform::Capitalize,
        ..style(16.0)
    };
    assert_eq!(
        shaped(&mut engine, "open file", &capitalized, None).runs,
        shaped(&mut engine, "Open File", &style(16.0), None).runs
    );
    // A character converts to several; spans move with it.
    let spans = [TextSpan {
        range: 0..2,
        style: SpanStyle {
            color: Some(RED),
            ..SpanStyle::default()
        },
    }];
    let rich = shaped(
        &mut engine,
        RichText {
            text: "ßa",
            spans: &spans,
        },
        &upper,
        None,
    );
    assert_eq!(glyph_count(&rich), 3, "SSA");
    let red: usize = rich
        .runs
        .iter()
        .filter(|r| r.color == RED)
        .map(|r| r.glyphs.len())
        .sum();
    assert_eq!(red, 2, "ß became SS, both red");
}

#[test]
fn spans_style_their_ranges_and_highlight_across_lines() {
    let mut engine = inter();
    let text = "find the matching words in a wrapped line";
    let spans = [
        TextSpan {
            range: 9..17,
            style: SpanStyle {
                weight: Some(700),
                color: Some(RED),
                ..SpanStyle::default()
            },
        },
        TextSpan {
            range: 18..32,
            style: SpanStyle {
                highlight: true,
                decoration: Some(TextDecoration {
                    lines: DecorationLines::UNDERLINE,
                    ..TextDecoration::default()
                }),
                ..SpanStyle::default()
            },
        },
    ];
    let rich = RichText {
        text,
        spans: &spans,
    };
    let plain = shaped(&mut engine, text, &style(16.0), None);
    let one = shaped(&mut engine, rich, &style(16.0), None);
    assert!(one.size.width > plain.size.width, "bold is wider");
    let red = one.runs.iter().find(|r| r.color == RED).expect("a red run");
    assert_eq!(red.glyphs.len(), "matching".len());
    let [band] = one.highlights[..] else {
        panic!("one band on one line: {:?}", one.highlights);
    };
    assert_eq!(band.span, 1);
    assert_eq!(one.decorations.len(), 1, "the span's underline");
    // Wrapped so the highlighted span crosses a line: a band per line.
    let narrow = shaped(&mut engine, rich, &style(16.0), Some(190.0));
    assert!(narrow.size.height > one.size.height);
    assert_eq!(narrow.highlights.len(), 2, "{:?}", narrow.highlights);
    assert!(narrow.highlights[1].rect.y > narrow.highlights[0].rect.y);
    // Colors draw only: the same extent, no new shaping key for layout.
    let recolored = [
        TextSpan {
            style: SpanStyle {
                color: Some(Rgba { r: 0.0, ..RED }),
                ..spans[0].style.clone()
            },
            ..spans[0].clone()
        },
        spans[1].clone(),
    ];
    let other = shaped(
        &mut engine,
        RichText {
            text,
            spans: &recolored,
        },
        &style(16.0),
        None,
    );
    assert_eq!(other.size, one.size);
}

#[test]
fn spans_that_overlap_or_cut_a_character_are_invalid_content() {
    let mut engine = engine();
    let span = |range| TextSpan {
        range,
        style: SpanStyle::default(),
    };
    for spans in [
        vec![span(0..3), span(2..4)],
        vec![span(2..3), span(0..1)],
        vec![span(0..9)],
        vec![span(1..2)],
    ] {
        let error = engine
            .measure_text(
                RichText {
                    text: "éabc",
                    spans: &spans,
                },
                &style(14.0),
                None,
            )
            .expect_err("invalid spans");
        assert_eq!(error.kind(), ErrorKind::Content, "{spans:?}");
    }
}

#[test]
fn text_metrics_give_the_first_baseline() {
    let mut engine = engine();
    let metrics = engine
        .text_metrics("Baseline", &style(20.0), None)
        .expect("metrics");
    let shaped = shaped(&mut engine, "Baseline", &style(20.0), None);
    assert_eq!(metrics.size, shaped.size);
    assert_eq!(metrics.baseline, shaped.baseline);
    assert_eq!(metrics.baseline, shaped.runs[0].glyphs[0].y);
}

/// Tuffy has no check mark (U+2713); Inter has one.
#[test]
fn a_span_falls_back_per_glyph_to_the_fonts_that_cover_it() {
    let mut engine =
        Engine::with_fonts_only(&[arvane::test_display::test_font(), Arc::from(INTER)])
            .expect("both fonts are usable");
    let text = "check \u{2713} done";
    let mark = text.find('\u{2713}').expect("a check mark");
    let spans = [TextSpan {
        range: mark - 2..mark + 5,
        style: SpanStyle {
            color: Some(RED),
            ..SpanStyle::default()
        },
    }];
    let tuffy = TextStyle {
        family: Some("Tuffy".into()),
        ..style(16.0)
    };
    let s = shaped(
        &mut engine,
        RichText {
            text,
            spans: &spans,
        },
        &tuffy,
        None,
    );
    let from_inter: Vec<_> = s
        .runs
        .iter()
        .filter(|r| r.font.len() == INTER.len())
        .collect();
    let [mark_run] = from_inter[..] else {
        panic!("the check mark alone from Inter: {} runs", s.runs.len());
    };
    assert_eq!(mark_run.glyphs.len(), 1);
    assert_eq!(mark_run.color, RED, "in its span's color");
    let red: usize = s
        .runs
        .iter()
        .filter(|r| r.color == RED)
        .map(|r| r.glyphs.len())
        .sum();
    assert_eq!(red, 5, "k, a space, the mark, a space, d");
}

/// A CR before an LF is shaped as a zero-width space, three bytes for one:
/// span ranges, written against the caller's text, still select the same
/// characters after it.
#[test]
fn spans_after_a_crlf_style_the_characters_they_name() {
    let mut engine = inter();
    let red_glyphs = |shaped: &ShapedText| -> usize {
        shaped
            .runs
            .iter()
            .filter(|r| r.color == RED)
            .map(|r| r.glyphs.len())
            .sum()
    };
    let spans = |range| {
        [TextSpan {
            range,
            style: SpanStyle {
                color: Some(RED),
                ..SpanStyle::default()
            },
        }]
    };
    // "a\r\nbcd\r\nef": the span names "bcd" (3..6) and, second, "ef" (8..10).
    let text = "a\r\nbcd\r\nef";
    for (range, expected) in [(3..6, 3), (8..10, 2), (0..1, 1)] {
        let spans = spans(range.clone());
        let rich = shaped(
            &mut engine,
            RichText {
                text,
                spans: &spans,
            },
            &style(16.0),
            None,
        );
        assert_eq!(red_glyphs(&rich), expected, "span {range:?}");
    }
    // The same ranges on LF-only text select the same characters.
    let plain = "a\nbcd\nef";
    let spans = spans(2..5);
    let lf = shaped(
        &mut engine,
        RichText {
            text: plain,
            spans: &spans,
        },
        &style(16.0),
        None,
    );
    assert_eq!(red_glyphs(&lf), 3);
}
