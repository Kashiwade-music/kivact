//! Role: The options of a date field: how it parses, accepts and checks text.

use kivact_core::state::{Callback, Signal, callback};
use kivact_core::view::HandlerResult;
use std::rc::Rc;

use super::types::{Parse, TypedCheck};

pub struct DateFieldOptions<T: 'static> {
    pub(super) parse: Parse<T>,
    pub(super) accept: Rc<dyn Fn(T) -> bool>,
    pub(super) check: TypedCheck,
    pub(super) valid: Option<Signal<bool>>,
    pub(super) on_write: Option<Callback<Option<T>>>,
}

impl<T: 'static> DateFieldOptions<T> {
    /// `parse` reads typed text; `accept` is the bounds. `TypedCheck::OnCommit`.
    pub fn new(
        parse: impl Fn(&str) -> Option<T> + 'static,
        accept: impl Fn(T) -> bool + 'static,
    ) -> Self {
        Self {
            parse: Rc::new(parse),
            accept: Rc::new(accept),
            check: TypedCheck::OnCommit,
            valid: None,
            on_write: None,
        }
    }

    pub fn check(mut self, check: TypedCheck) -> Self {
        self.check = check;
        self
    }

    /// Write whether the typed text is a valid optional value after every change.
    pub fn valid(mut self, valid: Option<Signal<bool>>) -> Self {
        self.valid = valid;
        self
    }

    /// Call `f` with each value the field writes (not with external writes). Setup only.
    #[track_caller]
    pub fn on_write<R: HandlerResult>(mut self, f: impl Fn(Option<T>) -> R + 'static) -> Self {
        self.on_write = Some(callback(f));
        self
    }
}
