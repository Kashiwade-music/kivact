//! Role: The IME session of a focusable node that edits its own text, and its events.
//!
//! A node outside the native editors (a canvas editor, a code editor that
//! draws its own text) gives the platform's input method a session
//! (`NodeExt::ime_session`): the text around its caret, the selection and
//! where the caret is. While the node has the focus the window allows the
//! IME, puts its candidate window at the caret and answers its queries from
//! the session; the IME's composition, commits and reconversion ranges come
//! back to `NodeExt::on_ime`.
//!
//! Invariants:
//! - The session's text is the node's committed text. The node shows the
//!   composition itself (where it wants, with its own underline) and moves
//!   the session's caret to it; a commit replaces the composition.
//! - Offsets are UTF-8 byte offsets of the session's text.
//! - The three values are read by one effect, which publishes the session
//!   when any of them changes.

use std::ops::Range;

use kivact_reactive::{IntoValue, Value};
use kivact_style::draw::Rect;

pub use kivact_types::ImeInput;

/// What a focusable node that edits its own text shows the input method
/// (IME): its text, selection and caret. Attach it with
/// `NodeExt::ime_session` and handle the IME's answers with
/// `NodeExt::on_ime`.
///
/// ```no_run
/// # use kivact_core as kivact;
/// use kivact::prelude::*;
/// use kivact::draw::Rect;
/// use kivact::primitive::{ImeInput, ImeSession, NodeExt};
///
/// fn line_editor() -> impl IntoView {
///     let text = signal(String::new());
///     let selection = signal(0..0);
///     let caret = signal(Rect::new(0.0, 0.0, 1.0, 16.0));
///     node()
///         .role(Role::TextBox)
///         .name("Line")
///         .focusable(true)
///         .ime_session(ImeSession::new(text, selection, caret))
///         .on_ime(move |e| {
///             if let ImeInput::Commit(committed) = e {
///                 text.update(|t| t.push_str(&committed));
///             }
///         })
/// }
/// # let _ = line_editor;
/// ```
#[derive(Clone, Copy)]
pub struct ImeSession {
    pub(crate) text: Value<String>,
    pub(crate) selection: Value<Range<usize>>,
    pub(crate) caret: Value<Rect>,
    pub(crate) password: bool,
}

impl ImeSession {
    /// A session over `text` (the committed text, never the composition),
    /// `selection` (`start..end` in UTF-8 bytes of `text`; empty: the caret)
    /// and `caret` (in the node's own coordinates, from its top left
    /// corner), which are read whenever they change. Each is a constant, a
    /// signal or a closure.
    #[track_caller]
    pub fn new(
        text: impl IntoValue<String>,
        selection: impl IntoValue<Range<usize>>,
        caret: impl IntoValue<Rect>,
    ) -> Self {
        ImeSession {
            text: text.into_value(),
            selection: selection.into_value(),
            caret: caret.into_value(),
            password: false,
        }
    }

    /// Password text: the IME is given no surrounding text, only the caret.
    #[must_use]
    pub fn password(mut self, on: bool) -> Self {
        self.password = on;
        self
    }
}
