//! Role: Shaped paragraphs: shaped once, queried for geometry, and drawn by `Draw`.
//!
//! A [`TextParagraph`] is text the host shaped in a style at a width: it
//! knows its size, its lines and where every caret lies, so a component
//! can place a caret, hit-test a pointer and paint a selection without a
//! text engine of its own, and [`Draw::paragraph`] draws exactly the glyphs
//! it measured.
//!
//! Invariants:
//! - Offsets are UTF-8 byte offsets into the paragraph's text, clamped to
//!   the char boundary at or before them and to the text's length.
//!   Geometry is in the paragraph's own logical pixels, from the top left
//!   of its first line.
//! - The host shapes equal inputs once: two paragraphs made from the same
//!   text, style and width share one shaping, and are distinct handles
//!   (`PartialEq` is the handle's identity).
//! - A caret lies at every scalar boundary, as the host's shaping gives
//!   them: a boundary that ends a soft-wrapped line is the start of the next
//!   one. A cluster (a ligature, a base with combining marks) gives its
//!   boundaries one position. Movement and hits use both Unicode grapheme
//!   and explicit shaping-cluster boundaries.
//! - Text the host could not shape is an empty paragraph: no size and no
//!   lines, and the geometry queries (caret, hit, rects, lines, size)
//!   answer the origin; `next_boundary` and `prev_boundary` still move by
//!   grapheme.
//! - The style's `align` and `baseline` do not apply: a paragraph is laid
//!   out from its start edge and placed by its origin.

use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use kivact_types::{self as p, DrawCmd, ShapedText, TextLine};

use super::TextFormat;
use super::recorder::Draw;
use crate::geometry::{Point, Rect, Size};

/// A shaped paragraph with positions for measured in-flow objects.
///
/// Draw the text with [`Draw::paragraph`] and position each object using
/// [`Self::boxes`]. This does not mount nodes or define text selection
/// across objects. The paragraph's caret APIs are unavailable.
#[derive(Clone, Debug)]
pub struct InlineParagraph {
    text: TextParagraph,
    boxes: Vec<p::InlineBoxPlacement>,
}

impl InlineParagraph {
    /// Convert a host shaping snapshot into a drawable paragraph and object positions.
    // xtask-allow(internal-contract): Core text shapers adapt engine output; applications use shape_paragraph or Draw layout.
    #[doc(hidden)]
    pub fn from_shaped(text: &str, shaped: p::ShapedInlineText, max_width: Option<f64>) -> Self {
        Self {
            text: TextParagraph::from_shaped(text, shaped.text, max_width),
            boxes: shaped.boxes,
        }
    }

    /// The glyph and line layout, drawable with [`Draw::paragraph`].
    pub fn paragraph(&self) -> &TextParagraph {
        &self.text
    }

    /// The objects' logical boxes in visual line order.
    pub fn boxes(&self) -> &[p::InlineBoxPlacement] {
        &self.boxes
    }
}

/// Text shaped by the host in a style, at a width.
///
/// Made by `Constraints::measure_paragraph` (in a measure function) or
/// [`Draw::layout_paragraph`] (in a draw closure), or
/// `primitive::shape_paragraph` outside drawing; all shape through the
/// window's own text engine, so the paragraph is what is drawn. It is a
/// cheap handle to clone and to keep across draws.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::draw::{Draw, TextFormat};
/// use kivact::geometry::{Point, Rect, Size};
///
/// // A line of text with the selected bytes highlighted.
/// fn selected(d: &mut Draw, text: &str, range: std::ops::Range<usize>) {
///     let paragraph = d.layout_paragraph(text, &TextFormat::default(), None);
///     for band in paragraph.rects(range) {
///         d.fill_rect(band, ColorToken::Selection);
///     }
///     d.paragraph(&paragraph, Point::new(0.0, 0.0), ColorToken::Text);
/// }
/// # let _ = selected;
/// ```
#[derive(Clone)]
pub struct TextParagraph(Rc<Shaped>);

struct Shaped {
    text: Rc<str>,
    shaped: Arc<ShapedText>,
    max_width: Option<f64>,
}

