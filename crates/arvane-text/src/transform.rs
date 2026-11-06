//! Role: Case conversion (CSS `text-transform`) applied before shaping.
//!
//! Case conversion (CSS `text-transform`) applied before shaping, with the
//! spans moved to the converted text.
//!
//! Invariant: a span covers the conversion of exactly the characters it
//! covered (a character may become several: `ß` upper cases to `SS`).

use std::borrow::Cow;

use kivact_types::TextTransform;
use kivact_types::render::TextSpan;

/// `text` converted by `transform`, and `spans` moved onto it. Borrowed as
/// they are when nothing changes.
pub(crate) fn apply<'a>(
    text: &'a str,
    spans: &'a [TextSpan],
    transform: TextTransform,
) -> (Cow<'a, str>, Cow<'a, [TextSpan]>) {
    if transform == TextTransform::None {
        return (Cow::Borrowed(text), Cow::Borrowed(spans));
    }
    let mut out = String::with_capacity(text.len());
    // The converted offset of each original character boundary (byte
    // offset in `text` → byte offset in `out`), in order.
    let mut map: Vec<(usize, usize)> =
        Vec::with_capacity(if spans.is_empty() { 0 } else { text.len() + 1 });
    let mut word_start = true;
    for (at, c) in text.char_indices() {
        if !spans.is_empty() {
            map.push((at, out.len()));
        }
        push_converted(&mut out, c, transform, word_start);
        word_start = next_word_start(c, word_start);
    }
    if out == text {
        return (Cow::Borrowed(text), Cow::Borrowed(spans));
    }
    if spans.is_empty() {
        return (Cow::Owned(out), Cow::Borrowed(spans));
    }
    map.push((text.len(), out.len()));
    let moved = |offset: usize| -> usize {
        match map.binary_search_by_key(&offset, |(from, _)| *from) {
            Ok(i) => map[i].1,
            // Not a boundary (an invalid span is dropped later): the next one.
            Err(i) => map.get(i).map_or(out.len(), |(_, to)| *to),
        }
    };
    let spans = spans
        .iter()
        .map(|span| TextSpan {
            range: moved(span.range.start)..moved(span.range.end),
            style: span.style.clone(),
        })
        .collect();
    (Cow::Owned(out), Cow::Owned(spans))
}

/// Appends `c` converted by `transform`; `word_start` is whether it starts
/// a word.
fn push_converted(out: &mut String, c: char, transform: TextTransform, word_start: bool) {
    match transform {
        TextTransform::Uppercase => out.extend(c.to_uppercase()),
        TextTransform::Lowercase => out.extend(c.to_lowercase()),
        TextTransform::Capitalize if word_start && c.is_alphabetic() => {
            out.extend(c.to_uppercase())
        }
        _ => out.push(c),
    }
}

/// Whether the character after `c` starts a word.
fn next_word_start(c: char, word_start: bool) -> bool {
    if c.is_alphanumeric() {
        false
    } else if c.is_whitespace() {
        true
    } else {
        word_start
    }
}

/// The byte offsets the layout of `text` is built on, against those of
/// `text`: `(layout, original)` at every character boundary and at the
/// end. The layout holds `text` converted by `transform`, with a CR before
/// an LF as a three byte zero width space. `None` when the two offsets are
/// always the same.
pub(crate) fn boundaries(text: &str, transform: TextTransform) -> Option<Vec<(usize, usize)>> {
    if transform == TextTransform::None && !text.contains("\r\n") {
        return None;
    }
    let mut converted = String::new();
    let mut pairs = Vec::with_capacity(text.len() + 1);
    let (mut layout, mut word_start, mut differs) = (0, true, false);
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        pairs.push((layout, at));
        converted.clear();
        push_converted(&mut converted, c, transform, word_start);
        word_start = next_word_start(c, word_start);
        let crlf = c == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n');
        let grown = if crlf {
            '\u{200B}'.len_utf8() - c.len_utf8()
        } else {
            0
        };
        differs |= converted.len() != c.len_utf8() || grown != 0;
        layout += converted.len() + grown;
    }
    pairs.push((layout, text.len()));
    differs.then_some(pairs)
}

/// The offset in the original text of the layout offset `at`, by
/// [`boundaries`] (the next boundary when `at` is inside a character).
pub(crate) fn original(pairs: &[(usize, usize)], at: usize) -> usize {
    let i = pairs.partition_point(|(layout, _)| *layout < at);
    pairs
        .get(i)
        .or(pairs.last())
        .map_or(at, |(_, original)| *original)
}
