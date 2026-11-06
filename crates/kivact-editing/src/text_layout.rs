//! Role: Snapshots of an editor's shaped lines and carets.

use kivact_types::Rect;
use std::ops::Range;

/// Shaped editing geometry of some of an editor's lines (those asked for),
/// in logical window coordinates after scroll and transforms when queried
/// through a node reference, or in canvas coordinates in `draw_with_layout`. Rectangles
/// are the outer bounds of transformed boxes.
#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    /// The active selection end, at the same position used by the native IME.
    pub caret: Rect,
    /// Selection bands on the requested lines.
    pub selection: Vec<Rect>,
    /// How many lines the editor shows (0 while it shows its placeholder);
    /// native editors break at line feeds.
    pub line_count: usize,
    /// The requested lines that exist, in reading order.
    pub lines: Vec<EditLine>,
    /// The caret stops of those lines, with Unicode scalar offsets in the
    /// whole value.
    pub carets: Vec<EditCaret>,
}

/// The original text range and geometry of one native editing line.
#[derive(Clone, Debug, PartialEq)]
pub struct EditLine {
    /// UTF-8 byte range in the original value, including its trailing break.
    pub range: Range<usize>,
    /// The line's outer bounds in the snapshot's coordinate space.
    pub rect: Rect,
    /// Baseline y coordinate in the snapshot's coordinate space.
    pub baseline: f64,
}

/// One shaped caret stop in the editor's original value.
#[derive(Clone, Debug, PartialEq)]
pub struct EditCaret {
    /// Unicode scalar offset, including the final boundary.
    pub offset: u32,
    /// The caret's outer bounds in the snapshot's coordinate space.
    pub rect: Rect,
}

impl TextLayout {
    /// The Unicode scalar offset nearest a point, among the requested lines.
    pub fn hit_test(&self, point: kivact_types::Point) -> Option<u32> {
        let vertical = |r: Rect| (r.y - point.y).max(0.0).max(point.y - r.bottom());
        let line = self
            .lines
            .iter()
            .min_by(|a, b| vertical(a.rect).total_cmp(&vertical(b.rect)));
        self.carets
            .iter()
            .filter(|caret| {
                line.is_none_or(|line| {
                    caret.rect.y >= line.rect.y && caret.rect.y < line.rect.bottom()
                })
            })
            .min_by(|a, b| {
                (a.rect.x - point.x)
                    .abs()
                    .total_cmp(&(b.rect.x - point.x).abs())
            })
            .map(|caret| caret.offset)
    }
}
