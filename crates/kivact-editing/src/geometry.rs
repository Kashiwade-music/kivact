//! Role: The one implementation of caret, hit and range geometry over shaped text.
//!
//! A [`TextGeometry`] reads a [`ShapedText`] and the text it was shaped from:
//! where the caret of an offset lies, the offset nearest a point, the bands a
//! range covers and the next or previous boundary a caret may move to. A
//! paragraph handle and the native editor's lines both answer through it.
//!
//! Invariants:
//! - Offsets are UTF-8 byte offsets into the text, clamped to the char
//!   boundary at or before them and to the text's length. The shaped carets
//!   are indexed by Unicode scalar, so every query converts a byte offset to
//!   a scalar index once (`scalar_of`); the scalar-indexed forms (`caret_at`,
//!   `hit_scalar`, `rects_scalar`) serve text that is shown with the same
//!   scalars but other bytes (masked text).
//! - Geometry is in the shaped text's own pixels, from the top left of its
//!   first line.
//! - Editing stops are the intersection of Unicode grapheme boundaries and
//!   explicit shaped cluster boundaries (scalar offsets). Hits and movement
//!   never split either kind; equal caret positions do not identify clusters.
//! - A caret lies at every scalar boundary, as the shaping gives them; a
//!   boundary that ends a soft-wrapped line is the start of the next one.
//!   Text that could not be shaped (no carets, no lines) answers the origin.

use std::ops::Range;

use kivact_types::render::ShapedText;
use kivact_types::{Point, Rect};
use unicode_segmentation::UnicodeSegmentation as _;

/// A shaped text and the text it was shaped from, read for geometry.
#[derive(Clone, Copy)]
pub struct TextGeometry<'a> {
    shaped: &'a ShapedText,
    text: &'a str,
}

/// The caret at scalar boundary `index` of `shaped`, or the last one when
/// the index is past them; the origin without carets.
pub fn caret_at(shaped: &ShapedText, index: usize) -> Rect {
    shaped
        .carets
        .get(index)
        .or(shaped.carets.last())
        .map_or_else(Rect::default, |caret| caret.rect)
}

impl<'a> TextGeometry<'a> {
    /// The geometry of `shaped`, which was shaped from `text` (or from a
    /// text of the same scalars: line ranges are in the bytes of `text`).
    pub fn new(shaped: &'a ShapedText, text: &'a str) -> Self {
        TextGeometry { shaped, text }
    }

