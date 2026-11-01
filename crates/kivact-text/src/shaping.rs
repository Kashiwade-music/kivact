//! Role: Lazy shaping, editing geometry, coloring and bounded run publication.

use crate::EditLines;
use crate::pieces::{floor, shape_long};
use crate::records::{Colored, Shown};
use kivact_editing::{LineColors, LineState, Lines, TextGeometry, caret_at};
use kivact_types::{Point, Rect};
use std::borrow::Cow;
use std::ops::Range;
use std::sync::Arc;

/// The text measured for an input's natural width: the lines starting in
/// its first this many bytes (at least its first line).
const MEASURED: usize = 16 * 1024;

/// Lines published around the caret and the lines drawn, on each side,
/// and the step the published lines start and end at.
const PUBLISHED_MARGIN: usize = 32;

const PUBLISHED_STEP: usize = 32;

/// The text runs published of a text input's lines: from scalar `start`
/// of its text to scalar `end`.
pub struct Published {
    /// First Unicode scalar covered by the returned runs.
    pub start: u32,
    /// Exclusive final Unicode scalar covered by the returned runs.
    pub end: u32,
    /// Owned shared runs, retaining their font-independent geometry and values.
    /// Allocation identity is not a publication equality contract.
    pub runs: Arc<[Arc<kivact_types::SemanticsTextRun>]>,
}

/// An input's lines as a behavior reads them, shaping the lines it asks
/// about.
pub struct Shaping<'a> {
    lines: std::cell::RefCell<(&'a mut EditLines, &'a mut arvane::Engine)>,
    revision: u64,
}

/// The text a line shows: bullets for masked text, spaces for line
/// breaks and CRs (one byte each, so offsets stay).
fn display(text: &str, masked: bool) -> Cow<'_, str> {
    if masked {
        Cow::Owned(
            text.chars()
                .map(|c| if c == '\n' { ' ' } else { '•' })
                .collect(),
        )
    } else if text.contains(['\n', '\r']) {
        Cow::Owned(text.replace(['\n', '\r'], " "))
    } else {
        Cow::Borrowed(text)
    }
}

impl EditLines {
    /// Line `i` shaped from `text`, if it is not yet.
    pub(crate) fn ensure(&mut self, text: &str, i: usize, engine: &mut arvane::Engine) {
        let Some((line, _)) = self.line(i) else {
            return;
        };
        if line.shaped.is_some() {
            return;
        }
        let key = self.key().clone();
        let shown = self.shown_text(text, i);
        let shaped = shape_long(&shown, |r| engine.shape(&shown[r], &key.style, None).ok());
        let width = shaped.as_ref().map_or(0.0, |s| s.size.width);
        let (c, j) = self.locate(i);
        let chunk = &mut self.chunks[c];
        let line = &mut chunk.lines[j];
        line.shaped = Some(shaped);
        line.width = width;
        chunk.width = chunk.width.max(width);
        self.width = self.width.max(width);
    }