impl PartialEq for TextParagraph {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for TextParagraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextParagraph")
            .field("text", &self.0.text)
            .field("size", &self.size())
            .field("lines", &self.0.shaped.lines.len())
            .finish_non_exhaustive()
    }
}

impl TextParagraph {
    /// A paragraph of `text` from its `shaped` form and the `max_width` it was
    /// wrapped at. For hosts and component libraries that shape text
    /// themselves; the shaped text must come from the same text (carets and
    /// lines are in its offsets). Components shape through the window's
    /// engine instead: `Constraints::measure_paragraph`,
    /// `Draw::layout_paragraph` or `primitive::shape_paragraph`.
    ///
    /// An empty `cluster_boundaries` list in `shaped` means every grapheme
    /// boundary is a stop for hits and movement.
    // xtask-allow(internal-contract): Core text shapers adapt engine output; applications use shape_paragraph or Draw layout.
    #[doc(hidden)]
    pub fn from_shaped(
        text: impl Into<Rc<str>>,
        shaped: Arc<ShapedText>,
        max_width: Option<f64>,
    ) -> Self {
        TextParagraph(Rc::new(Shaped {
            text: text.into(),
            shaped,
            max_width,
        }))
    }

    /// The text, as it was shaped.
    pub fn text(&self) -> &str {
        &self.0.text
    }

    /// The width it was wrapped at (`None`: not wrapped).
    pub fn max_width(&self) -> Option<f64> {
        self.0.max_width
    }

    /// The paragraph's extent.
    pub fn size(&self) -> Size {
        self.0.shaped.size
    }

    /// The first line's baseline, from the top.
    pub fn baseline(&self) -> f64 {
        self.0.shaped.baseline
    }

    /// The lines shown, top to bottom (none for empty text). Each line's
    /// `range` is in bytes of [`text`](Self::text) and includes the white
    /// space and the line break that end it.
    pub fn lines(&self) -> &[TextLine] {
        &self.0.shaped.lines
    }

    /// The caret at byte `offset`: a thin rectangle the height of its
    /// line. At the end of a soft-wrapped line it is at the start of the next.
    pub fn caret(&self, offset: usize) -> Rect {
        self.geometry().caret(offset)
    }

    /// The nearest grapheme and shaping-cluster boundary to `point`.
    pub fn hit(&self, point: Point) -> usize {
        self.geometry().hit(point)
    }

    /// Selection bands covering adjacent scalar carets in the byte range.
    /// Touching bands on one line are merged, matching native editing.
    /// Mixed bidi selections retain the native adjacent-caret approximation.
    pub fn rects(&self, range: Range<usize>) -> Vec<Rect> {
        self.geometry().rects(range)
    }

    /// The next grapheme and shaping-cluster boundary, or the text's end.
    pub fn next_boundary(&self, offset: usize) -> usize {
        self.geometry().next_boundary(offset)
    }

    /// The previous grapheme and shaping-cluster boundary, or zero.
    pub fn prev_boundary(&self, offset: usize) -> usize {
        self.geometry().prev_boundary(offset)
    }

