//! Role: An edit session: the selection's anchor and caret and the IME composition, holding no text.

use std::ops::Range;

/// An editing session over a text: the selection's two ends, and the IME
/// composition.
///
/// Offsets are UTF-8 byte offsets into the session's text. A session holds
/// no text, so reading it never copies the document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextSession {
    /// The selection's fixed end (byte offset).
    pub(crate) anchor: usize,
    /// The caret (byte offset); the selection runs between it and `anchor`.
    pub(crate) caret: usize,
    pub(crate) marked: Option<Range<usize>>,
}

impl TextSession {
    /// A caret at byte offset `at`, without a selection or composition.
    pub fn caret_at(at: usize) -> Self {
        TextSession {
            anchor: at,
            caret: at,
            marked: None,
        }
    }

    /// A selection from `anchor` to the caret at `caret` (byte offsets).
    pub fn select(anchor: usize, caret: usize) -> Self {
        TextSession {
            anchor,
            caret,
            marked: None,
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
