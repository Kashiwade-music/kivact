//! Role: A long line shaped in pieces of under 64 KiB, joined into one line.
//!
//! parley keeps a cluster's text offset in 16 bits, so a paragraph past
//! 64 KiB gets wrong carets and ranges. A line longer than [`FRAGMENT`]
//! bytes is shaped in pieces cut at grapheme boundaries (after white
//! space where one is near the cut), joined end to end on one baseline.
//!
//! Invariants:
//! - The joined line has one caret per scalar offset of the whole line,
//!   and one line box; glyphs, carets and decorations keep their pieces'
//!   positions moved by the pieces before them. Ligatures, kerning and
//!   bidi reordering across a cut are lost.

use std::ops::Range;
use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation as _;

use kivact_types::Rect;

/// The longest piece of a line shaped at once, in bytes (parley keeps a
/// cluster's text offset in 16 bits).
const FRAGMENT: usize = 16 * 1024;

/// The pieces `text` is shaped in: all of it when it is short, else
/// pieces of at most [`FRAGMENT`] bytes ending at grapheme boundaries,
/// after white space where one is in the piece's second half.
fn fragments(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut from = 0;
    while text.len() - from > FRAGMENT {
        let piece = &text[from..kivact_types::text_utf::floor_char_boundary(text, from + FRAGMENT)];
        let (mut last, mut after_space, mut space) = (0, 0, false);
        for (at, g) in piece.grapheme_indices(true) {
            if at > 0 {
                last = at;
                if space {
                    after_space = at;
                }
            }
            space = g.chars().all(char::is_whitespace);
        }
        let cut = match () {
            _ if after_space >= piece.len() / 2 => after_space,
            _ if last > 0 => last,
            _ => piece.len().max(1),
        };
        out.push(from..from + cut);
        from += cut;
    }
    out.push(from..text.len());
    out
}

/// `text` shaped in its [`fragments`] by `shape`, the pieces joined end to
/// end on one baseline into one line.
pub(super) fn shape_long(
    text: &str,
    mut shape: impl FnMut(Range<usize>) -> Option<kivact_types::ShapedTextSnapshot>,
) -> Option<kivact_types::ShapedTextSnapshot> {
    let pieces = fragments(text);
    if pieces.len() == 1 {
        return shape(0..text.len());
    }
    let shaped: Vec<(Range<usize>, kivact_types::render::ShapedText)> = pieces
        .into_iter()
        .map(|range| {
            Some((
                range.clone(),
                Arc::unwrap_or_clone(shape(range)?.into_shared()),
            ))
        })
        .collect::<Option<_>>()?;
    let baseline = shaped.iter().map(|(_, s)| s.baseline).fold(0.0, f64::max);
    let mut out = kivact_types::render::ShapedText {
        baseline,
        ..Default::default()
    };
    let mut line: Option<kivact_types::render::TextLine> = None;
    let (mut x, mut scalars) = (0.0f64, 0u32);
    for (i, (range, piece)) in shaped.into_iter().enumerate() {
        let dy = baseline - piece.baseline;
        let advance = piece
            .runs
            .iter()
            .flat_map(|r| r.glyphs.iter())
            .map(|g| g.x + g.advance)
            .fold(piece.size.width, f64::max);
        let at = |r: Rect| Rect {
            x: r.x + x,
            y: r.y + dy,
            ..r
        };
        for run in piece.runs {
            let mut run = (*run).clone();
            for g in &mut run.glyphs {
                g.x += x;
                g.y += dy;
            }
            out.runs.push(run.into());
        }
        // A piece's first caret is the one before it ended at.
        let skip = usize::from(i > 0 && !out.carets.is_empty());
        out.carets.extend(
            piece
                .carets
                .iter()
                .skip(skip)
                .map(|c| kivact_types::TextCaret {
                    offset: c.offset + scalars,
                    rect: at(c.rect),
                }),
        );
        out.cluster_boundaries.extend(
            piece
                .cluster_boundaries
                .iter()
                .map(|offset| offset + scalars),
        );
        out.decorations
            .extend(piece.decorations.into_iter().map(|mut d| {
                d.rect = at(d.rect);
                d
            }));
        merge_line(&mut line, &piece.lines, at, text.len(), baseline);
        out.size.height = out.size.height.max(piece.size.height + dy);
        x += advance;
        scalars += text[range].chars().count() as u32;
    }
    out.cluster_boundaries.sort_unstable();
    out.cluster_boundaries.dedup();
    out.size.width = x;
    out.lines.extend(line);
    Some(out.into())
}

fn merge_line(
    line: &mut Option<kivact_types::render::TextLine>,
    lines: &[kivact_types::render::TextLine],
    at: impl Fn(Rect) -> Rect,
    text_len: usize,
    baseline: f64,
) {
    if let Some(first) = lines.first() {
        let rect = at(first.rect);
        *line = Some(match line.take() {
            None => kivact_types::render::TextLine {
                range: 0..text_len,
                rect,
                baseline,
                ..first.clone()
            },
            Some(mut l) => {
                l.rect = l.rect.union(rect);
                l.ascent = l.ascent.max(first.ascent);
                l.descent = l.descent.max(first.descent);
                l.cap_height = l.cap_height.max(first.cap_height);
                l.x_height = l.x_height.max(first.x_height);
                l
            }
        });
    }
}