    fn geometry(&self) -> kivact_editing::TextGeometry<'_> {
        kivact_editing::TextGeometry::new(&self.0.shaped, self.text())
    }

    /// The text for assistive technology, one run per line, placed at
    /// `origin` in the node (the point `Draw::paragraph` drew it at), with
    /// the selection from byte `anchor` to byte `caret`. Give it to
    /// `Node::semantics_text`.
    pub fn semantics_text(&self, origin: Point, anchor: usize, caret: usize) -> p::SemanticsText {
        let text = self.text();
        let at = |r: Rect| Rect {
            x: r.x + origin.x,
            y: r.y + origin.y,
            ..r
        };
        let mut starts: Vec<usize> = self
            .lines()
            .iter()
            .map(|line| self.floor(line.range.end))
            .collect();
        match starts.last_mut() {
            Some(last) => *last = text.len(),
            None => starts.push(text.len()),
        }
        let mut runs = Vec::with_capacity(starts.len());
        let mut from = 0;
        for (i, end) in starts.iter().copied().enumerate() {
            let end = end.max(from);
            let first = self.scalar(from);
            let last = self.scalar(end);
            let line = self.lines().get(i);
            let bounds = line.map_or_else(|| at(self.caret(0)), |line| at(line.rect));
            let mut carets = Vec::with_capacity(last - first + 1);
            for index in first..=last {
                let mut rect = self.caret_at(index);
                // The end of a line that wraps lies at the next line's start.
                if index == last
                    && let Some(line) = line.filter(|_| i + 1 < starts.len())
                {
                    rect = Rect {
                        x: line.rect.right(),
                        y: line.rect.y,
                        width: rect.width,
                        height: line.rect.height,
                    };
                }
                carets.push(p::TextCaret {
                    offset: (index - first) as u32,
                    rect: at(rect),
                });
            }
            runs.push(Arc::new(p::SemanticsTextRun {
                id: p::TextRunId(i as u64),
                value: text[from..end].to_string(),
                bounds,
                carets,
            }));
            from = end;
        }
        p::SemanticsText {
            runs: runs.into(),
            start: 0,
            selection: (
                self.scalar(self.floor(anchor)) as u32,
                self.scalar(self.floor(caret)) as u32,
            ),
            caret: at(self.caret(caret)),
        }
    }

    /// The shaping this paragraph draws (a scene command's).
    pub(super) fn shaped(&self) -> Arc<ShapedText> {
        Arc::clone(&self.0.shaped)
    }

    /// `offset` cut to the text and to the char boundary at or before it.
    fn floor(&self, offset: usize) -> usize {
        self.geometry().floor(offset)
    }

    /// The scalar index of the char boundary `offset`.
    fn scalar(&self, offset: usize) -> usize {
        self.geometry().scalar_of(offset)
    }

    /// The caret at scalar boundary `index`.
    fn caret_at(&self, index: usize) -> Rect {
        kivact_editing::caret_at(&self.0.shaped, index)
    }
}

impl Draw {
    /// Shape `text` in `style`, wrapped at `max_width` (`None`: at hard line
    /// breaks only), with the window's text engine: the paragraph
    /// [`Draw::paragraph`] draws, and the one `Constraints::measure_paragraph`
    /// makes of the same inputs. Without an engine to shape with it is empty.
    pub fn layout_paragraph(
        &self,
        text: &str,
        style: &TextFormat,
        max_width: Option<f64>,
    ) -> TextParagraph {
        let origin = p::Point { x: 0.0, y: 0.0 };
        let run = super::text_run(text, origin, style, 1.0, p::Rgba::TRANSPARENT, None);
        let shaped = self
            .measurer
            .as_ref()
            .and_then(|shaper| shaper.shape(&run, max_width))
            .unwrap_or_default();
        TextParagraph::from_shaped(text, shaped, max_width)
    }

    /// Shape text around measured inline objects with the window's engine.
    /// Returns `None` when the host cannot shape the input.
    pub fn layout_inline_paragraph(
        &self,
        text: &str,
        style: &TextFormat,
        max_width: Option<f64>,
        boxes: &[p::InlineBox],
    ) -> Option<InlineParagraph> {
        let run = super::text_run(
            text,
            p::Point::default(),
            style,
            1.0,
            p::Rgba::TRANSPARENT,
            None,
        );
        let shaped = self
            .measurer
            .as_ref()?
            .shape_inline(&run, max_width, boxes)?;
        Some(InlineParagraph::from_shaped(text, shaped, max_width))
    }

    /// Draw `paragraph` with its top left at `origin`, in `color`: the glyphs
    /// and decoration lines it was shaped with, not shaped again.
    pub fn paragraph(
        &mut self,
        paragraph: &TextParagraph,
        origin: Point,
        color: impl crate::theme::value::ThemeColor,
    ) {
        let color = self.color(color.resolve(&self.theme));
        self.cmds.push(DrawCmd::Paragraph {
            shaped: paragraph.shaped(),
            origin,
            color,
        });
    }
}
