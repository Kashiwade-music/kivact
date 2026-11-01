//! Role: Passive line, chunk, cache-key and prefix records.

use kivact_editing::LineState;
use std::ops::Range;
use std::sync::Arc;
use ui_types::{Point, Rgba};

/// What the lines show.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Shown {
    Value,
    /// The value, one bullet per scalar.
    Masked,
    /// The placeholder (the value is empty).
    Placeholder(String),
}

/// What the lines were shaped for: the style (start aligned), what they
/// show and whether they break at line feeds.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Key {
    pub(crate) style: ui_types::TextStyle,
    pub(crate) shown: Shown,
    pub(crate) multiline: bool,
}

/// One line's colors and the states it was colored from and leaves.
#[derive(Clone, Debug)]
pub(crate) struct Colored {
    pub(crate) input: LineState,
    pub(crate) spans: Vec<(Range<usize>, Rgba)>,
    pub(crate) output: LineState,
}

/// What a line keeps of its colors and of assistive technology.
#[derive(Clone, Debug, Default)]
pub(crate) struct Kept {
    pub(crate) colors: Option<Colored>,
    /// Its glyphs shaped with its colors, over the text color they were
    /// shaped with.
    pub(crate) painted: Option<(ui_types::Rgba, Arc<ui_types::ShapedText>)>,
    /// Its text run's identity (0: none yet).
    pub(crate) run_id: u64,
    /// Its text run and where it was placed.
    pub(crate) run: Option<(Point, Arc<ui_types::SemanticsTextRun>)>,
}

/// One hard line: its byte range from its chunk's start, its counts and
/// what is kept for it.
#[derive(Clone, Debug)]
pub(crate) struct Line {
    pub(crate) start: usize,
    /// Where its text ends (before its break).
    pub(crate) content: usize,
    /// Where it ends (after its break).
    pub(crate) end: usize,
    /// Scalars and UTF-16 units of `start..end`.
    pub(crate) chars: u32,
    pub(crate) units: u32,
    /// Its text, shaped (`None`: not yet; `Some(None)`: it cannot be).
    pub(crate) shaped: Option<Option<Arc<ui_types::ShapedText>>>,
    /// Its shaped width (0 until shaped).
    pub(crate) width: f32,
    pub(crate) kept: Option<Box<Kept>>,
}

impl Line {
    /// The line `(start, content, end)` of `text`, its offsets from `base`.
    pub(crate) fn new(
        text: &str,
        (start, content, end): (usize, usize, usize),
        base: usize,
    ) -> Self {
        let all = &text[start..end];
        Line {
            start: start - base,
            content: content - base,
            end: end - base,
            chars: all.chars().count() as u32,
            units: all.encode_utf16().count() as u32,
            shaped: None,
            width: 0.0,
            kept: None,
        }
    }

    pub(crate) fn kept(&mut self) -> &mut Kept {
        self.kept.get_or_insert_with(Default::default)
    }

    pub(crate) fn colors(&self) -> Option<&Colored> {
        self.kept.as_ref()?.colors.as_ref()
    }

    /// Forget its colors and its colored glyphs.
    pub(crate) fn uncolor(&mut self) {
        if let Some(kept) = &mut self.kept {
            kept.colors = None;
            kept.painted = None;
        }
    }

    /// Move its offsets by `by`.
    pub(crate) fn shift(&mut self, by: isize) {
        for at in [&mut self.start, &mut self.content, &mut self.end] {
            *at = (*at as isize + by) as usize;
        }
    }
}

/// Lines in a row, their offsets from the chunk's start.
#[derive(Clone, Debug, Default)]
pub(crate) struct Chunk {
    pub(crate) lines: Vec<Line>,
    /// Its bytes, scalars and UTF-16 units, and its widest shaped line.
    pub(crate) bytes: usize,
    pub(crate) chars: usize,
    pub(crate) units: usize,
    pub(crate) width: f32,
}

impl Chunk {
    pub(crate) fn new(lines: Vec<Line>) -> Self {
        let mut chunk = Chunk {
            lines,
            ..Chunk::default()
        };
        chunk.recount();
        chunk
    }

    pub(crate) fn recount(&mut self) {
        self.bytes = self.lines.last().map_or(0, |l| l.end);
        self.chars = self.lines.iter().map(|l| l.chars as usize).sum();
        self.units = self.lines.iter().map(|l| l.units as usize).sum();
        self.width = self.lines.iter().map(|l| l.width).fold(0.0, f32::max);
    }
}

/// What comes before a chunk: lines, bytes, scalars and UTF-16 units.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Before {
    pub(crate) line: usize,
    pub(crate) byte: usize,
    pub(crate) chars: usize,
    pub(crate) units: usize,
}