    /// What line `i` of `text` shows, as it is shaped (see [`display`]);
    /// empty past the last line.
    fn shown_text<'t>(&self, text: &'t str, i: usize) -> Cow<'t, str> {
        let Some((line, base)) = self.line(i) else {
            return Cow::Borrowed("");
        };
        let masked = self.key().shown == Shown::Masked;
        display(&text[base + line.start..base + line.content], masked)
    }

    /// How far apart lines are.
    #[inline]
    pub fn height(&self) -> f32 {
        self.height
    }

    /// The widest shaped line's width.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// The natural width of the lines: the widest of the lines that
    /// start in the first 16 KiB bytes of `text` (shaped now).
    pub fn measured_width(&mut self, text: &str, engine: &mut arvane::Engine) -> f32 {
        let mut width = 0.0f32;
        let mut i = 0;
        while i < self.len() && (i == 0 || self.range(i).start < MEASURED) {
            self.ensure(text, i, engine);
            width = width.max(self.line(i).map_or(0.0, |(l, _)| l.width));
            i += 1;
        }
        width
    }

    /// Line `i`'s baseline below its top (a line without text has a
    /// space's).
    pub fn baseline(&mut self, text: &str, i: usize, engine: &mut arvane::Engine) -> f32 {
        if self.line(i).is_none_or(|(l, _)| l.content == l.start) {
            return self.blank;
        }
        self.shaped(text, i, engine)
            .map_or(self.blank, |s| s.baseline)
    }

    /// Lazily shape line `i`, returning None if missing or unshapable.
    /// The owned Arc retains font bytes and geometry beyond engine cache resets.
    pub fn shaped(
        &mut self,
        text: &str,
        i: usize,
        engine: &mut arvane::Engine,
    ) -> Option<Arc<kivact_types::protocol::ShapedText>> {
        self.ensure(text, i, engine);
        self.line(i)?.0.shaped.clone().flatten()
    }

    /// How far line `i` of `text` starts into a box `width` wide (its
    /// alignment).
    #[inline]
    pub fn offset(&mut self, text: &str, i: usize, width: f32, engine: &mut arvane::Engine) -> f32 {
        if matches!(
            self.align,
            kivact_types::TextAlign::Start | kivact_types::TextAlign::Justify
        ) {
            return 0.0;
        }
        self.ensure(text, i, engine);
        self.aligned(self.line(i).map_or(0.0, |(l, _)| l.width), width)
    }

    /// How far a line `line` wide starts into a box `width` wide.
    pub(crate) fn aligned(&self, line: f32, width: f32) -> f32 {
        let free = (width - line).max(0.0);
        match self.align {
            kivact_types::TextAlign::Center => free / 2.0,
            kivact_types::TextAlign::End => free,
            kivact_types::TextAlign::Start | kivact_types::TextAlign::Justify => 0.0,
        }
    }

    /// The lines that reach the band from `top` to `bottom` below the
    /// first line's top.
    pub fn rows(&self, top: f32, bottom: f32) -> Range<usize> {
        let h = self.height.max(f32::EPSILON);
        let len = self.len();
        let first = ((top / h).floor().max(0.0) as usize).min(len);
        let last = ((bottom / h).ceil().max(0.0) as usize).min(len);
        first..last.max(first)
    }

    /// The lines `rows` were drawn (published with the caret's).
    pub fn drawn(&mut self, rows: Range<usize>) {
        self.drawn = rows;
    }

    /// The scalar offset of byte offset `at` in line `i`'s text (a break
    /// at its end).
    pub(crate) fn scalar(&self, text: &str, i: usize, at: usize) -> usize {
        let Some((l, base)) = self.line(i) else {
            return 0;
        };
        let at = floor(text, at.clamp(base + l.start, base + l.content));
        text[base + l.start..at].chars().count()
    }

    /// The caret at byte offset `at` of `text` in a box `width` wide: from
    /// the start of the box and the first line's top.
    /// An empty value (the placeholder shown) has its caret where an
    /// empty line's would be.
    pub fn caret(
        &mut self,
        text: &str,
        at: usize,
        width: f32,
        engine: &mut arvane::Engine,
    ) -> Rect {
        if !self.shows_value() {
            return Rect {
                x: self.aligned(0.0, width),
                y: 0.0,
                width: 1.0,
                height: self.height,
            };
        }
        let i = self.line_of(at);
        let y = i as f32 * self.height;
        let x = self.offset(text, i, width, engine);
        let scalar = self.scalar(text, i, at);
        match self
            .shaped(text, i, engine)
            .filter(|s| !s.carets.is_empty())
        {
            Some(shaped) => {
                let c = caret_at(&shaped, scalar);
                Rect {
                    x: x + c.x,
                    y: y + c.y,
                    ..c
                }
            }
            None => Rect {
                x,
                y,
                width: 1.0,
                height: self.height,
            },
        }
    }

    /// The bands of line `i` that the bytes from `from` to `to` (of its
    /// text, clamped) cover: what a selection or a composition underline
    /// shows on the line, in a box `width` wide.
    pub fn spans(
        &mut self,
        text: &str,
        i: usize,
        (from, to): (usize, usize),
        width: f32,
        engine: &mut arvane::Engine,
    ) -> Vec<Rect> {
        if !self.shows_value() {
            return Vec::new();
        }
        let (a, b) = (self.scalar(text, i, from), self.scalar(text, i, to));
        let (x, y) = (self.offset(text, i, width, engine), i as f32 * self.height);
        let Some(shaped) = self.shaped(text, i, engine).filter(|_| a < b) else {
            return Vec::new();
        };
        let shown = self.shown_text(text, i);
        TextGeometry::new(&shaped, &shown)
            .rects_scalar(a..b)
            .into_iter()
            .map(|r| Rect {
                x: x + r.x,
                y: y + r.y,
                ..r
            })
            .collect()
    }

    /// The byte offset on line `i` at the grapheme boundary whose caret is
    /// nearest `x`, in a box `width` wide.
    pub fn nearest(
        &mut self,
        text: &str,
        i: usize,
        x: f32,
        width: f32,
        engine: &mut arvane::Engine,
    ) -> usize {
        if !self.shows_value() {
            return 0;
        }
        let content = self.content(i).unwrap_or(0..0);
        let Some(shaped) = self.shaped(text, i, engine) else {
            return content.start;
        };
        let x = x - self.offset(text, i, width, engine);
        let line = &text[content.clone()];
        let bounds = shaped
            .lines
            .first()
            .map_or_else(Rect::default, |line| line.rect);
        let scalar = TextGeometry::new(&shaped, line).hit_scalar_in_line(
            Point { x, y: bounds.y },
            0..line.len(),
            bounds,
        );
        let at = line
            .char_indices()
            .map(|(at, _)| at)
            .nth(scalar)
            .unwrap_or(line.len());
        content.start + at
    }

    /// The byte offset nearest the point `(x, y)` (from the box's start
    /// and the first line's top), on a grapheme boundary.
    pub fn offset_at(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        width: f32,
        engine: &mut arvane::Engine,
    ) -> usize {
        let i = ((y / self.height.max(f32::EPSILON)).floor().max(0.0) as usize)
            .min(self.len().saturating_sub(1));
        self.nearest(text, i, x, width, engine)
    }

    /// Color the lines up to `upto` with `colors` (see the module
    /// invariants).
    pub fn color(&mut self, text: &str, colors: &LineColors, upto: usize) {
        if self.colors.as_ref() != Some(colors) {
            self.colors = Some(colors.clone());
            self.forget_colors();
        }
        while self.colored < upto.min(self.len()) {
            let i = self.colored;
            let input = match i {
                0 => LineState::default(),
                _ => self
                    .line(i - 1)
                    .and_then(|(l, _)| l.colors())
                    .map(|c| c.output.clone())
                    .unwrap_or_default(),
            };
            let content = self.content(i).unwrap_or(0..0);
            let line = self.line_mut(i).expect("invariant: colored lines exist");
            if line.colors().is_none_or(|c| c.input != input) {
                let (spans, output) = colors.line(&text[content], &input);
                let kept = line.kept();
                if kept.colors.as_ref().is_none_or(|c| c.spans != spans) {
                    kept.painted = None;
                }
                kept.colors = Some(Colored {
                    input,
                    spans,
                    output,
                });
            }
            self.colored += 1;
        }
    }

    /// Forget the colors (the input has none).
    pub fn uncolor(&mut self) {
        if self.colors.take().is_some() {
            self.forget_colors();
        }
    }

    pub(crate) fn forget_colors(&mut self) {
        for chunk in &mut self.chunks {
            for line in &mut chunk.lines {
                line.uncolor();
            }
        }
        self.colored = 0;
    }

    /// Line `i` shaped with its colors over the text color `color`
    /// (`None`: it has none; draw it as shaped).
    pub fn painted(
        &mut self,
        text: &str,
        i: usize,
        color: kivact_types::Rgba,
        engine: &mut arvane::Engine,
    ) -> Option<Arc<kivact_types::protocol::ShapedText>> {
        let content = self.content(i)?;
        let mut style = self.key().style.clone();
        style.color = color;
        let kept = self.line_mut(i)?.kept.as_deref_mut()?;
        let colored = kept.colors.as_ref().filter(|c| !c.spans.is_empty())?;
        if let Some((with, shaped)) = &kept.painted
            && *with == color
        {
            return Some(shaped.clone());
        }
        let shown = display(&text[content], false);
        let shaped = shape_long(&shown, |r| {
            // The spans of the piece `r`, from its start.
            let spans: Vec<kivact_types::protocol::TextSpan> = colored
                .spans
                .iter()
                .filter(|(range, _)| range.start < r.end && range.end > r.start)
                .map(|(range, color)| kivact_types::protocol::TextSpan {
                    range: range.start.max(r.start) - r.start..range.end.min(r.end) - r.start,
                    style: kivact_types::protocol::SpanStyle {
                        color: Some(kivact_lower::sanitize::color(*color)),
                        ..Default::default()
                    },
                })
                .collect();
            let rich = kivact_types::protocol::RichText {
                text: &shown[r],
                spans: &spans,
            };
            engine.shape_glyphs(rich, &style, None).ok()
        })?;
        kept.painted = Some((color, shaped.clone()));
        Some(shaped)
    }

    /// The identity of line `i`'s text run, a new one from `ids` if it has
    /// none yet.
    /// `ids` must allocate fresh nonzero IDs, collision-free throughout the
    /// caller's publication document/tree (not the whole process or OS). Zero
    /// is the unassigned sentinel; never reuse an ID while it is addressable.
    pub fn run_id(&mut self, i: usize, ids: &mut dyn FnMut() -> u64) -> u64 {
        let Some(line) = self.line_mut(i) else {
            return ids();
        };
        let kept = line.kept();
        if kept.run_id == 0 {
            kept.run_id = ids();
        }
        kept.run_id
    }

    /// The lines published to assistive technology: those around the
    /// line holding byte `caret` and, while it is near them, the lines
    /// last drawn, [`PUBLISHED_MARGIN`] more on each side, from and to a
    /// multiple of [`PUBLISHED_STEP`].
    pub(crate) fn published_lines(&self, caret: usize) -> Range<usize> {
        let line = self.line_of(caret);
        let len = self.len();
        let drawn = self.drawn.start.min(len)..self.drawn.end.min(len);
        let near = !drawn.is_empty()
            && line + PUBLISHED_MARGIN >= drawn.start
            && line < drawn.end + PUBLISHED_MARGIN;
        let (from, to) = match near {
            true => (drawn.start.min(line), drawn.end.max(line + 1)),
            false => (line, line + 1),
        };
        let from = from.saturating_sub(PUBLISHED_MARGIN) / PUBLISHED_STEP * PUBLISHED_STEP;
        let to = ((to + PUBLISHED_MARGIN).div_ceil(PUBLISHED_STEP) * PUBLISHED_STEP).min(len);
        from..to.max(from + 1).min(len)
    }

    /// The text runs of the lines published for the caret at byte `caret`
    /// (the bounded publication window), the first line's top left at `at`
    /// in a box `width` wide. A line keeps its run while it does not
    /// change or move; a new line's run takes a new identity from `ids`.
    /// Runs contain original text, including for visually masked passwords;
    /// callers own redaction/protected-publication policy. `ids` must allocate
    /// fresh nonzero IDs unique in the caller's publication document/tree,
    /// never reused while addressable. Detached copies may own separate domains.
    pub fn published(
        &mut self,
        text: &str,
        caret: usize,
        at: Point,
        width: f32,
        ids: &mut dyn FnMut() -> u64,
        engine: &mut arvane::Engine,
    ) -> Published {
        let lines = self.published_lines(caret);
        let runs = lines
            .clone()
            .map(|i| self.run(text, i, at, width, ids, engine))
            .collect();
        Published {
            start: self.char_of_line(lines.start),
            end: self.char_of_line(lines.end),
            runs,
        }
    }

    /// Line `i`'s text run, its first line's top left at `at` in a box
    /// `width` wide (kept until the line changes or moves).
    pub(crate) fn run(
        &mut self,
        text: &str,
        i: usize,
        at: Point,
        width: f32,
        ids: &mut dyn FnMut() -> u64,
        engine: &mut arvane::Engine,
    ) -> Arc<kivact_types::SemanticsTextRun> {
        let origin = Point {
            x: at.x + self.offset(text, i, width, engine),
            y: at.y + i as f32 * self.height,
        };
        let height = self.height;
        let id = self.run_id(i, ids);
        let shaped = self.shaped(text, i, engine);
        let range = self.range(i);
        let kept = self
            .line_mut(i)
            .expect("invariant: published lines exist")
            .kept();
        if let Some((placed, run)) = &kept.run
            && *placed == origin
        {
            return run.clone();
        }
        let shift = |r: Rect| Rect {
            x: r.x + origin.x,
            y: r.y + origin.y,
            ..r
        };
        let empty = Rect {
            height,
            ..Rect::default()
        };
        let (bounds, carets) = match &shaped {
            Some(shaped) => (
                shift(shaped.lines.first().map_or(empty, |l| l.rect)),
                shaped
                    .carets
                    .iter()
                    .map(|c| kivact_types::TextCaret {
                        offset: c.offset,
                        rect: shift(c.rect),
                    })
                    .collect(),
            ),
            None => (shift(empty), Vec::new()),
        };
        let run = Arc::new(kivact_types::SemanticsTextRun {
            id: kivact_types::NodeId(id),
            value: text[range].to_string(),
            bounds,
            carets,
        });
        kept.run = Some((origin, run.clone()));
        run
    }
}

