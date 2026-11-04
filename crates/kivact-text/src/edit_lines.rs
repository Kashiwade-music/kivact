//! Role: `EditLines`: the retained, lazily shaped lines of one text input.

use crate::records::{Before, Chunk, Key};
use crate::source::Source;
use kivact_editing::LineColors;
use std::ops::Range;

/// The lines of a text input (see the module invariants).
#[derive(Clone, Debug)]
pub struct EditLines {
    pub(crate) key: Key,
    pub(crate) align: kivact_types::TextAlign,
    pub(crate) revision: u64,
    pub(crate) height: f64,
    /// The baseline of a line without text (a space's, in the style).
    pub(crate) blank: f64,
    pub(crate) chunks: Vec<Chunk>,
    /// What comes before each chunk, then in all.
    pub(crate) before: Vec<Before>,
    /// The widest shaped line's width.
    pub(crate) width: f64,
    /// The colors the lines were colored with.
    pub(crate) colors: Option<LineColors>,
    pub(crate) colored: usize,
    /// The lines last drawn.
    pub(crate) drawn: Range<usize>,
}

impl EditLines {
    /// The lines of what `source` shows in `style`, every line split, none shaped (its
    /// `damage` is not needed: nothing was laid out before).
    pub fn new(
        source: Source<'_>,
        style: kivact_types::ComputedTextStyle,
        engine: &mut arvane::Engine,
    ) -> Self {
        let (key, align) = Key::of(&source, style);
        let height = crate::line_height(engine, &key.style);
        let blank = engine
            .shape(" ", &key.style, None)
            .map_or(height, |s| s.baseline);
        // `before` always ends with the totals, even with no line yet.
        let mut lines = EditLines {
            key,
            align,
            revision: source.revision,
            height,
            blank,
            chunks: Vec::new(),
            before: vec![Before::default()],
            width: 0.0,
            colors: None,
            colored: 0,
            drawn: 0..0,
        };
        let text = lines.shown(source.value, source.placeholder);
        lines.rebuild(text, height, blank, engine);
        lines
    }
}
