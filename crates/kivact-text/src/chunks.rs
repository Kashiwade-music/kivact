//! Role: Hard-line splits, chunk reconciliation and byte/scalar/UTF16 lookup.

use crate::pieces::floor;
use crate::records::{Before, Chunk, Line, Shown};
use crate::{CHUNK, EditLines};
use kivact_editing::Damage;
use std::ops::Range;

/// The lines of `text[from..to]` as `(start, content, end)`: a whole text
/// is one line unless `multiline`; `to` is the text's end or follows a
/// line feed. The empty line after a final line feed is one of them only
/// when they reach the `last` line (else it follows them already).
pub(crate) fn split(
    text: &str,
    from: usize,
    to: usize,
    multiline: bool,
    last: bool,
) -> Vec<(usize, usize, usize)> {
    if !multiline {
        return vec![(from, to, to)];
    }
    let mut out = Vec::new();
    let mut at = from;
    for piece in text[from..to].split_inclusive('\n') {
        let end = at + piece.len();
        let content = if piece.ends_with("\r\n") {
            end - 2
        } else if piece.ends_with('\n') {
            end - 1
        } else {
            end
        };
        out.push((at, content, end));
        at = end;
    }
    if last && to == text.len() && (to == from || text[..to].ends_with('\n')) {
        out.push((to, to, to));
    }
    out
}

impl EditLines {
    /// Every line of `text` split again, none shaped.
    pub(crate) fn rebuild(&mut self, text: &str, height: f32, blank: f32) {
        let key = self.key().clone();
        self.height = height;
        self.blank = blank;
        let lines = split(text, 0, text.len(), key.multiline, true);
        self.chunks = lines
            .chunks(CHUNK)
            .map(|rows| {
                let base = rows[0].0;
                Chunk::new(rows.iter().map(|r| Line::new(text, *r, base)).collect())
            })
            .collect();
        self.restart(0);
        self.colored = 0;
        self.drawn = 0..0;
    }

    /// The lines `damage` touches split again from `text`, the later ones
    /// moved; `false` when the damage does not fit the lines.
    pub(crate) fn splice(&mut self, text: &str, damage: Damage) -> bool {
        let Damage {
            start,
            old_end,
            new_end,
            ..
        } = damage;
        let old_len = self.before.last().map_or(0, |b| b.byte);
        if start > old_end || old_end > old_len || new_end < start {
            return false;
        }
        let delta = new_end as isize - old_end as isize;
        if old_len as isize + delta != text.len() as isize {
            return false;
        }
        let k = self.line_of(start);
        let m = self.line_of(old_end).max(k);
        let from = self.range(k).start;
        let to = (self.range(m).end as isize + delta) as usize;
        let multiline = self.key().multiline;
        let bounded = from <= to
            && to <= text.len()
            && text.is_char_boundary(from)
            && text.is_char_boundary(to)
            && (!multiline || to == text.len() || text.as_bytes()[to - 1] == b'\n');
        if !bounded {
            return false;
        }
        let ((ck, jk), (cm, jm)) = (self.locate(k), self.locate(m));
        let base = self.before[ck].byte;
        // The lines of chunks `ck..=cm` before `k`, the new lines, and the
        // lines after `m` moved: one chunk from `base`, cut if too long.
        let after = (self.before[cm].byte - base) as isize + delta;
        let mut tail: Vec<Line> = self.chunks[cm].lines.drain(jm + 1..).collect();
        for line in &mut tail {
            line.shift(after);
        }
        let mut lines: Vec<Line> = std::mem::take(&mut self.chunks[ck].lines);
        lines.truncate(jk);
        lines.extend(
            split(text, from, to, multiline, m + 1 == self.len())
                .into_iter()
                .map(|r| Line::new(text, r, base)),
        );
        lines.extend(tail);
        let mut made: Vec<Chunk> = Vec::new();
        let mut rest = lines;
        while rest.len() > 2 * CHUNK {
            let more = rest.split_off(CHUNK);
            made.push(Chunk::new(rest));
            rest = more;
        }
        made.push(Chunk::new(rest));
        for chunk in made.iter_mut().skip(1) {
            let by = chunk.lines[0].start as isize;
            for line in &mut chunk.lines {
                line.shift(-by);
            }
            chunk.recount();
        }
        let count = made.len();
        self.chunks.splice(ck..=cm, made);
        // A chunk left short joins the next one.
        let last = ck + count - 1;
        if self.chunks[last].lines.len() < CHUNK / 4
            && self
                .chunks
                .get(last + 1)
                .is_some_and(|next| next.lines.len() <= CHUNK)
        {
            let next = self.chunks.remove(last + 1);
            let by = self.chunks[last].bytes as isize;
            let chunk = &mut self.chunks[last];
            chunk.lines.extend(next.lines.into_iter().map(|mut l| {
                l.shift(by);
                l
            }));
            chunk.recount();
        }
        self.chunks.retain(|c| !c.lines.is_empty());
        self.restart(ck);
        self.colored = self.colored.min(k);
        true
    }

