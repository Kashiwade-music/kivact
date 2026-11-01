//! Role: Bounded IME excerpts cut at grapheme and CRLF boundaries.

use crate::pieces::floor;
use crate::{EditLines, IME_CONTEXT};
use std::ops::Range;
use unicode_segmentation::GraphemeCursor;

impl EditLines {
    /// The part of the text the IME is given around `around` (byte
    /// offsets): the lines around it while they fit in 2048
    /// bytes on each side, else the part of its own lines that does, cut
    /// at grapheme boundaries (never between a CR and its LF).
    /// The range addresses original, unredacted UTF-8 text even for passwords.
    /// The caller must suppress protected surrounding text where required.
    pub fn excerpt(&self, text: &str, around: Range<usize>) -> Range<usize> {
        if !self.shows_value() {
            return 0..0;
        }
        let (mut i, mut j) = (self.line_of(around.start), self.line_of(around.end));
        let mut from = self.range(i).start;
        if around.start - from > IME_CONTEXT {
            from = grapheme_ceil(text, around.start - IME_CONTEXT).min(around.start);
        } else {
            while i > 0 && around.start - self.range(i - 1).start <= IME_CONTEXT {
                i -= 1;
                from = self.range(i).start;
            }
        }
        let mut to = self.range(j).end;
        if to - around.end > IME_CONTEXT {
            to = grapheme_floor(text, around.end + IME_CONTEXT).max(around.end);
        } else {
            while j + 1 < self.len() && self.range(j + 1).end - around.end <= IME_CONTEXT {
                j += 1;
                to = self.range(j).end;
            }
        }
        from..to
    }
}

/// The smallest grapheme boundary of `s` not before `x`.
fn grapheme_ceil(s: &str, x: usize) -> usize {
    let mut x = x.min(s.len());
    while !s.is_char_boundary(x) {
        x += 1;
    }
    let mut cursor = GraphemeCursor::new(x, s.len(), true);
    match cursor.is_boundary(s, 0) {
        Ok(true) => x,
        _ => cursor.next_boundary(s, 0).ok().flatten().unwrap_or(s.len()),
    }
}

/// The largest grapheme boundary of `s` not after `x`.
fn grapheme_floor(s: &str, x: usize) -> usize {
    let x = floor(s, x);
    let mut cursor = GraphemeCursor::new(x, s.len(), true);
    match cursor.is_boundary(s, 0) {
        Ok(true) => x,
        _ => cursor.prev_boundary(s, 0).ok().flatten().unwrap_or(0),
    }
}
