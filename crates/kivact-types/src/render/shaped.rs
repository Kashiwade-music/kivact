//! Role: Shaped text: glyph runs, lines, decoration and highlight rects.

use crate::color::Rgba;
use crate::geometry::{Rect, Size};
use crate::semantics_text::TextCaret;
use crate::text::primitives::DecorationLines;
use std::ops::Range;
use std::sync::Arc;

/// Positioned glyph, usable on any thread.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Glyph {
    /// Font-specific glyph identifier.
    pub id: u32,
    /// Horizontal baseline position.
    pub x: f64,
    /// Vertical baseline position.
    pub y: f64,
    /// Logical advance.
    pub advance: f64,
}

/// A shaped run retaining its font bytes; usable on any thread.
/// The shared bytes remain valid after the text engine or its caches are
/// dropped. Glyph positions use logical pixels from the text layout origin;
/// a scene places the immutable run with its glyph command's origin.
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphRun {
    /// Complete font file bytes.
    pub font: crate::AssetBytes,
    /// Font face index.
    pub font_index: u32,
    /// Logical font size.
    pub font_size: f64,
    /// Glyph positions relative to the run's layout origin.
    pub glyphs: Vec<Glyph>,
    /// Straight-alpha text color.
    pub color: Rgba,
    /// Normalized font variation coordinates.
    pub normalized_coords: Vec<f64>,
}

/// A measured atomic object inserted at a UTF-8 boundary in a paragraph.
/// Its bottom edge aligns with the line's baseline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBox {
    /// Stable caller identity returned with the object's placement.
    pub id: u64,
    /// Byte offset in the original text, before the character at that offset.
    pub offset: usize,
    /// Logical size to reserve in the paragraph.
    pub size: Size,
}

/// The logical placement of one measured object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoxPlacement {
    /// Identity supplied by the caller.
    pub id: u64,
    /// The object's box relative to the paragraph's origin.
    pub rect: Rect,
    /// The line containing this object, numbered from zero.
    pub line: usize,
}

/// One paragraph and the placements of its inline objects.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapedInlineText {
    /// Shaped glyphs and lines, with the objects' space reserved.
    pub text: ShapedTextSnapshot,
    /// Objects in visual line order.
    pub boxes: Vec<InlineBoxPlacement>,
}

/// Owned shaping snapshot; usable on any thread, independent of the
/// engine's lifetime. All positions are logical pixels from its layout
/// origin, before any scene transform or device raster scale.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapedText {
    /// Logical measured extent.
    pub size: Size,
    /// Positioned glyph runs.
    pub runs: Vec<GlyphRunSnapshot>,
    /// Caret rectangles at Unicode scalar offsets, following shaped clusters.
    pub carets: Vec<TextCaret>,
    /// Sorted, unique shaping-cluster boundaries in Unicode scalar offsets.
    /// Includes zero and the final scalar offset; independent of caret positions.
    /// An empty list means every grapheme boundary is a stop.
    pub cluster_boundaries: Vec<u32>,
    /// The first line's baseline, from the top.
    pub baseline: f64,
    /// The lines drawn with the text (underlines and overlines before the
    /// glyphs, line-throughs after them).
    pub decorations: Vec<TextDecorationRect>,
    /// The bands of the highlighted spans, behind the glyphs.
    pub highlights: Vec<TextHighlight>,
    /// The lines shown, top to bottom (empty for empty text). A line
    /// clamp leaves out the lines it hides.
    pub lines: Vec<TextLine>,
}

/// The geometry of one shown line of a [`ShapedText`], in logical pixels
/// from its layout origin; usable on any thread.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextLine {
    /// The line's text as UTF-8 byte offsets in the string given to the
    /// engine, before any [`crate::TextTransform`]. It includes the
    /// trailing white space and the line break that end the line, and stays
    /// whole when the line is cut.
    pub range: Range<usize>,
    /// Where the line's content lies: its advance from the aligned start,
    /// without the trailing white space (a cut line: the kept text and the
    /// ellipsis), and the line box's height. The line box tiles the
    /// paragraph from top to bottom.
    pub rect: Rect,
    /// The baseline, from the top of the paragraph.
    pub baseline: f64,
    /// The distance from the baseline up to the top of the tallest font on
    /// the line.
    pub ascent: f64,
    /// The distance from the baseline down to the bottom of the deepest font
    /// on the line.
    pub descent: f64,
    /// The distance from the baseline up to the top of capital letters
    /// (the largest of the line's fonts); 0 for a line without glyphs or a
    /// font that gives none.
    pub cap_height: f64,
    /// The distance from the baseline up to the top of short lowercase
    /// letters (the largest of the line's fonts); 0 for a line without
    /// glyphs or a font that gives none.
    pub x_height: f64,
    /// Whether an ellipsis cut the line.
    pub cut: bool,
}

