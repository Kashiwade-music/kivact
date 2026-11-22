//! Role: The date field's shared vocabulary: panels, picker, typed text, checks.

use kivact_core::view::NodeRef;
use std::rc::Rc;

/// Reads typed text as a value, `None` when it is not one.
pub(super) type Parse<T> = Rc<dyn Fn(&str) -> Option<T>>;

/// Which panel of a picker shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePanel {
    Date,
    Time,
}

/// A picker's state: closed, or open on a panel, opened from `invoker`.
#[derive(Clone, Copy, PartialEq)]
pub enum DatePicker {
    Closed,
    Open { panel: DatePanel, invoker: NodeRef },
}

/// The typed text of a date or time field.
#[derive(Clone, Debug, PartialEq)]
pub enum TypedText {
    /// The field shows the value, formatted by the caller.
    Following,
    /// The user's text and whether a commit rejected it.
    Editing { text: String, check: TextCheck },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextCheck {
    Unchecked,
    Rejected,
}

/// How typed text that is no valid value shows and closes the picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedCheck {
    /// Invalid only after a commit rejected it; a rejected commit keeps the picker (a date field).
    OnCommit,
    /// Invalid as soon as it is no valid value; every commit closes the picker (a time field).
    WhileTyping,
}
