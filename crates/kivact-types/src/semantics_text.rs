//! Role: Portable editable text runs, carets and selection snapshots.

use std::sync::Arc;

use crate::geometry::{NodeId, Rect};

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
/// Built with [`SemanticsText::new`].
#[derive(Clone, Debug)]
#[non_exhaustive]
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

impl SemanticsText {
    /// Text made of `runs` starting at scalar offset `start`, with the
    /// selection `(anchor, caret)` and the active end's `caret` bounds.
    pub fn new(
        runs: Arc<[Arc<SemanticsTextRun>]>,
        start: u32,
        selection: (u32, u32),
        caret: Rect,
    ) -> Self {
        SemanticsText {
            runs,
            start,
            selection,
            caret,
        }
    }
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

/// One line of selectable text and the carets used to locate its characters.
///
/// A hard line break belongs to the end of its run. The native adapter treats
/// CRLF as one selectable unit when converting scalar offsets to platform
/// text positions. Missing character geometry falls back to the run's bounds.
/// Built with [`SemanticsTextRun::new`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SemanticsTextRun {
    /// Stable platform child identity, unique among all nodes and text runs.
    pub id: NodeId,
    /// Original text, including any trailing line break.
    pub value: String,
    /// Unscrolled content bounds in the owning node's coordinate space.
    pub bounds: Rect,
    /// Run-local scalar offsets and content-space caret rectangles.
    pub carets: Vec<TextCaret>,
}

impl SemanticsTextRun {
    /// A run with identity `id` holding `value` within `bounds`, with the
    /// caret rectangles of its characters.
    pub fn new(id: NodeId, value: String, bounds: Rect, carets: Vec<TextCaret>) -> Self {
        SemanticsTextRun {
            id,
            value,
            bounds,
            carets,
        }
    }
}

/// A cursor position in shaped text; usable on any thread.
///
/// Combining marks and ligatures can share a cluster and therefore a rectangle.
/// Built with [`TextCaret::new`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct TextCaret {
    /// Unicode scalar offset after `TextTransform`, including the
    /// final boundary. This is neither a UTF-8 byte nor a grapheme index;
    /// case conversion can change the number of scalars.
    pub offset: u32,
    /// Logical caret bounds relative to the text layout origin.
    pub rect: Rect,
}

impl TextCaret {
    /// A caret at scalar `offset` occupying `rect`.
    pub const fn new(offset: u32, rect: Rect) -> Self {
        TextCaret { offset, rect }
    }
}