/// A line drawn with shaped text, in the text's coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextDecorationRect {
    /// The decoration pattern.
    pub style: crate::TextDecorationStyle,
    /// Where.
    pub rect: Rect,
    /// Its color.
    pub color: Rgba,
    /// Which line it is (one of them): a line-through is drawn after the
    /// glyphs, an underline or overline before them.
    pub line: DecorationLines,
}

/// The band of a highlighted span on one line, in the text's coordinates
/// (the span's glyphs' advance by their font's ascent and descent).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextHighlight {
    /// Where.
    pub rect: Rect,
    /// The span's index in [`super::RichText::spans`].
    pub span: usize,
}

/// Retained immutable shaped text data; cloning preserves the shaping allocation.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ShapedTextSnapshot(Arc<ShapedText>);
impl ShapedTextSnapshot {
    /// Retain an owned shaping result for reuse across snapshots and threads.
    pub fn new(value: ShapedText) -> Self {
        Self(Arc::new(value))
    }
    /// Whether both snapshots retain the same allocation.
    pub fn same_allocation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    /// Adopt a backend shaping allocation without copying its data.
    // xtask-allow(internal-contract): Text-engine/cache adapters retain an existing shaping allocation; applications use owned shaping records.
    #[doc(hidden)]
    pub fn from_shared(value: Arc<ShapedText>) -> Self {
        Self(value)
    }
    /// Lend the retained allocation to a backend cache or renderer.
    // xtask-allow(internal-contract): Text-engine/cache adapters preserve shared shaping allocation identity across the backend boundary.
    #[doc(hidden)]
    pub fn as_shared(&self) -> &Arc<ShapedText> {
        &self.0
    }
    /// Transfer the retained allocation to a consuming backend operation.
    // xtask-allow(internal-contract): Backend assembly preserves ownership-sensitive unwrap-or-clone behavior without forcing an extra retained clone.
    #[doc(hidden)]
    pub fn into_shared(self) -> Arc<ShapedText> {
        self.0
    }
    /// Mutate backend output with copy-on-write semantics before publication.
    // xtask-allow(internal-contract): Backend shaping adjustment preserves the prior Arc::make_mut behavior and clones only when shared.
    #[doc(hidden)]
    pub fn make_mut(&mut self) -> &mut ShapedText {
        Arc::make_mut(&mut self.0)
    }
}
impl From<ShapedText> for ShapedTextSnapshot {
    fn from(value: ShapedText) -> Self {
        Self::new(value)
    }
}
impl std::ops::Deref for ShapedTextSnapshot {
    type Target = ShapedText;
    fn deref(&self) -> &ShapedText {
        &self.0
    }
}

/// Retained immutable glyph run data; cloning preserves the shaping allocation.
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphRunSnapshot(Arc<GlyphRun>);
impl GlyphRunSnapshot {
    /// Retain an owned shaping result for reuse across snapshots and threads.
    pub fn new(value: GlyphRun) -> Self {
        Self(Arc::new(value))
    }
    /// Whether both snapshots retain the same allocation.
    pub fn same_allocation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    /// Adopt a backend shaping allocation without copying its data.
    // xtask-allow(internal-contract): Text-engine/cache adapters retain an existing shaping allocation; applications use owned shaping records.
    #[doc(hidden)]
    pub fn from_shared(value: Arc<GlyphRun>) -> Self {
        Self(value)
    }
    /// Lend the retained allocation to a backend cache or renderer.
    // xtask-allow(internal-contract): Text-engine/cache adapters preserve shared shaping allocation identity across the backend boundary.
    #[doc(hidden)]
    pub fn as_shared(&self) -> &Arc<GlyphRun> {
        &self.0
    }
    /// Transfer the retained allocation to a consuming backend operation.
    // xtask-allow(internal-contract): Backend assembly preserves ownership-sensitive unwrap-or-clone behavior without forcing an extra retained clone.
    #[doc(hidden)]
    pub fn into_shared(self) -> Arc<GlyphRun> {
        self.0
    }
    /// Mutate backend output with copy-on-write semantics before publication.
    // xtask-allow(internal-contract): Backend shaping adjustment preserves the prior Arc::make_mut behavior and clones only when shared.
    #[doc(hidden)]
    pub fn make_mut(&mut self) -> &mut GlyphRun {
        Arc::make_mut(&mut self.0)
    }
}
impl From<GlyphRun> for GlyphRunSnapshot {
    fn from(value: GlyphRun) -> Self {
        Self::new(value)
    }
}
impl std::ops::Deref for GlyphRunSnapshot {
    type Target = GlyphRun;
    fn deref(&self) -> &GlyphRun {
        &self.0
    }
}
