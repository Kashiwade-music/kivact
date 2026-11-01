//! Role: Text rendering contract tests.
//!
//! Text: shaping and measuring agree, another width only wraps again, font
//! families are known and registered, unknown families fall back, white
//! space alone never wraps, and bad input is classified.

use kivact_types::protocol::{TextStyle, TextWrap};

use crate::support::engine;

fn style(font_size: f32) -> TextStyle {
    TextStyle {
        font_size,
        ..TextStyle::default()
    }
}

const PARAGRAPH: &str = "Arvane draws the frames, shapes the text and keeps the tree.";

#[test]
fn measuring_gives_the_shaped_size() {
    let mut engine = engine();
    for (text, size, width) in [
        ("", 14.0, None),
        ("a", 14.0, None),
        ("Hello, world", 20.0, None),
        (PARAGRAPH, 14.0, None),
        (PARAGRAPH, 14.0, Some(120.0)),
        (PARAGRAPH, 18.0, Some(200.0)),
        ("line one\nline two", 16.0, None),
    ] {
        let style = style(size);
        let measured = engine.measure_text(text, &style, width).expect("measure");
        let shaped = engine.shape(text, &style, width).expect("shape");
        let glyphs = engine.shape_glyphs(text, &style, width).expect("glyphs");
        assert_eq!(measured, shaped.size, "{text:?} at {width:?}");
        assert_eq!(glyphs.size, shaped.size, "{text:?} at {width:?}");
        assert_eq!(glyphs.runs, shaped.runs, "{text:?} at {width:?}");
        assert!(glyphs.carets.is_empty());
        // A caret for every scalar boundary, the last one included.
        assert_eq!(shaped.carets.len(), text.chars().count() + 1, "{text:?}");
    }
}

#[test]
fn another_width_wraps_the_same_text_again() {
    let mut engine = engine();
    let style = style(14.0);
    let line = engine
        .measure_text(PARAGRAPH, &style, None)
        .expect("one line");
    let narrow = engine
        .measure_text(PARAGRAPH, &style, Some(line.width / 3.0))
        .expect("wrapped");
    assert!(narrow.width <= line.width / 3.0, "{narrow:?} fits");
    assert!(
        narrow.height >= line.height * 3.0,
        "three lines or more: {narrow:?} vs {line:?}"
    );
    // Back to the first width: the same extent as before.
    assert_eq!(
        engine.measure_text(PARAGRAPH, &style, None).expect("again"),
        line
    );
    // No wrapping keeps one line whatever the width.
    let nowrap = TextStyle {
        wrap: TextWrap::NoWrap,
        ..style
    };
    let kept = engine
        .measure_text(PARAGRAPH, &nowrap, Some(line.width / 3.0))
        .expect("not wrapped");
    assert_eq!(kept.height, line.height);
}

#[test]
fn an_unknown_family_falls_back_to_the_fonts_there_are() {
    let mut engine = engine();
    let known = engine.shape("Fallback", &style(16.0), None).expect("shape");
    let unknown = engine
        .shape(
            "Fallback",
            &TextStyle {
                family: Some("No Such Family".into()),
                ..style(16.0)
            },
            None,
        )
        .expect("shape");
    assert!(!unknown.runs.is_empty(), "drawn with a fallback");
    assert_eq!(unknown.size, known.size);
    assert_eq!(
        unknown.runs[0].font, known.runs[0].font,
        "the bundled font stands in"
    );
}
