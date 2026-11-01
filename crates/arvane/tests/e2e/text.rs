//! Role: Text rendering contract tests.
//!
//! Text: shaping and measuring agree, another width only wraps again, font
//! families are known and registered, unknown families fall back, white
//! space alone never wraps, and bad input is classified.

use std::sync::Arc;

use arvane::asset::{FamilyChoice, FamilyResolution};
use arvane::test_display::test_font;
use arvane::{Engine, ErrorKind};
use ui_types::{
    DrawCommand, LineHeight, ShapedText, Size, TextLine, TextOverflow, TextStyle, TextWrap,
};

use crate::support::{engine, open, show};

fn style(font_size: f32) -> TextStyle {
    TextStyle {
        font_size,
        ..TextStyle::default()
    }
}

fn with_line_height(font_size: f32, line_height: LineHeight) -> TextStyle {
    TextStyle {
        line_height,
        ..style(font_size)
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
fn font_families_are_known_by_name_ignoring_case() {
    let mut engine = engine();
    assert!(engine.has_font_family("Tuffy"));
    assert!(engine.has_font_family("tUFFY"));
    assert!(!engine.has_font_family("No Such Family"));
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

#[test]
fn registering_a_font_drops_what_was_cached_without_it() {
    let font = test_font();
    let mut engine = Engine::with_fonts_only(&[]).expect("an engine without fonts");
    assert!(engine.font_families().is_empty());
    let tuffy = TextStyle {
        family: Some("Tuffy".into()),
        ..style(16.0)
    };
    // Cached answers without the font ...
    assert!(!engine.has_font_family("Tuffy"));
    let before = engine.shape("Hello", &tuffy, None);
    let measured_before = engine.measure_text("Hello", &tuffy, None);
    // ... are dropped when it is registered.
    assert_eq!(
        engine.register_font(font.clone()).expect("a font"),
        ["Tuffy"]
    );
    assert!(engine.has_font_family("Tuffy"));
    assert_eq!(engine.font_families(), ["Tuffy"]);
    engine.release_caches();
    assert_eq!(
        engine.font_families(),
        ["Tuffy"],
        "font names survive cache release"
    );
    let after = engine.shape("Hello", &tuffy, None).expect("shaped");
    assert_ne!(Ok(after.clone()), before, "shaped again with the font");
    assert!(
        after.runs.iter().all(|run| run.font == font),
        "every run uses the registered font"
    );
    let measured = engine
        .measure_text("Hello", &tuffy, None)
        .expect("measured");
    assert_ne!(Ok(measured), measured_before);
    assert_eq!(measured, after.size);
}

#[test]
fn css_family_lists_keep_ordered_fallback_and_quoted_names() {
    let mut engine = engine();
    let font = test_font();
    for source in [
        "No Such Family, 'Tuffy', sans-serif",
        "'Tuffy'",
        "\"Tuffy\", system-ui",
        "No Such Family, 'Tuffy',",
    ] {
        let resolution = engine.resolve_font_family(source);
        assert!(!resolution.missing, "optional list entries: {source}");
        let FamilyChoice::Family(family) = resolution.choice else {
            panic!("the explicit fallback order must survive resolution");
        };
        let shaped = engine
            .shape(
                "Explicit fallback",
                &TextStyle {
                    family: Some(family),
                    ..style(16.0)
                },
                None,
            )
            .expect("the registered face shapes the text");
        assert!(shaped.runs.iter().all(|run| run.font == font), "{source}");
    }
    let direct = engine
        .shape(
            "Direct quoted family",
            &TextStyle {
                family: Some("  'Tuffy'  ".into()),
                ..style(16.0)
            },
            None,
        )
        .expect("CSS whitespace without facade resolution");
    assert!(direct.runs.iter().all(|run| run.font == font));
    // Invalid syntax must neither panic nor discard the system fallback.
    assert!(
        engine
            .shape(
                "Still readable",
                &TextStyle {
                    family: Some("Tuffy,, 'unfinished".into()),
                    ..style(16.0)
                },
                None
            )
            .is_ok()
    );
}

#[test]
fn resolving_a_font_family_uses_the_engine_fonts_and_platform_defaults() {
    let mut engine = Engine::with_fonts_only(&[]).expect("an engine without fonts");
    assert_eq!(
        engine.resolve_font_family("Tuffy"),
        FamilyResolution {
            choice: FamilyChoice::SystemUi,
            missing: true
        }
    );
    engine.register_font(test_font()).expect("a font");
    assert_eq!(
        engine.resolve_font_family(" tUFFY "),
        FamilyResolution {
            choice: FamilyChoice::Family("tUFFY".into()),
            missing: false
        }
    );
    engine = Engine::with_fonts_only(&[test_font()]).expect("a deterministic UI fallback");
    // The isolated collection has no platform candidate installed. Its
    // own system UI fallback still shapes with the registered test font.
    for name in ["", "   ", "system-ui", "Sans-Serif", " serif ", "monospace"] {
        let resolved = engine.resolve_font_family(name);
        assert_eq!(
            resolved,
            FamilyResolution {
                choice: FamilyChoice::SystemUi,
                missing: false
            }
        );
    }
    let resolution = engine.resolve_font_family("No Such Family");
    assert!(resolution.missing);
    let family = match resolution.choice {
        FamilyChoice::Family(name) => Some(name),
        FamilyChoice::SystemUi => None,
    };
    let shaped = engine
        .shape(
            "Fallback",
            &TextStyle {
                family,
                ..style(16.0)
            },
            None,
        )
        .expect("fallback");
    let mut window = open(120.0, 60.0);
    show(
        &mut window,
        shaped
            .runs
            .iter()
            .cloned()
            .map(|run| DrawCommand::Glyphs {
                run,
                origin: ui_types::Point::default(),
                color: None,
            })
            .collect(),
    );
    assert!(
        window
            .frame()
            .expect("a frame")
            .2
            .iter()
            .any(|pixel| *pixel != 0x00ff_ffff)
    );
}

#[test]
fn releasing_text_caches_drops_unused_font_copies_and_preserves_owned_runs() {
    let mut engine = engine();
    let mut window = open(120.0, 60.0);
    let style = TextStyle {
        overflow: TextOverflow::Ellipsis,
        max_lines: Some(1),
        ..style(16.0)
    };
    let width = Some(100.0);
    let shaped = engine.shape(PARAGRAPH, &style, width).expect("shape");
    let measured = engine
        .text_metrics(PARAGRAPH, &style, width)
        .expect("measure");
    assert!(Arc::ptr_eq(
        &shaped,
        &engine
            .shape(PARAGRAPH, &style, width)
            .expect("cached shape"),
    ));
    let unused_font = Arc::downgrade(&shaped.runs[0].font);
    drop(shaped);
    assert!(
        unused_font.upgrade().is_some(),
        "the engine caches the font"
    );
    engine.release_caches();
    assert!(
        unused_font.upgrade().is_none(),
        "no caller owns the old font copy, so every text cache releases it"
    );

    let owned = engine.shape(PARAGRAPH, &style, width).expect("reshaped");
    let cached = engine
        .shape(PARAGRAPH, &style, width)
        .expect("cached again");
    assert!(Arc::ptr_eq(&owned, &cached));
    drop(cached);
    show(
        &mut window,
        owned
            .runs
            .iter()
            .cloned()
            .map(|run| DrawCommand::Glyphs {
                run,
                origin: ui_types::Point::default(),
                color: None,
            })
            .collect(),
    );
    let pixels = window.frame().expect("a frame").2.to_vec();
    engine.release_caches();
    assert!(engine.has_font_family("Tuffy"), "registered fonts remain");
    assert_eq!(
        engine
            .text_metrics(PARAGRAPH, &style, width)
            .expect("remeasure"),
        measured
    );
    assert_eq!(
        engine.shape(PARAGRAPH, &style, width).expect("reshaped"),
        owned
    );
    drop(engine);
    show(
        &mut window,
        owned
            .runs
            .iter()
            .cloned()
            .map(|run| DrawCommand::Glyphs {
                run,
                origin: ui_types::Point::default(),
                color: None,
            })
            .collect(),
    );
    assert_eq!(
        window.frame().expect("a frame").2,
        pixels.as_slice(),
        "owned runs still draw"
    );
}

#[test]
fn bad_font_bytes_and_bad_metrics_are_classified() {
    let mut engine = engine();
    let error = engine
        .register_font(Arc::from(&b"not a font"[..]))
        .expect_err("not a font");
    assert_eq!(error.kind(), ErrorKind::Resource);
    let error = Engine::with_fonts_only(&[Arc::from(&b"garbage"[..])])
        .err()
        .expect("not a font");
    assert_eq!(error.kind(), ErrorKind::Resource);
    for bad in [f32::NAN, -1.0, 0.0, f32::INFINITY] {
        let error = engine
            .shape("x", &style(bad), None)
            .expect_err("bad font size");
        assert_eq!(error.kind(), ErrorKind::Content, "font size {bad}");
        let error = engine
            .measure_text("x", &style(bad), None)
            .expect_err("bad font size");
        assert_eq!(error.kind(), ErrorKind::Content, "font size {bad}");
    }
    // A failure is not the engine's end.
    assert_ne!(
        engine.measure_text("x", &style(14.0), None).expect("good"),
        Size::default()
    );
}

/// The height of one line of `text_style`, measured as the shaper lays it out.
fn line_of(engine: &mut Engine, text_style: &TextStyle) -> f32 {
    engine
        .measure_text("a", text_style, None)
        .expect("measure")
        .height
}

#[test]
fn a_scaled_line_height_multiplies_the_font_size_on_both_sides_of_four() {
    let mut engine = engine();
    // The number never changes meaning: 4.5 times the size is not 4.5 px.
    let mut last = 0.0;
    for scale in [1.0, 3.9, 4.0, 4.1, 4.5, 6.0] {
        let height = line_of(
            &mut engine,
            &with_line_height(14.0, LineHeight::Scale(scale)),
        );
        assert!(
            (height - 14.0 * scale).abs() < 0.01,
            "scale {scale}: line of {height}"
        );
        assert!(height > last, "scale {scale} is taller than the one before");
        last = height;
    }
}

#[test]
fn a_line_height_in_px_is_the_same_on_both_sides_of_four() {
    let mut engine = engine();
    for px in [1.0, 3.0, 4.0, 4.5, 5.0, 30.0] {
        for size in [10.0, 14.0, 20.0] {
            let height = line_of(&mut engine, &with_line_height(size, LineHeight::Px(px)));
            assert!(
                (height - px).abs() < 0.01,
                "{px} px at size {size}: line of {height}"
            );
        }
    }
}

#[test]
fn a_normal_line_height_is_the_font_size_times_1_3_rounded() {
    let mut engine = engine();
    for (size, expected) in [
        (10.0, 13.0),
        (11.0, 14.0),
        (14.0, 18.0),
        (15.0, 20.0),
        (20.0, 26.0),
    ] {
        let height = line_of(&mut engine, &with_line_height(size, LineHeight::Normal));
        assert!(
            (height - expected).abs() < 0.01,
            "size {size}: line of {height}, expected {expected}"
        );
        // Normal is what a style without a line height gets.
        assert_eq!(height, line_of(&mut engine, &style(size)), "size {size}");
    }
}

#[test]
fn a_line_height_that_is_not_finite_or_not_above_zero_is_classified() {
    let mut engine = engine();
    for bad in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
        for line_height in [LineHeight::Scale(bad), LineHeight::Px(bad)] {
            let error = engine
                .measure_text("x", &with_line_height(14.0, line_height), None)
                .expect_err("bad line height");
            assert_eq!(error.kind(), ErrorKind::Content, "{line_height:?}");
        }
    }
}

fn lines_of(
    engine: &mut Engine,
    text: &str,
    style: &TextStyle,
    width: Option<f32>,
) -> (Arc<ShapedText>, Vec<TextLine>) {
    let shaped = engine.shape(text, style, width).expect("shape");
    let lines = shaped.lines.clone();
    (shaped, lines)
}

#[test]
fn a_single_line_has_its_range_baseline_and_font_heights() {
    let mut engine = engine();
    let text = "Hello, world";
    let (shaped, lines) = lines_of(&mut engine, text, &style(20.0), None);
    let [line] = &lines[..] else {
        panic!("one line: {lines:?}");
    };
    assert_eq!(line.range, 0..text.len());
    assert!(!line.cut);
    assert_eq!(line.baseline, shaped.baseline);
    // The line box is the paragraph; its content is what the glyphs advance.
    assert_eq!((line.rect.x, line.rect.y), (0.0, 0.0));
    assert_eq!(line.rect.height, shaped.size.height);
    assert!(line.rect.width > 0.0 && line.rect.width <= shaped.size.width + 0.01);
    let pen_end = shaped.runs[0]
        .glyphs
        .last()
        .map(|g| g.x + g.advance)
        .expect("glyphs");
    assert!((line.rect.width - pen_end).abs() < 0.5, "{line:?}");
    // The font's heights are positive and fit above the baseline.
    assert!(line.x_height > 0.0, "{line:?}");
    assert!(line.cap_height >= line.x_height, "{line:?}");
    assert!(line.cap_height <= line.ascent, "{line:?}");
    assert!(line.descent > 0.0);
    assert!(line.baseline > 0.0 && line.baseline <= line.rect.height + 0.01);
    // The same through the drawing-only snapshot.
    let glyphs = engine
        .shape_glyphs(text, &style(20.0), None)
        .expect("glyphs");
    assert_eq!(glyphs.lines, lines);
}

#[test]
fn a_trailing_space_is_in_the_range_but_not_the_content() {
    let mut engine = engine();
    let (_, lines) = lines_of(&mut engine, "Hello   ", &style(16.0), None);
    let (_, bare) = lines_of(&mut engine, "Hello", &style(16.0), None);
    assert_eq!(lines[0].range, 0..8);
    assert!((lines[0].rect.width - bare[0].rect.width).abs() < 0.01);
}

#[test]
fn white_space_alone_is_one_line_at_any_width() {
    let mut engine = engine();
    for wrap in [TextWrap::Wrap, TextWrap::NoWrap] {
        let style = TextStyle {
            wrap,
            ..style(20.0)
        };
        let line = engine.measure_text("x", &style, None).expect("a line");
        // Narrower than one space: the white space hangs, as CSS does,
        // instead of each space breaking a line of its own.
        for text in [" ", "    ", "	 "] {
            for width in [None, Some(0.0), Some(2.0)] {
                let measured = engine.measure_text(text, &style, width).expect("measure");
                assert_eq!(
                    measured.height, line.height,
                    "{text:?} at {width:?}, {wrap:?}"
                );
                let (_, lines) = lines_of(&mut engine, text, &style, width);
                assert_eq!(lines.len(), 1, "{text:?} at {width:?}, {wrap:?}: {lines:?}");
            }
        }
        // A line break still breaks.
        let (_, lines) = lines_of(
            &mut engine,
            " 
 ",
            &style,
            Some(0.0),
        );
        assert_eq!(lines.len(), 2, "{lines:?}");
    }
}

#[test]
fn empty_text_has_no_lines() {
    let mut engine = engine();
    let (_, lines) = lines_of(&mut engine, "", &style(16.0), Some(100.0));
    assert!(lines.is_empty());
}

#[test]
fn wrapped_lines_tile_the_text_and_the_paragraph_downwards() {
    let mut engine = engine();
    let (shaped, lines) = lines_of(&mut engine, PARAGRAPH, &style(14.0), Some(120.0));
    assert!(lines.len() >= 3, "wrapped: {lines:?}");
    assert_eq!(lines[0].range.start, 0);
    assert_eq!(lines.last().expect("lines").range.end, PARAGRAPH.len());
    for pair in lines.windows(2) {
        assert_eq!(pair[0].range.end, pair[1].range.start, "{pair:?}");
        assert!(pair[0].baseline < pair[1].baseline, "{pair:?}");
        let bottom = pair[0].rect.y + pair[0].rect.height;
        assert!((bottom - pair[1].rect.y).abs() < 0.01, "{pair:?}");
    }
    for line in &lines {
        assert!(!line.range.is_empty());
        assert!(PARAGRAPH.is_char_boundary(line.range.start));
        assert!(PARAGRAPH.is_char_boundary(line.range.end));
        assert!(line.rect.width <= 120.01, "{line:?}");
        assert!(!line.cut);
        assert!(line.x_height > 0.0 && line.x_height <= line.cap_height);
        assert!(line.cap_height <= line.ascent);
    }
    assert_eq!(lines[0].baseline, shaped.baseline);
    let last = lines.last().expect("lines");
    assert!((last.rect.y + last.rect.height - shaped.size.height).abs() < 0.01);
    // A hard break ends its line.
    let (_, broken) = lines_of(&mut engine, "one\ntwo", &style(16.0), None);
    assert_eq!(
        broken.iter().map(|l| l.range.clone()).collect::<Vec<_>>(),
        [0..4, 4..7]
    );
}

#[test]
fn an_ellipsis_marks_the_lines_it_cut() {
    let mut engine = engine();
    let text = "A long line that does not fit its box";
    let one_line = TextStyle {
        wrap: TextWrap::NoWrap,
        overflow: TextOverflow::Ellipsis,
        ..style(16.0)
    };
    let (cut, lines) = lines_of(&mut engine, text, &one_line, Some(120.0));
    let [line] = &lines[..] else {
        panic!("one line: {lines:?}");
    };
    assert!(line.cut);
    // The whole text stays the line's range; the content is the kept text
    // and the ellipsis.
    assert_eq!(line.range, 0..text.len());
    assert!(
        line.rect.width <= 120.01 && line.rect.width > 60.0,
        "{line:?}"
    );
    assert_eq!(line.rect.width, cut.size.width);
    let ellipsis = cut.runs.last().expect("the ellipsis").glyphs[0];
    assert!((line.rect.x + line.rect.width - (ellipsis.x + ellipsis.advance)).abs() < 0.5);
    // Wide enough: nothing is cut.
    let (_, whole) = lines_of(&mut engine, text, &one_line, Some(1000.0));
    assert!(!whole[0].cut);

    // A line clamp cuts the last line it shows and hides the others.
    let clamp = TextStyle {
        max_lines: Some(2),
        overflow: TextOverflow::Ellipsis,
        ..style(14.0)
    };
    let (shaped, lines) = lines_of(&mut engine, PARAGRAPH, &clamp, Some(120.0));
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_eq!(
        lines.iter().map(|l| l.cut).collect::<Vec<_>>(),
        [false, true]
    );
    assert!((lines[1].rect.y + lines[1].rect.height - shaped.size.height).abs() < 0.01);
}

#[test]
fn cjk_and_latin_lines_keep_whole_characters_in_their_ranges() {
    let mut engine = engine();
    let text = "Hello 日本語のテキスト and 漢字 world";
    let (_, lines) = lines_of(&mut engine, text, &style(16.0), Some(90.0));
    assert!(lines.len() >= 2, "{lines:?}");
    assert_eq!(lines[0].range.start, 0);
    assert_eq!(lines.last().expect("lines").range.end, text.len());
    for pair in lines.windows(2) {
        assert_eq!(pair[0].range.end, pair[1].range.start);
        assert!(pair[0].baseline < pair[1].baseline);
    }
    for line in &lines {
        assert!(text.get(line.range.clone()).is_some(), "{line:?}");
        assert!(line.cap_height > 0.0 && line.x_height > 0.0);
    }
}

#[test]
fn ranges_follow_the_original_text_through_case_conversion_and_crlf() {
    let mut engine = engine();
    // "ß" becomes "SS" in upper case: the converted text is longer.
    let upper = TextStyle {
        transform: ui_types::TextTransform::Uppercase,
        ..style(16.0)
    };
    let text = "straße straße straße";
    let (_, lines) = lines_of(&mut engine, text, &upper, Some(80.0));
    assert!(lines.len() >= 2, "{lines:?}");
    assert_eq!(lines[0].range.start, 0);
    assert_eq!(lines.last().expect("lines").range.end, text.len());
    for pair in lines.windows(2) {
        assert_eq!(pair[0].range.end, pair[1].range.start);
    }
    for line in &lines {
        assert!(text.get(line.range.clone()).is_some(), "{line:?}");
    }
    // A CRLF breaks once and counts two bytes.
    let (_, lines) = lines_of(&mut engine, "ab\r\ncd", &style(16.0), None);
    assert_eq!(
        lines.iter().map(|l| l.range.clone()).collect::<Vec<_>>(),
        [0..4, 4..6]
    );
}

#[test]
fn glyph_ink_lies_over_the_advance_of_its_line() {
    let mut engine = engine();
    let text = "Hello, Wxyz";
    let (shaped, lines) = lines_of(&mut engine, text, &style(24.0), None);
    let line = &lines[0];
    let ink = arvane::glyph_ink_bounds(&shaped.runs[0]).expect("ink");
    let size = 24.0;
    // Overhangs aside, the ink is the line's content.
    assert!(ink.x >= line.rect.x - size * 0.2, "{ink:?} in {line:?}");
    assert!(ink.x + ink.width <= line.rect.x + line.rect.width + size * 0.2);
    assert!(ink.x + ink.width > line.rect.x + line.rect.width * 0.8);
    // Inside the line box, its top about a capital above the baseline.
    assert!(ink.y >= line.rect.y && ink.y + ink.height <= line.rect.y + line.rect.height);
    assert!(
        (line.baseline - ink.y - line.cap_height).abs() < size * 0.15,
        "{ink:?} against {line:?}"
    );
    // White space has no ink.
    let blank = engine.shape(" ", &style(24.0), None).expect("blank");
    assert!(
        blank
            .runs
            .iter()
            .all(|run| arvane::glyph_ink_bounds(run).is_none())
    );
}
