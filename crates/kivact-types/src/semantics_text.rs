//! Role: Portable editable text runs, carets and selection snapshots.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::geometry::Rect;
use crate::ids::TextRunId;

/// Editable text and its active selection as published to assistive technology.
///
/// Rectangles use the owner's unscrolled content coordinates, like
/// `SemanticsNode::bounds`, and inherit its transform. Protected text
/// must omit this snapshot. The runs become platform text children and do not
/// appear as ordinary nodes in `SemanticsTree::nodes`.
///
/// The runs may be a part of a long text, in a row from `start`: the
/// lines around what is shown and the caret (as an editor in a browser
/// gives assistive technology the text around the caret), so a long
/// document costs the platform those lines. Assistive technology reads
/// the text through the runs; the owner's value may be left out.
///
/// Runs are shared: a producer keeps the runs of the lines an edit did not
/// touch and gives them again (and the whole list while none changed),
/// and two snapshots compare their runs by identity first, so an update of
/// a long text hands the platform only the runs that changed.
/// Hashing uses run and caret contents, never Arc addresses; both signed zeros
/// have the same hash without changing float equality or NaN comparisons.
#[derive(Clone, Debug)]
pub struct SemanticsText {
    /// Text runs in reading order; even empty text has one empty run. A
    /// run starts at the scalar offset where the run before it ends, the
    /// first at `start`.
    pub runs: Arc<[Arc<SemanticsTextRun>]>,
    /// The Unicode scalar offset in the complete text where the first run
    /// starts.
    pub start: u32,
    /// `(anchor, caret)` in Unicode scalar offsets across the complete
    /// text, both in the runs (from `start` to where the last run ends).
    pub selection: (u32, u32),
    /// The active selection end, using the editor's painted caret geometry.
    pub caret: Rect,
}

impl PartialEq for SemanticsText {
    fn eq(&self, other: &Self) -> bool {
        self.selection == other.selection
            && self.start == other.start
            && self.caret == other.caret
            && (Arc::ptr_eq(&self.runs, &other.runs)
                || (self.runs.len() == other.runs.len()
                    && self
                        .runs
                        .iter()
                        .zip(other.runs.iter())
                        .all(|(a, b)| Arc::ptr_eq(a, b) || a == b)))
    }
}

impl Hash for SemanticsText {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.runs.hash(state);
        self.start.hash(state);
        self.selection.hash(state);
        hash_rect(self.caret, state);
    }
}

/// One line of selectable text and the carets used to locate its characters.
///
/// A hard line break belongs to the end of its run. The native adapter treats
/// CRLF as one selectable unit when converting scalar offsets to platform
/// text positions. Missing character geometry falls back to the run's bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticsTextRun {
    /// Stable platform child identity, unique among all nodes and text runs.
    pub id: TextRunId,
    /// Original text, including any trailing line break.
    pub value: String,
    /// Unscrolled content bounds in the owning node's coordinate space.
    pub bounds: Rect,
    /// Run-local scalar offsets and content-space caret rectangles.
    pub carets: Vec<TextCaret>,
}

impl Hash for SemanticsTextRun {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.value.hash(state);
        hash_rect(self.bounds, state);
        self.carets.hash(state);
    }
}

/// A cursor position in shaped text; usable on any thread.
///
/// Combining marks and ligatures can share a cluster and therefore a rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextCaret {
    /// Unicode scalar offset after `TextTransform`, including the
    /// final boundary. This is neither a UTF-8 byte nor a grapheme index;
    /// case conversion can change the number of scalars.
    pub offset: u32,
    /// Logical caret bounds relative to the text layout origin.
    pub rect: Rect,
}

impl Hash for TextCaret {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
        hash_rect(self.rect, state);
    }
}

/// Hash geometry by value, preserving the existing float equality contract.
fn hash_rect<H: Hasher>(rect: Rect, state: &mut H) {
    for value in [rect.x, rect.y, rect.width, rect.height] {
        let value = if value == 0.0 { 0.0 } else { value };
        value.to_bits().hash(state);
    }
}
