//! Role: `EditLines`: the retained, lazily shaped lines of one text input.

use crate::text_input::records::{Before, Chunk, Key};
use crate::text_input::source::Source;
use kivact_editing::LineColors;
use std::ops::Range;

/// The lines of a text input (see the module invariants).
#[derive(Clone, Debug)]
pub(crate) struct EditLines {
    // xtask-allow(state-fields): cache: the damage the next shaping applies, from the refresh that made these lines
    pub(crate) damage: Option<kivact_editing::Damage>,
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
    // xtask-allow(state-fields): cache: the colorizer the kept line colors were made with and how many leading lines hold them; any pending damage can stand beside it
    /// The colors the lines were colored with, and how far.
    pub(crate) coloring: Option<Coloring>,
    /// The lines last drawn.
    pub(crate) drawn: Range<usize>,
}

/// The colorizer of a text input's lines and how far it has colored them.
#[derive(Clone, Debug)]
pub(crate) struct Coloring {
    pub(crate) colors: LineColors,
    /// Lines `0..colored` hold colors made with `colors`.
    pub(crate) colored: usize,
}

impl EditLines {
    /// How many leading lines hold colors (0 without a colorizer).
    pub(crate) fn colored(&self) -> usize {
        self.coloring.as_ref().map_or(0, |c| c.colored)
    }

    /// Note that lines `0..n` hold colors; nothing without a colorizer.
    pub(crate) fn set_colored(&mut self, n: usize) {
        if let Some(c) = &mut self.coloring {
            c.colored = n;
        }
    }

    pub(crate) fn hold_damage(&mut self, damage: kivact_editing::Damage) {
        self.damage = Some(match self.damage.take() {
            None => damage,
            Some(before) => {
                let range = damage.start..damage.old_end;
                let len = damage.new_end - damage.start;
                let grown = range.end.saturating_sub(before.new_end);
                kivact_editing::Damage {
                    base: before.base,
                    start: before.start.min(range.start),
                    old_end: before.old_end + grown,
                    new_end: before.new_end.max(range.end) + len - range.len(),
                }
            }
        });
    }

    /// The lines of what `source` shows in `style`, every line split, none shaped (its
    /// `damage` is not needed: nothing was laid out before).
    pub(crate) fn new(
        source: Source<'_>,
        style: kivact_types::ComputedTextStyle,
        engine: &mut arvane::Engine,
    ) -> Self {
        let (key, align) = Key::of(&source, style);
        let height = crate::text_input::line_height(engine, &key.style);
        let blank = engine
            .shape(" ", &key.style, None)
            .map_or(height, |s| s.baseline);
        // `before` always ends with the totals, even with no line yet.
        let mut lines = EditLines {
            damage: None,
            key,
            align,
            revision: source.revision,
            height,
            blank,
            chunks: Vec::new(),
            before: vec![Before::default()],
            width: 0.0,
            coloring: None,
            drawn: 0..0,
        };
        let text = lines.shown(source.value, source.placeholder);
        lines.rebuild(text, height, blank, engine);
        lines
    }
}
