//! Role: An edit session: selection, caret side and IME composition, holding no text.

use std::ops::Range;

/// An editing session over a text: the selection, which end holds the
/// caret, and the IME composition.
///
/// Offsets are UTF-8 byte offsets into the session's text. A session holds
/// no text, so reading it never copies the document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextSession {
    pub(crate) sel: Range<usize>,
    pub(crate) reversed: bool,
    pub(crate) marked: Option<Range<usize>>,
}

impl TextSession {
    /// A caret at byte offset `at`, without a selection or composition.
    pub fn caret_at(at: usize) -> Self {
        TextSession {
            sel: at..at,
            reversed: false,
            marked: None,
        }
    }

    /// A selection from `anchor` to the caret at `caret` (byte offsets).
    pub fn select(anchor: usize, caret: usize) -> Self {
        TextSession {
            sel: anchor.min(caret)..anchor.max(caret),
            reversed: caret < anchor,
            marked: None,
        }
    }

    /// The selected byte range (empty: the caret).
    pub fn selection(&self) -> Range<usize> {
        self.sel.clone()
    }

    /// The caret's byte offset.
    pub fn caret(&self) -> usize {
        if self.reversed {
            self.sel.start
        } else {
            self.sel.end
        }
    }

    /// The selection's fixed end (the caret's other side).
    pub fn anchor(&self) -> usize {
        if self.reversed {
            self.sel.end
        } else {
            self.sel.start
        }
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
