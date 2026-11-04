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

use arvane::Engine;
use kivact_types::render::{RichText, ShapedText, SpanStyle, TextSpan};
use kivact_types::{ComputedTextStyle, TextDecoration};
use kivact_types::{DecorationLines, Rgba};

use crate::support::engine;

static INTER: &[u8] = include_bytes!("../fonts/InterVariable.ttf");

fn inter() -> Engine {
    Engine::with_fonts_only(&[Arc::from(INTER)]).expect("Inter is usable")
}

fn style(font_size: f64) -> ComputedTextStyle {
    ComputedTextStyle {
        font_size,
        ..ComputedTextStyle::default()
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
    style: &ComputedTextStyle,
    width: Option<f64>,
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

#[test]
fn capsize_uses_shaped_cap_height_and_last_visible_baseline() {
    let mut engine = inter();
    let normal_style = ComputedTextStyle {
        max_lines: Some(2),
        ..style(24.0)
    };
    let cap_style = ComputedTextStyle {
        capsize: true,
        ..normal_style.clone()
    };
    let text = "AHoy\nAXgy\nClamped";
    let plain = shaped(&mut engine, text, &normal_style, Some(160.0));
    let trimmed = shaped(&mut engine, text, &cap_style, Some(160.0));
    assert_eq!(trimmed.lines.len(), 2);
    assert!(trimmed.lines[1].cut);
    let cap = plain.lines[0].cap_height;
    assert!(cap > 0.0);
    let shift = plain.lines[0].baseline - cap;
    assert!((trimmed.baseline - cap).abs() < 0.01);
    assert!((trimmed.size.height - (plain.lines[1].baseline - shift)).abs() < 0.01);
    assert!((trimmed.lines[1].baseline - trimmed.size.height).abs() < 0.01);
    assert!((trimmed.runs[0].glyphs[0].y - (plain.runs[0].glyphs[0].y - shift)).abs() < 0.01);
    assert_eq!(trimmed.size.width, plain.size.width);
}

#[test]
fn letter_spacing_adds_its_space_after_every_character() {
    let mut engine = engine();
    let plain = shaped(&mut engine, "spacing", &style(16.0), None);
    let spaced = ComputedTextStyle {
        letter_spacing: kivact_types::TextLen::Px(3.0),
        ..style(16.0)
    };
    let wide = shaped(&mut engine, "spacing", &spaced, None);
    let grown = wide.size.width - plain.size.width;
    assert!(
        (grown - 7.0 * 3.0).abs() < 0.1,
        "seven characters, 3 px each: grown by {grown}"
    );
    // A negative spacing tightens.
    let tight = ComputedTextStyle {
        letter_spacing: kivact_types::TextLen::Px(-1.0),
        ..style(16.0)
    };
    assert!(shaped(&mut engine, "spacing", &tight, None).size.width < plain.size.width);
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
