//! Role: What a text input shows, to lay its lines out from.

use kivact_editing::Damage;

/// What a text input shows, to lay its lines out from.
#[derive(Clone, Copy)]
pub(crate) struct Source<'a> {
    /// Borrowed original UTF-8 value; all source byte offsets refer to it.
    pub(crate) value: &'a str,
    /// Revision of the value and compatible damage history.
    pub(crate) revision: u64,
    // xtask-allow(state-fields): independent: the damage since the last refresh, from the same snapshot
    /// Optional UTF-8 byte replacement since the previously refreshed revision.
    pub(crate) damage: Option<Damage>,
    /// Borrowed UTF-8 display text used only while the value is empty.
    pub(crate) placeholder: &'a str,
    // xtask-allow(state-fields): independent: one of the input's flags in the snapshot; any combination is valid
    /// Visually mask non-LF scalars with bullets; LF keeps display spacing.
    /// This does not redact excerpts or published raw run text. Callers must
    /// suppress/protect surrounding IME text and accessible publication.
    pub(crate) password: bool,
    // xtask-allow(state-fields): independent: one of the input's flags in the snapshot; any combination is valid
    /// Split at LF hard breaks; an unmasked single line shows LF/CR as spaces.
    pub(crate) multiline: bool,
    // xtask-allow(state-fields): option: absent means the text does not soft-wrap
    /// A multiline editor's content width when soft wrapping is enabled.
    pub(crate) wrap_width: Option<f64>,
}
