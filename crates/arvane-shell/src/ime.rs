//! Role: Defines the surrounding text and geometry supplied to platform IMEs.

use kivact_types::Rect;

/// What the focused text input shows the IME: the text around its caret,
/// its selection and composition, and where its characters are, for the
/// platform IME's queries (on Windows: reconversion, `IMR_DOCUMENTFEED`
/// and `IMR_QUERYCHARPOSITION`).
///
/// A platform IME needs the context of the caret, not the document: the
/// host gives an excerpt (the lines around the caret, a few thousand
/// characters at most). Offsets in this snapshot use UTF-16 code units.
/// Reconversion selections emitted as [`ShellEvent::Ime`](crate::ShellEvent::Ime)
/// use UTF-8 byte ranges relative to `text`; the engine owns the excerpt's
/// mapping back into the complete document.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShellImeText {
    // xtask-allow(state-fields): independent: the input is a password; a password field can still compose, so every combination with the other snapshot values is valid
    /// Password input: no surrounding text is supplied to native reconversion.
    pub password: bool,
    /// The text around the caret (all of the input's text when short).
    pub text: String,
    /// Where `text` starts in the input's whole text, in UTF-16 code units
    /// (0 when it starts with it).
    pub text_start: usize,
    /// The selection, `[start, end)` in UTF-16 code units of `text`
    /// (empty: the caret), cut to `text`.
    pub selection: (usize, usize),
    /// The caret in UTF-16 code units: the selection's end, or its start
    /// when the selection was made backwards.
    pub caret: usize,
    // xtask-allow(state-fields): option: the composition's `[start, end)` in UTF-16 code units; absent when not composing
    /// The composition, `[start, end)` in UTF-16 code units, if any.
    pub composition: Option<(usize, usize)>,
    /// The UTF-16 offset of the character `rects` starts with: the
    /// composition's start while composing, else the caret's.
    pub rects_from: usize,
    /// The boxes of the characters from `rects_from` on (one per UTF-16
    /// code unit; the last is the caret after them), in logical window
    /// pixels.
    pub rects: Vec<Rect>,
    /// The input's text area in logical window pixels.
    pub area: Rect,
}
