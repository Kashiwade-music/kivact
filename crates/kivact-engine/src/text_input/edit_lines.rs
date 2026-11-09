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
    /// The colors the lines were colored with.
    pub(crate) colors: Option<LineColors>,
    pub(crate) colored: usize,
    /// The lines last drawn.
    pub(crate) drawn: Range<usize>,
}

impl EditLines {
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
            colors: None,
            colored: 0,
            drawn: 0..0,
        };
        let text = lines.shown(source.value, source.placeholder);
        lines.rebuild(text, height, blank, engine);
        lines
    }
}
