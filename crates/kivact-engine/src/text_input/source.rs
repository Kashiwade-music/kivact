//! Role: What a text input shows, to lay its lines out from.

use kivact_editing::Damage;

// xtask-allow(state-fields): Borrowed editor input snapshot has independent damage and presentation parameters, not retained transition state.
/// What a text input shows, to lay its lines out from.
#[derive(Clone, Copy)]
pub(crate) struct Source<'a> {
    /// Borrowed original UTF-8 value; all source byte offsets refer to it.
    pub(crate) value: &'a str,
    /// Revision of the value and compatible damage history.
    pub(crate) revision: u64,
    /// Optional UTF-8 byte replacement since the previously refreshed revision.
    pub(crate) damage: Option<Damage>,
    /// Borrowed UTF-8 display text used only while the value is empty.
    pub(crate) placeholder: &'a str,
    /// Visually mask non-LF scalars with bullets; LF keeps display spacing.
    /// This does not redact excerpts or published raw run text. Callers must
    /// suppress/protect surrounding IME text and accessible publication.
    pub(crate) password: bool,
    /// Split at LF hard breaks; an unmasked single line shows LF/CR as spaces.
    pub(crate) multiline: bool,
    /// A multiline editor's content width when soft wrapping is enabled.
    pub(crate) wrap_width: Option<f64>,
}
