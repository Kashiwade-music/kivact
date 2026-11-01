//! Role: Snapshots of an editor's shaped lines and carets.

use kivact_types::Rect;
use std::ops::Range;

/// Shaped editing geometry of some of an editor's lines (those asked for),
/// in logical window coordinates after scroll and transforms. Rectangles
/// are the outer bounds of transformed boxes.
/// Built with [`TextLayout::new`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct TextLayout {
    /// The active selection end, at the same position used by the native IME.
    pub caret: Rect,
    /// How many lines the editor shows (0 while it shows its placeholder);
    /// native editors break at line feeds.
    pub line_count: usize,
    /// The requested lines that exist, in reading order.
    pub lines: Vec<EditLine>,
    /// The caret stops of those lines, with Unicode scalar offsets in the
    /// whole value.
    pub carets: Vec<EditCaret>,
}

impl TextLayout {
    /// A layout whose active selection end is `caret`, of an editor showing
    /// `line_count` lines, holding the requested `lines` and their `carets`.
    pub fn new(
        caret: Rect,
        line_count: usize,
        lines: Vec<EditLine>,
        carets: Vec<EditCaret>,
    ) -> Self {
        TextLayout {
            caret,
            line_count,
            lines,
            carets,
        }
    }
}

/// The original text range and geometry of one native editing line.
/// Built with [`EditLine::new`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct EditLine {
    /// UTF-8 byte range in the original value, including its trailing break.
    pub range: Range<usize>,
    /// The line's transformed outer bounds in logical window coordinates.
    pub rect: Rect,
    /// Window y coordinate of the baseline at the line origin.
    pub baseline: f32,
}

impl EditLine {
    /// A line covering `range` of the value, bounded by `rect`, whose
    /// baseline is at window `baseline`.
    pub const fn new(range: Range<usize>, rect: Rect, baseline: f32) -> Self {
        EditLine {
            range,
            rect,
            baseline,
        }
    }
}

/// One shaped caret stop in the editor's original value.
/// Built with [`EditCaret::new`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct EditCaret {
    /// Unicode scalar offset, including the final boundary.
    pub offset: u32,
    /// The caret's transformed outer bounds in logical window coordinates.
    pub rect: Rect,
}

impl EditCaret {
    /// A caret stop at scalar `offset` occupying `rect`.
    pub const fn new(offset: u32, rect: Rect) -> Self {
        EditCaret { offset, rect }
    }
}
