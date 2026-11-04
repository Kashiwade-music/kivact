//! Role: Compare normal line queries using disposable retained and rebuilt copies.

use crate::measure::input::RefreshedLines;
use kivact_text::{EditLines, Source};
use kivact_types::Point;

fn equal<T: PartialEq + std::fmt::Debug>(out: &mut Vec<String>, label: &str, a: T, b: T) {
    if a != b {
        out.push(format!("{label}: retained {a:?}, rebuilt {b:?}"));
    }
}

pub(super) fn compare(
    snapshot: RefreshedLines<'_>,
    caret: usize,
    engine: &mut arvane::Engine,
) -> Vec<String> {
    // Every query below mutates only disposable copies. No live IDs or caches.
    let mut retained = snapshot.lines.clone();
    let mut rebuilt = EditLines::new(
        Source {
            damage: None,
            ..snapshot.source
        },
        snapshot
            .style
            .expect("invariant: Verify captures one reference style"),
        engine,
    );
    let text = retained.shown(snapshot.source.value, snapshot.source.placeholder);
    let mut errors = Vec::new();
    equal(
        &mut errors,
        "revision/value/lines/height",
        (
            retained.revision(),
            retained.shows_value(),
            retained.len(),
            retained.height(),
        ),
        (
            rebuilt.revision(),
            rebuilt.shows_value(),
            rebuilt.len(),
            rebuilt.height(),
        ),
    );
    for i in 0..retained.len().max(rebuilt.len()) {
        equal(
            &mut errors,
            &format!("line {i} content/range/CRLF/scalar prefix"),
            (
                retained.content(i),
                retained.range(i),
                retained.crlf(i),
                retained.char_of_line(i),
            ),
            (
                rebuilt.content(i),
                rebuilt.range(i),
                rebuilt.crlf(i),
                rebuilt.char_of_line(i),
            ),
        );
        let range = rebuilt.range(i);
        let end = rebuilt.content(i).map_or(range.end, |r| r.end);
        for at in [range.start, end, range.end] {
            equal(
                &mut errors,
                &format!("byte {at} line/scalar/UTF16"),
                (
                    retained.line_of(at),
                    retained.char_of_byte(text, at),
                    retained.utf16_of_byte(text, at),
                ),
                (
                    rebuilt.line_of(at),
                    rebuilt.char_of_byte(text, at),
                    rebuilt.utf16_of_byte(text, at),
                ),
            );
        }
    }
    equal(
        &mut errors,
        "terminal scalar/UTF16",
        (
            retained.char_of_line(retained.len()),
            retained.utf16_of_byte(text, text.len()),
        ),
        (
            rebuilt.char_of_line(rebuilt.len()),
            rebuilt.utf16_of_byte(text, text.len()),
        ),
    );
    equal(
        &mut errors,
        "bounded natural width",
        retained.measured_width(text, engine),
        rebuilt.measured_width(text, engine),
    );
    let width = 239.5;
    let caret = caret.min(snapshot.source.value.len());
    let line = rebuilt.line_of(caret);
    let samples: std::collections::BTreeSet<_> = [0, line, rebuilt.len().saturating_sub(1)]
        .into_iter()
        .collect();
    for i in samples {
        equal(
            &mut errors,
            &format!("line {i} baseline/alignment"),
            (
                retained.baseline(text, i, engine),
                retained.offset(text, i, width, engine),
            ),
            (
                rebuilt.baseline(text, i, engine),
                rebuilt.offset(text, i, width, engine),
            ),
        );
        let a = retained.shaped(text, i, engine);
        let b = rebuilt.shaped(text, i, engine);
        if a != b {
            errors.push(format!(
                "line {i} neutral shaping differs: retained {:?}, rebuilt {:?}",
                a.as_ref().map(|s| (s.size, s.baseline)),
                b.as_ref().map(|s| (s.size, s.baseline))
            ));
        }
        let range = rebuilt.content(i).unwrap_or(0..0);
        equal(
            &mut errors,
            &format!("line {i} selection spans"),
            retained.spans(text, i, (range.start, range.end), width, engine),
            rebuilt.spans(text, i, (range.start, range.end), width, engine),
        );
        equal(
            &mut errors,
            &format!("line {i} grapheme hit"),
            retained.nearest(text, i, width / 2.0, width, engine),
            rebuilt.nearest(text, i, width / 2.0, width, engine),
        );
    }
    equal(
        &mut errors,
        "caret",
        retained.caret(text, caret, width, engine),
        rebuilt.caret(text, caret, width, engine),
    );
    equal(
        &mut errors,
        "point hit",
        retained.offset_at(text, width / 2.0, retained.height(), width, engine),
        rebuilt.offset_at(text, width / 2.0, rebuilt.height(), width, engine),
    );
    equal(
        &mut errors,
        "IME excerpt",
        retained.excerpt(text, caret..caret),
        rebuilt.excerpt(text, caret..caret),
    );
    let band = line.saturating_sub(2)..(line + 3).min(rebuilt.len());
    equal(
        &mut errors,
        "visible row band",
        retained.rows(0.0, retained.height() * 3.0),
        rebuilt.rows(0.0, rebuilt.height() * 3.0),
    );
    retained.drawn(band.clone());
    rebuilt.drawn(band);
    // Never re-evaluate user LineColors merely to create a reference. Neutral
    // shapes/publication do not invoke it; colored glyph comparisons are omitted.
    let mut before_id = 1;
    let mut after_id = 1;
    let a = retained.published(
        text,
        caret,
        Point::default(),
        width,
        &mut || {
            before_id += 1;
            before_id
        },
        engine,
    );
    let b = rebuilt.published(
        text,
        caret,
        Point::default(),
        width,
        &mut || {
            after_id += 1;
            after_id
        },
        engine,
    );
    equal(
        &mut errors,
        "publication scalar coverage",
        (a.start, a.end),
        (b.start, b.end),
    );
    let before: Vec<_> = a
        .runs
        .iter()
        .map(|r| (&r.semantics.value_text, r.bounds, &r.carets))
        .collect();
    let after: Vec<_> = b
        .runs
        .iter()
        .map(|r| (&r.semantics.value_text, r.bounds, &r.carets))
        .collect();
    if before != after {
        // Compare all raw values strictly, but never put document text into
        // failure evidence (password masking is only a display transform).
        let first = before.iter().zip(&after).position(|(a, b)| a != b);
        let describe = |runs: &[std::sync::Arc<kivact_types::SemanticsTextRun>]| {
            first
                .and_then(|i| runs.get(i))
                .map(|r| (r.semantics.value_text.len(), r.bounds, r.carets.len()))
        };
        errors.push(format!(
            "publication values differ (text redacted, IDs excluded): run counts {}/{}, first {first:?}, retained {:?}, rebuilt {:?}",
            before.len(), after.len(), describe(&a.runs), describe(&b.runs)));
    }
    errors
}
