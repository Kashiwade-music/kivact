//! Role: Editor behavior hooks: the routed request, its context and the outcome.

use crate::edit_state::EditState;
use kivact_types::{Key, Modifiers, Rect};
use std::ops::Range;
use std::rc::Rc;

/// The editing behavior's result, interpreted by the native host.
#[derive(Clone, Debug, PartialEq)]
pub enum EditOutcome {
    /// Leave this request for normal event routing.
    NotEditing,
    /// Consumed; `changed` says the value changed (send `TextChanged`).
    Done {
        /// The value changed.
        changed: bool,
    },
    /// Copy or cut of a non-empty selection: the host writes `text` to the
    /// clipboard (a cut already removed it: `changed`).
    Copy {
        /// The copied text.
        text: String,
        /// The value changed (a cut).
        changed: bool,
    },
    /// Ask the host to read the clipboard and route a Paste request.
    Paste,
}

/// What an editing behavior is given beside the state and the request:
/// the editor's native constraints and its laid-out lines.
///
/// The layout queries take the state the behavior was given and answer
/// for its text as the request found it (`None` once the behavior changed
/// the text, or when the editor is not laid out or shows its
/// placeholder). Lines are hard lines (a single-line editor has one);
/// `x` is in logical pixels from the start of the editor's text box,
/// before the editor's horizontal scroll.
pub struct EditContext<'a> {
    // xtask-allow(state-fields): option: the editor's authored multiline prop the engine passes per request; false means committed text keeps no line breaks
    /// Whether committed text may contain line breaks.
    pub multiline: bool,
    // xtask-allow(state-fields): option: the editor's authored read_only prop the engine passes per request; false lets user input change the value
    /// Whether user input must preserve the value.
    pub read_only: bool,
    // xtask-allow(state-fields): option: whether the editor's authored input purpose is Password, passed per request; false lets copy and cut expose the value
    /// Whether clipboard copy and cut must hide the value.
    pub password: bool,
    // xtask-allow(state-fields): independent: the editor's laid-out lines for this request, which the engine always supplies and a caller without layout leaves out; the flags are authored props and do not decide whether lines exist
    layout: Option<EditLayout<'a>>,
}

impl<'a> EditContext<'a> {
    /// Build the constraints and optional geometry for one editing request.
    /// Geometry queries remain valid only for the supplied layout's revision.
    pub fn new(
        multiline: bool,
        read_only: bool,
        password: bool,
        layout: Option<EditLayout<'a>>,
    ) -> Self {
        Self {
            multiline,
            read_only,
            password,
            layout,
        }
    }

    /// The lines, when they are of `state`'s text.
    fn layout(&self, state: &EditState) -> Option<EditLayout<'a>> {
        self.layout
            .filter(|l| l.revision == state.revision() && l.lines.shows_value())
    }

    /// The number of lines of `state`'s text.
    pub fn line_count(&self, state: &EditState) -> Option<usize> {
        Some(self.layout(state)?.lines.len())
    }

    /// The line holding byte offset `at` of `state`'s text (a line break
    /// belongs to the line it ends).
    pub fn line_of(&self, state: &EditState, at: usize) -> Option<usize> {
        Some(self.layout(state)?.lines.line_of(at))
    }

    /// The byte range of `line`'s text, without its line break.
    pub fn line_range(&self, state: &EditState, line: usize) -> Option<Range<usize>> {
        self.layout(state)?.lines.content(line)
    }

    /// Where the caret at byte offset `at` of `state`'s text is drawn: its
    /// `x`.
    pub fn x_of(&self, state: &EditState, at: usize) -> Option<f64> {
        let l = self.layout(state)?;
        Some(l.lines.caret(state.value(), at, l.width).x)
    }

    /// The grapheme boundary of `line` whose caret is nearest `x` (the
    /// line's end past its last character), as a byte offset of
    /// `state`'s text.
    pub fn offset_at(&self, state: &EditState, line: usize, x: f64) -> Option<usize> {
        let l = self.layout(state)?;
        (line < l.lines.len()).then(|| l.lines.nearest(state.value(), line, x, l.width))
    }
}

/// The host's laid-out lines, as an [`EditContext`] reads them.
/// Offsets and ranges are UTF-8 byte positions in the supplied document;
/// content ranges exclude line breaks. A break belongs to the line it ends,
/// and the byte immediately after the break belongs to the next line.
pub trait Lines {
    /// They show the value, not the placeholder.
    fn shows_value(&self) -> bool;
    /// How many there are.
    fn len(&self) -> usize;
    /// Whether this provider has no lines.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The line holding byte offset `at`, including its terminating break.
    fn line_of(&self, at: usize) -> usize;
    /// Line `i`'s text, without its break.
    fn content(&self, i: usize) -> Option<Range<usize>>;
    /// The caret at byte offset `at` of `text`, in a box `width` wide.
    fn caret(&self, text: &str, at: usize, width: f64) -> Rect;
    /// The grapheme boundary of line `i` whose caret is nearest `x`.
    fn nearest(&self, text: &str, i: usize, x: f64, width: f64) -> usize;
}

/// The lines an [`EditContext`] reads, the revision of the text they are
/// of and the width of the box they are aligned in.
#[derive(Clone, Copy)]
pub struct EditLayout<'a> {
    /// Host line geometry provider for this snapshot.
    pub lines: &'a (dyn Lines + 'a),
    /// Document revision for which the geometry was computed.
    pub revision: u64,
    /// Logical alignment width before horizontal text scrolling.
    pub width: f64,
}

/// A routed key or clipboard text for the attached editing behavior.
pub enum EditRequest<'a> {
    /// A key after native composition and focus routing.
    Key {
        /// The logical editing key.
        key: Key,
        /// Modifiers held for this request.
        modifiers: Modifiers,
    },
    /// Clipboard text to normalize and insert.
    Paste(&'a str),
}

/// What an editing behavior runs.
type Behave = dyn Fn(&mut EditState, EditRequest<'_>, &EditContext<'_>) -> EditOutcome;

/// An editing function run synchronously while the engine owns its model.
///
/// The function changes the supplied EditState and may use state it
/// captured; it must not touch signals or the engine tree. Native IME
/// transport, shaping, pointer selection and clipboard services remain
/// with the host. Two behaviors are equal when they are the same function
/// object.
#[derive(Clone)]
pub struct EditBehavior(Rc<Behave>);

impl EditBehavior {
    /// Use this function for routed editing keys and clipboard text.
    pub fn new(
        f: impl Fn(&mut EditState, EditRequest<'_>, &EditContext<'_>) -> EditOutcome + 'static,
    ) -> Self {
        Self(Rc::new(f))
    }
    /// Invoke the behavior synchronously on the supplied document and request.
    /// The caller owns host services and handles the returned editing outcome.
    pub fn invoke(
        &self,
        state: &mut EditState,
        request: EditRequest<'_>,
        context: &EditContext<'_>,
    ) -> EditOutcome {
        (self.0)(state, request, context)
    }
}

impl PartialEq for EditBehavior {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for EditBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EditBehavior(..)")
    }
}
