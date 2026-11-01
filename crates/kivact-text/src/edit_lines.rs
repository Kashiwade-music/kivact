//! Role: `EditLines`: the retained, lazily shaped lines of one text input.

use crate::records::{Before, Chunk, Key};
use kivact_editing::LineColors;
use std::ops::Range;

/// The lines of a text input (see the module invariants).
#[derive(Clone, Debug)]
pub struct EditLines {
    pub(crate) key: Option<Key>,
    pub(crate) align: kivact_types::TextAlign,
    pub(crate) revision: u64,
    pub(crate) height: f32,
    /// The baseline of a line without text (a space's, in the style).
    pub(crate) blank: f32,
    pub(crate) chunks: Vec<Chunk>,
    /// What comes before each chunk, then in all.
    pub(crate) before: Vec<Before>,
    /// The widest shaped line's width.
    pub(crate) width: f32,
    /// The colors the lines were colored with.
    pub(crate) colors: Option<LineColors>,
    pub(crate) colored: usize,
    /// The lines last drawn.
    pub(crate) drawn: Range<usize>,
}

impl Default for EditLines {
    /// No lines yet, with what comes before them all (nothing): `before`
    /// always ends with the totals.
    fn default() -> Self {
        Self {
            key: None,
            align: kivact_types::TextAlign::default(),
            revision: 0,
            height: 0.0,
            blank: 0.0,
            chunks: Vec::new(),
            before: vec![Before::default()],
            width: 0.0,
            colors: None,
            colored: 0,
            drawn: 0..0,
        }
    }
}

impl EditLines {
    pub(crate) fn key(&self) -> &Key {
        self.key
            .as_ref()
            .expect("invariant: refreshed lines have a key")
    }
}
