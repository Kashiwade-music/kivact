//! Role: Editor behavior hooks: the routed request, its context and the outcome.

use crate::edit_state::EditState;
use kivact_types::{Key, Modifiers};
use std::ops::Range;
use std::rc::Rc;

/// The editing behavior's result, interpreted by the native host. A
/// change the behavior made to the text is reported as `TextChanged`
/// whatever the result says: the host reads it from the state.
#[derive(Clone, Debug, PartialEq)]
pub enum EditOutcome {
    /// Leave this request for normal event routing.
    NotEditing,
    /// Consumed.
    Done,
    /// Copy or cut of a non-empty selection: the host writes the text to
    /// the clipboard (a cut also removed it from the state).
    Copy(String),
    /// Ask the host to read the clipboard and route a Paste request.
    Paste,
}

/// What an editing behavior is given beside the state and the request:
/// the editor's native constraints and its laid-out lines.
///
/// The layout queries take the state the behavior was given and answer
/// for its text as the request found it, for every line of the text
/// (`None` once the behavior changed the text, or when the editor is not
/// laid out or shows its placeholder). The host shapes only the lines a
/// query asks about. Lines are hard lines (a single-line editor has
/// one); `x` is in logical pixels from the start of the editor's text
/// box, before the editor's horizontal scroll. A context serves one
/// request: it is neither cloned nor kept.
pub struct EditContext {
    /// Whether committed text may contain line breaks.
    pub multiline: bool,
    /// Whether user input must preserve the value.
    pub read_only: bool,
    /// Whether clipboard copy and cut must hide the value.
    pub password: bool,
    layout: Option<EditLayout>,
}

impl EditContext {
    /// The constraints and the lines of one editing request (the host
    /// builds it; `layout` answers the line queries).
    #[doc(hidden)]
    pub fn new(
        multiline: bool,
        read_only: bool,
        password: bool,
        layout: Option<EditLayout>,
    ) -> Self {
        Self {
            multiline,
            read_only,
            password,
            layout,
        }
    }

    /// The lines, when they are of `state`'s text.
    fn layout(&self, state: &EditState) -> Option<&EditLayout> {
        self.layout
            .as_ref()
            .filter(|l| l.revision == state.revision())
    }

    /// The number of lines of `state`'s text.
    pub fn line_count(&self, state: &EditState) -> Option<usize> {
        self.layout(state)?.lines.count()
    }

    /// The line holding byte offset `at` of `state`'s text (a line break
    /// belongs to the line it ends).
    pub fn line_of(&self, state: &EditState, at: usize) -> Option<usize> {
        self.layout(state)?.lines.line_of(at)
    }

    /// The byte range of `line`'s text, without its line break.
    pub fn line_range(&self, state: &EditState, line: usize) -> Option<Range<usize>> {
        self.layout(state)?.lines.content(line)
    }

    /// Where the caret at byte offset `at` of `state`'s text is drawn: its
    /// `x`.
    pub fn x_of(&self, state: &EditState, at: usize) -> Option<f64> {
        let l = self.layout(state)?;
        l.lines.caret_x(state.value(), at, l.width)
    }

    /// The editing stop of `line` (a grapheme and shaping-cluster
    /// boundary) whose caret is nearest `x` (the line's end past its last
    /// character), as a byte offset of `state`'s text.
    pub fn offset_at(&self, state: &EditState, line: usize, x: f64) -> Option<usize> {
        let l = self.layout(state)?;
        l.lines.nearest(state.value(), line, x, l.width)
    }
}

/// The host's laid-out lines of one revision of the text, as an
/// [`EditContext`] reads them. Offsets and ranges are UTF-8 byte offsets
/// into that text; content ranges exclude line breaks. A break belongs to
/// the line it ends, and the byte right after it to the next line. Every
/// query answers `None` when the host cannot answer it now: the lines are
/// no longer of that revision, they show the placeholder, or the host is
/// using them.
#[doc(hidden)]
pub trait Lines {
    /// How many lines there are.
    fn count(&self) -> Option<usize>;
    /// The line holding byte offset `at`, including its terminating break.
    fn line_of(&self, at: usize) -> Option<usize>;
    /// Line `i`'s text, without its break.
    fn content(&self, i: usize) -> Option<Range<usize>>;
    /// The `x` of the caret at byte offset `at` of `text`, in a box
    /// `width` wide.
    fn caret_x(&self, text: &str, at: usize, width: f64) -> Option<f64>;
    /// The editing stop of line `i` whose caret is nearest `x`: the
    /// stops of [`crate::TextGeometry`] (grapheme and shaping-cluster
    /// boundaries); `None` past the last line.
    fn nearest(&self, text: &str, i: usize, x: f64, width: f64) -> Option<usize>;
}

/// The lines an [`EditContext`] reads, the revision of the text they are
/// of and the width of the box they are aligned in.
#[doc(hidden)]
pub struct EditLayout {
    /// The host's lines, shared with its line cache for the request.
    pub(crate) lines: Box<dyn Lines>,
    /// The revision of the text the lines are of.
    pub(crate) revision: u64,
    /// The width of the box the lines are aligned in.
    pub(crate) width: f64,
}

impl EditLayout {
    /// The host's `lines` of the text at `revision`, aligned in a box
    /// `width` wide.
    #[doc(hidden)]
    pub fn new(lines: impl Lines + 'static, revision: u64, width: f64) -> Self {
        Self {
            lines: Box::new(lines),
            revision,
            width,
        }
    }
}

/// A routed key or clipboard text for the attached editing behavior.
#[derive(Clone, Debug, PartialEq)]
pub enum EditRequest {
    /// A key after native composition and focus routing.
    Key {
        /// The logical editing key.
        key: Key,
        /// Modifiers held for this request.
        modifiers: Modifiers,
    },
    /// Clipboard text to normalize and insert.
    Paste(String),
}

/// What an editing behavior runs.
type Behave = dyn Fn(&mut EditState, EditRequest, &EditContext) -> EditOutcome;

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
        f: impl Fn(&mut EditState, EditRequest, &EditContext) -> EditOutcome + 'static,
    ) -> Self {
        Self(Rc::new(f))
    }
    /// Invoke the behavior synchronously on the supplied document and request.
    /// The caller owns host services and handles the returned editing outcome.
    pub fn invoke(
        &self,
        state: &mut EditState,
        request: EditRequest,
        context: &EditContext,
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
