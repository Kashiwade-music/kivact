//! Role: Snapshots of an editor's shaped lines and carets.

use kivact_types::Rect;
use std::ops::Range;

/// Shaped editing geometry of some of an editor's lines (those asked for),
/// in logical window coordinates after scroll and transforms. Rectangles
/// are the outer bounds of transformed boxes.
#[derive(Clone, Debug, PartialEq)]
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

/// The original text range and geometry of one native editing line.
#[derive(Clone, Debug, PartialEq)]
pub struct EditLine {
    /// UTF-8 byte range in the original value, including its trailing break.
    pub range: Range<usize>,
    /// The line's transformed outer bounds in logical window coordinates.
    pub rect: Rect,
    /// Window y coordinate of the baseline at the line origin.
    pub baseline: f64,
}

/// One shaped caret stop in the editor's original value.
#[derive(Clone, Debug, PartialEq)]
pub struct EditCaret {
    /// Unicode scalar offset, including the final boundary.
    pub offset: u32,
    /// The caret's transformed outer bounds in logical window coordinates.
    pub rect: Rect,
}
