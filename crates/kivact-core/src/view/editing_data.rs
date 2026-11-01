//! Role: Passive native editing settings and edit notifications.

use kivact_editing::{EditContext, EditOutcome, EditRequest, EditState};
use kivact_reactive::Prop as InputProp;
use std::ops::Range;
use std::panic::Location;
use std::rc::Rc;

pub use kivact_editing::{TextChange, TextSession};

/// A native value or composition update, including preedit text.
///
/// The update says what changed, not the whole text: `changes` replace
/// byte ranges in order (each in the text the previous one left, starting
/// from the text before the update), and `session` is the selection and
/// composition after them, in byte offsets of the new text. Applying the
/// changes to a copy of the text keeps the copy equal to the editor's,
/// preedit included. A composition that ends without changing the text
/// has no changes. Caret-only moves produce no event.
#[derive(Clone, Debug, PartialEq)]
pub struct TextEditEvent {
    /// The range replacements, in order.
    pub changes: Vec<TextChange>,
    /// The selection and composition after the changes.
    pub session: TextSession,
    /// The whole text after the changes, for an editor attached with
    /// [`TextEditing::report_text`]; `None` otherwise.
    pub text: Option<String>,
}

impl TextEditEvent {
    /// Make the same changes to `text`, a copy of the text before them.
    /// Returns `false` when a change does not fit `text` (its range is not
    /// in it on char boundaries): the copy is not the editor's text, and
    /// the changes from that one on are not made. Sync the copy again from
    /// the editor ([`NodeRefExt::edit_state`]).
    ///
    /// [`NodeRefExt::edit_state`]: crate::primitive::NodeRefExt::edit_state
    pub fn apply(&self, text: &mut String) -> bool {
        self.changes.iter().all(|change| change.apply(text))
    }

    /// The marked range of a composition in progress, or `None` after a
    /// commit or cancellation.
    pub fn composition(&self) -> Option<Range<usize>> {
        self.session.marked()
    }
}

/// One colored byte range of a line of a native editor
/// ([`TextEditing::highlight`]).
#[derive(Clone, Debug, PartialEq)]
pub struct TextHighlight {
    /// The UTF-8 byte range in the line, which excludes its line feed.
    pub range: Range<usize>,
    /// The glyphs' color.
    pub color: kivact_style::Color,
}

/// What a behavior attached with [`TextEditing::behavior`] runs.
pub(crate) type Behavior = dyn Fn(&mut EditState, EditRequest<'_>, &EditContext<'_>) -> EditOutcome;

/// Where the line colors of a mounted editor are bound: its runtime,
/// owner and theme, the node and the author's call site.
pub(crate) struct HighlightAt {
    pub(crate) rt: Rc<kivact_reactive::Runtime>,
    pub(crate) owner: kivact_reactive::Owner,
    pub(crate) theme: Option<kivact_reactive::Value<kivact_style::Theme>>,
    pub(crate) id: kivact_wire::NodeId,
    pub(crate) at: &'static Location<'static>,
}

/// Binds the line colors of a mounted editor (the content's type erased).
pub(crate) type BindHighlight = Box<dyn FnOnce(HighlightAt)>;

/// Native editing attached to an ordinary node through a [`Part`](crate::view::Part).
///
/// This supplies state, shaping, caret, selection and the platform IME session.
/// The author supplies role, accessible name and focus policy on the Part.
/// The resulting editor is a native leaf: children and custom drawing are
/// unsupported. Canvas regions cannot carry an editing attachment.
pub struct TextEditing {
    pub(crate) behavior: Option<Rc<Behavior>>,
    pub(crate) highlight: Option<BindHighlight>,
    pub(crate) forward_keys: Option<InputProp<Vec<kivact_wire::Key>>>,
    pub(crate) value: InputProp<Rc<str>>,
    pub(crate) multiline: bool,
    pub(crate) report_text: bool,
    pub(crate) password: Option<InputProp<bool>>,
    pub(crate) read_only: Option<InputProp<bool>>,
    pub(crate) max_length: Option<InputProp<usize>>,
    pub(crate) placeholder: Option<InputProp<String>>,
    pub(crate) on_edit: Option<Rc<dyn Fn(TextEditEvent) -> kivact_reactive::Result<()>>>,
    pub(crate) on_selection: Option<SessionHandler>,
    /// An `on_selection` handler was attached (the engine reports the
    /// session).
    pub(crate) reports_session: bool,
}

/// What [`TextEditing::on_selection`] runs.
pub(crate) type SessionHandler = Rc<dyn Fn(TextSession) -> kivact_reactive::Result<()>>;