    /// `offset` cut to the text and to the char boundary at or before it.
    pub fn floor(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    /// The scalar index of the char boundary at or before `offset`.
    pub fn scalar_of(&self, offset: usize) -> usize {
        self.text[..self.floor(offset)].chars().count()
    }

    /// The byte offset of scalar boundary `index` (the text's length past
    /// the last).
    pub fn byte_of(&self, index: usize) -> usize {
        self.text
            .char_indices()
            .map(|(at, _)| at)
            .nth(index)
            .unwrap_or(self.text.len())
    }

    /// The caret at byte `offset`: a thin rectangle the height of its line.
    pub fn caret(&self, offset: usize) -> Rect {
        caret_at(self.shaped, self.scalar_of(offset))
    }

    /// The byte offset of the grapheme boundary whose caret is nearest
    /// `point`.
    pub fn hit(&self, point: Point) -> usize {
        self.byte_of(self.hit_scalar(point))
    }

    /// The scalar index of the grapheme boundary whose caret is nearest
    /// `point`. A point above or below the text is on its first or last
    /// line, and one past a line's end gives that line's last caret.
    pub fn hit_scalar(&self, point: Point) -> usize {
        let lines = &self.shaped.lines;
        let Some(line) = lines
            .iter()
            .find(|line| point.y < line.rect.bottom())
            .or(lines.last())
        else {
            return 0;
        };
        self.hit_scalar_in_line(point, line.range.clone(), line.rect)
    }

    /// Hit a known original-text line against its shaped bounds. This keeps
    /// original grapheme stops when masked display bytes differ but scalar
    /// offsets agree. `range` is in this view's original UTF-8 text. Its
    /// ends clamp to preceding character boundaries; an empty or reversed
    /// range returns the clamped start as a scalar offset.
    pub fn hit_scalar_in_line(&self, point: Point, range: Range<usize>, bounds: Rect) -> usize {
        let start = self.floor(range.start);
        let end = self.floor(range.end).max(start);
        let from = self.scalar_of(start);
        if start == end {
            return from;
        }
        let mut best: Option<(usize, f64)> = None;
        let mut index = from;
        let stops = self.text[start..end]
            .grapheme_indices(true)
            .map(|(at, _)| at)
            .chain(std::iter::once(end - start));
        let mut chars = self.text[start..end].char_indices().peekable();
        for at in stops {
            while chars.next_if(|(c, _)| *c < at).is_some() {
                index += 1;
            }
            if !self.is_cluster_boundary(index) {
                continue;
            }
            let caret = caret_at(self.shaped, index);
            let middle = caret.y + caret.height / 2.0;
            // A boundary that ends a wrapped line is the next line's start.
            if middle < bounds.y || middle >= bounds.bottom() {
                continue;
            }
            let distance = (caret.x - point.x).abs();
            if best.is_none_or(|(_, nearest)| distance < nearest) {
                best = Some((index, distance));
            }
        }
        best.map_or(from, |(index, _)| index)
    }

    /// The union of bands between adjacent scalar carets in `range` (bytes),
    /// merging touching intervals on the same line. Mixed bidi ranges retain
    /// native editing's adjacent-caret approximation. Empty for an empty range.
    pub fn rects(&self, range: Range<usize>) -> Vec<Rect> {
        self.rects_scalar(self.scalar_of(range.start)..self.scalar_of(range.end))
    }

    /// [`rects`](Self::rects) for the scalars `range`.
    pub fn rects_scalar(&self, range: Range<usize>) -> Vec<Rect> {
        let lines = &self.shaped.lines;
        let mut bands = Vec::new();
        // The lines tile the text in order: count the scalars between their
        // ends once.
        let (mut cursor, mut scalars) = (0, 0);
        for (i, line) in lines.iter().enumerate() {
            let start = self.floor(line.range.start).max(cursor);
            let end = self.floor(line.range.end).max(start);
            let line_start = scalars + self.text[cursor..start].chars().count();
            let line_end = line_start + self.text[start..end].chars().count();
            (cursor, scalars) = (end, line_end);
            let (from, to) = (range.start.max(line_start), range.end.min(line_end));
            if from >= to {
                continue;
            }
            let mut intervals = Vec::with_capacity(to - from);
            for index in from..to {
                let left = caret_at(self.shaped, index).x;
                let right = if index + 1 == line_end && i + 1 < lines.len() {
                    line.rect.right()
                } else {
                    caret_at(self.shaped, index + 1).x
                };
                intervals.push((left.min(right), left.max(right)));
            }
            intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut merged: Option<(f64, f64)> = None;
            for (left, right) in intervals {
                if let Some((start, end)) = merged.as_mut() {
                    if left <= *end {
                        *end = end.max(right);
                        continue;
                    }
                    bands.push(Rect {
                        x: *start,
                        y: line.rect.y,
                        width: *end - *start,
                        height: line.rect.height,
                    });
                }
                merged = Some((left, right));
            }
            if let Some((left, right)) = merged {
                bands.push(Rect {
                    x: left,
                    y: line.rect.y,
                    width: right - left,
                    height: line.rect.height,
                });
            }
        }
        bands
    }

    /// The next grapheme and shaping-cluster boundary, or the text's end.
    pub fn next_boundary(&self, offset: usize) -> usize {
        let at = self.floor(offset);
        self.boundaries()
            .find(|&byte| byte > at)
            .unwrap_or(self.text.len())
    }

    /// The previous grapheme and shaping-cluster boundary, or zero.
    pub fn prev_boundary(&self, offset: usize) -> usize {
        let at = self.floor(offset);
        self.boundaries()
            .take_while(|&byte| byte < at)
            .last()
            .unwrap_or(0)
    }

    fn boundaries(&self) -> impl Iterator<Item = usize> + '_ {
        let mut scalar = 0;
        self.text
            .grapheme_indices(true)
            .map(move |(byte, grapheme)| {
                let index = scalar;
                scalar += grapheme.chars().count();
                (byte, index)
            })
            .chain(std::iter::once((
                self.text.len(),
                self.text.chars().count(),
            )))
            .filter(|&(_, index)| self.is_cluster_boundary(index))
            .map(|(byte, _)| byte)
    }

    /// Whether the scalar offset `index` is a stop. A shaped text without
    /// cluster boundaries (a host that lists none) stops at every grapheme
    /// boundary.
    fn is_cluster_boundary(&self, index: usize) -> bool {
        let boundaries = &self.shaped.cluster_boundaries;
        boundaries.is_empty() || boundaries.binary_search(&(index as u32)).is_ok()
    }
}