impl<'a> Shaping<'a> {
    /// Lend exclusive cache and shaper access for synchronous editing queries.
    /// Queries may lazily shape lines; the captured revision stays fixed.
    pub fn new(lines: &'a mut EditLines, engine: &'a mut arvane::Engine) -> Self {
        Shaping {
            revision: lines.revision(),
            lines: std::cell::RefCell::new((lines, engine)),
        }
    }

    /// The revision of the text the lines are of.
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

impl Lines for Shaping<'_> {
    fn shows_value(&self) -> bool {
        self.lines.borrow().0.shows_value()
    }
    fn len(&self) -> usize {
        self.lines.borrow().0.len()
    }
    fn line_of(&self, at: usize) -> usize {
        self.lines.borrow().0.line_of(at)
    }
    fn content(&self, i: usize) -> Option<Range<usize>> {
        self.lines.borrow().0.content(i)
    }
    fn caret(&self, text: &str, at: usize, width: f32) -> Rect {
        let (lines, engine) = &mut *self.lines.borrow_mut();
        lines.caret(text, at, width, engine)
    }
    fn nearest(&self, text: &str, i: usize, x: f32, width: f32) -> usize {
        let (lines, engine) = &mut *self.lines.borrow_mut();
        lines.nearest(text, i, x, width, engine)
    }
}
