//! Role: An edit session: selection anchor, caret and IME composition, holding no text.

use std::ops::Range;

/// An editing session over a text: the selection's two ends, and the IME
/// composition.
///
/// Offsets are UTF-8 byte offsets into the session's text. A session holds
/// no text, so reading it never copies the document.
#[derive(Clone, Debug, Default)]
pub struct TextSession {
    /// The selection's fixed end (byte offset).
    pub(crate) anchor: usize,
    /// The caret (byte offset); the selection runs between it and `anchor`.
    pub(crate) caret: usize,
    pub(crate) marked: Option<Range<usize>>,
    /// The column a vertical caret move aims for (set by `EditState::move_vertically`, cleared by every other move or edit). It is not part of the session's identity: equality ignores it.
    pub(crate) goal_x: Option<f64>,
}

impl PartialEq for TextSession {
    fn eq(&self, other: &Self) -> bool {
        self.anchor == other.anchor && self.caret == other.caret && self.marked == other.marked
    }
}

impl Eq for TextSession {}

impl TextSession {
    /// A caret at byte offset `at`, without a selection or composition.
    pub fn caret_at(at: usize) -> Self {
        TextSession {
            anchor: at,
            caret: at,
            marked: None,
            goal_x: None,
        }
    }

    /// A selection from `anchor` to the caret at `caret` (byte offsets).
    pub fn select(anchor: usize, caret: usize) -> Self {
        TextSession {
            anchor,
            caret,
            marked: None,
            goal_x: None,
        }
    }

    /// The selected byte range (empty: the caret).
    pub fn selection(&self) -> Range<usize> {
        self.anchor.min(self.caret)..self.anchor.max(self.caret)
    }

    /// The caret's byte offset.
    pub fn caret(&self) -> usize {
        self.caret
    }

    /// The selection's fixed end (the caret's other side).
    pub fn anchor(&self) -> usize {
        self.anchor
    }

    /// The composed (marked) byte range of an IME composition.
    pub fn marked(&self) -> Option<Range<usize>> {
        self.marked.clone()
    }

    /// An IME composition is in progress.
    pub fn is_composing(&self) -> bool {
        self.marked.is_some()
    }
}
