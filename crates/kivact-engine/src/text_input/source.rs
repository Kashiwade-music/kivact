//! Role: What a text input shows, to lay its lines out from.

use kivact_editing::Damage;

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
    /// Hard breaks at LF and soft wrapping (an unmasked single line shows
    /// LF/CR as spaces).
    pub(crate) breaks: Breaks,
}

/// How the shown text breaks into lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Breaks {
    /// One line: an unmasked LF or CR shows as a space.
    Single,
    /// Split at LF hard breaks; `wrap` is the content width the rows also
    /// soft-wrap at, when the field wraps.
    Lines { wrap: Option<f64> },
}

impl Breaks {
    /// Whether the text splits at LF hard breaks.
    pub(crate) fn multiline(self) -> bool {
        matches!(self, Breaks::Lines { .. })
    }

    /// The width the rows soft-wrap at, if they do.
    pub(crate) fn wrap_width(self) -> Option<f64> {
        match self {
            Breaks::Single => None,
            Breaks::Lines { wrap } => wrap,
        }
    }
}
