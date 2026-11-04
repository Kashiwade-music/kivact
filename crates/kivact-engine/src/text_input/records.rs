//! Role: Passive line, chunk, cache-key and prefix records.

use crate::text_input::source::Source;
use kivact_editing::LineState;
use kivact_types::{Point, Rgba};
use std::ops::Range;
use std::sync::Arc;

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
    pub(crate) style: kivact_types::ComputedTextStyle,
    pub(crate) shown: Shown,
    pub(crate) multiline: bool,
    pub(crate) wrap_width: Option<f64>,
}

impl Key {
    /// What `source` in `style` is shaped for (start aligned, without a color), and the
    /// alignment its lines are moved by.
    pub(crate) fn of(
        source: &Source<'_>,
        mut style: kivact_types::ComputedTextStyle,
    ) -> (Key, kivact_types::TextAlign) {
        let shown = if source.value.is_empty() {
            Shown::Placeholder(source.placeholder.to_owned())
        } else if source.password {
            Shown::Masked
        } else {
            Shown::Value
        };
        // Shaped start aligned and without a color: aligning and coloring
        // (a hover, a focus style) never shape again.
        let align = style.align;
        style.align = kivact_types::TextAlign::Start;
        style.color = kivact_types::ComputedTextStyle::default().color;
        let key = Key {
            style,
            shown,
            multiline: source.multiline,
            wrap_width: source.wrap_width,
        };
        (key, align)
    }
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
    pub(crate) painted: Option<(kivact_types::Rgba, Arc<kivact_types::render::ShapedText>)>,
    /// Its text run's identity, once it has one.
    pub(crate) run_id: Option<u64>,
    /// Its text run and where it was placed.
    pub(crate) run: Option<(Point, Arc<kivact_types::SemanticsTextRun>)>,
}

/// What became of shaping a line.
#[derive(Clone, Debug)]
pub(crate) enum Shape {
    /// Not shaped yet.
    Pending,
    /// The shaper could not shape it: it is laid out as an empty line.
    Failed,
    /// Shaped.
    Done(Arc<kivact_types::render::ShapedText>),
}

impl Shape {
    /// Its shaped width (0 until shaped, and when shaping failed).
    pub(crate) fn width(&self) -> f64 {
        match self {
            Shape::Done(shaped) => shaped.size.width,
            Shape::Pending | Shape::Failed => 0.0,
        }
    }

    /// Its shaped text, when shaped.
    pub(crate) fn shaped(&self) -> Option<&Arc<kivact_types::render::ShapedText>> {
        match self {
            Shape::Done(shaped) => Some(shaped),
            Shape::Pending | Shape::Failed => None,
        }
    }
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
    /// Its text, shaped as it was first needed.
    pub(crate) shape: Shape,
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
            shape: Shape::Pending,
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
    pub(crate) width: f64,
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
        self.width = self
            .lines
            .iter()
            .map(|l| l.shape.width())
            .fold(0.0, f64::max);
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