    /// What comes before every chunk from `from` on, and the widest line.
    pub(crate) fn restart(&mut self, from: usize) {
        self.before.truncate(from + 1);
        for chunk in &self.chunks[self.before.len() - 1..] {
            let b = self.before[self.before.len() - 1];
            self.before.push(Before {
                line: b.line + chunk.lines.len(),
                byte: b.byte + chunk.bytes,
                chars: b.chars + chunk.chars,
                units: b.units + chunk.units,
            });
        }
        self.width = self.chunks.iter().map(|c| c.width).fold(0.0, f32::max);
    }

    /// The chunk holding line `i` and the line's place in it.
    pub(crate) fn locate(&self, i: usize) -> (usize, usize) {
        let c = self
            .before
            .partition_point(|b| b.line <= i)
            .saturating_sub(1)
            .min(self.chunks.len().saturating_sub(1));
        (c, i - self.before[c].line)
    }

    pub(crate) fn line(&self, i: usize) -> Option<(&Line, usize)> {
        let (c, j) = self.locate(i);
        Some((self.chunks.get(c)?.lines.get(j)?, self.before[c].byte))
    }

    pub(crate) fn line_mut(&mut self, i: usize) -> Option<&mut Line> {
        let (c, j) = self.locate(i);
        self.chunks.get_mut(c)?.lines.get_mut(j)
    }

    /// The revision of the text the lines are of.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The lines show the value (masked or not), not the placeholder.
    pub fn shows_value(&self) -> bool {
        self.key
            .as_ref()
            .is_some_and(|k| !matches!(k.shown, Shown::Placeholder(_)))
    }

    /// The number of lines. This is zero before the first refresh and at least one afterward.
    #[inline]
    pub fn len(&self) -> usize {
        self.before.last().map_or(0, |b| b.line)
    }

    /// The line holding byte offset `at` (a break belongs to the line it
    /// ends).
    #[inline]
    pub fn line_of(&self, at: usize) -> usize {
        let c = self
            .before
            .partition_point(|b| b.byte <= at)
            .saturating_sub(1)
            .min(self.chunks.len().saturating_sub(1));
        let Some(chunk) = self.chunks.get(c) else {
            return 0;
        };
        let local = at - self.before[c].byte.min(at);
        self.before[c].line
            + chunk
                .lines
                .partition_point(|l| l.start <= local)
                .saturating_sub(1)
    }

    /// UTF-8 bytes of line `i`'s text, excluding LF/CRLF; None if absent.
    pub fn content(&self, i: usize) -> Option<Range<usize>> {
        self.line(i)
            .map(|(l, base)| base + l.start..base + l.content)
    }

    /// UTF-8 bytes of line `i`, LF/CRLF included; 0..0 if absent.
    pub fn range(&self, i: usize) -> Range<usize> {
        self.line(i)
            .map_or(0..0, |(l, base)| base + l.start..base + l.end)
    }

    /// Whether line `i` ends with CRLF; false if the line is absent.
    pub fn crlf(&self, i: usize) -> bool {
        self.line(i).is_some_and(|(l, _)| l.end - l.content == 2)
    }

    /// The scalars and UTF-16 units before line `i`.
    pub(crate) fn counts_before(&self, i: usize) -> (usize, usize) {
        let (c, j) = self.locate(i);
        let b = self.before[c];
        self.chunks.get(c).map_or((b.chars, b.units), |chunk| {
            chunk.lines[..j.min(chunk.lines.len())]
                .iter()
                .fold((b.chars, b.units), |(c, u), l| {
                    (c + l.chars as usize, u + l.units as usize)
                })
        })
    }

    /// Unicode scalar offset of byte `at`, clamped/floored to a char boundary.
    /// Returns zero while displaying a placeholder.
    pub fn char_of_byte(&self, text: &str, at: usize) -> u32 {
        if !self.shows_value() {
            return 0;
        }
        let i = self.line_of(at);
        let range = self.range(i);
        let at = floor(text, at.clamp(range.start, range.end));
        (self.counts_before(i).0 + text[range.start..at].chars().count()) as u32
    }

    /// UTF-16 code-unit offset of byte `at`, clamped/floored to a char boundary.
    /// Surrogate pairs count as two; placeholders return zero.
    pub fn utf16_of_byte(&self, text: &str, at: usize) -> usize {
        if !self.shows_value() {
            return 0;
        }
        let i = self.line_of(at);
        let range = self.range(i);
        let at = floor(text, at.clamp(range.start, range.end));
        self.counts_before(i).1 + text[range.start..at].encode_utf16().count()
    }

    /// Unicode scalar offset where line `i` starts; terminal total if absent.
    pub fn char_of_line(&self, i: usize) -> u32 {
        if i >= self.len() {
            return self.before.last().map_or(0, |b| b.chars) as u32;
        }
        self.counts_before(i).0 as u32
    }
    /// Whether there are no retained lines (before the first refresh).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
