//! Role: Shared event outcomes and erased effectful handlers.

use crate::reactive::error::UiError;
use std::rc::Rc;

/// What a key/pointer/wheel callback did with its event.
///
/// Like a React (DOM) handler: `Yes` is `stopPropagation()` together with
/// `preventDefault()`, `No` does neither, and `PreventDefault` is
/// `preventDefault()` alone. The default actions are the framework's, run
/// once after the whole chain when no callback prevented them:
///
/// - key press: Enter/Space activates the focused control or region, and
///   an Escape dismisses the topmost overlay offering `Dismiss`. Tab focus
///   moves and native editing happen in the host before the callbacks and
///   are not affected. Key releases have no default action.
/// - wheel: the scroll container around the listener scrolls on.
/// - pointer and file: none. The host has already moved the focus and
///   taken the capture when the callbacks run, and a click's activation is
///   a separate Activate, so `PreventDefault` behaves as `No` there.
///
/// Use `PreventDefault` when a component reacts to a key but an ancestor's
/// shortcut must still see it, for example a pan surface that tracks a
/// held Space without activating the focused control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handled {
    /// Consumed: later callbacks and ancestors do not see the event, and
    /// no default action runs.
    Yes,
    /// Not handled: the event continues to later callbacks and ancestors,
    /// then to its default action.
    No,
    /// The event continues to later callbacks and ancestors exactly as with
    /// `No`, but its default action does not run. A later `Yes` still stops
    /// it; a later `No` does not undo the prevention.
    PreventDefault,
}

impl Handled {
    /// The answer of a chain where `self` and then `next` saw the event:
    /// `Yes` wins, and a prevented default stays prevented.
    pub fn and(self, next: Handled) -> Handled {
        match (self, next) {
            (Handled::Yes, _) | (_, Handled::Yes) => Handled::Yes,
            (Handled::PreventDefault, _) | (_, Handled::PreventDefault) => Handled::PreventDefault,
            (Handled::No, Handled::No) => Handled::No,
        }
    }
}

/// A shared effectful event handler.
pub struct Handler<E>(Rc<dyn Fn(E) -> Result<Handled, UiError>>);
impl<E> Clone for Handler<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<E: 'static> Handler<E> {
    /// Erase a handler, preserving errors and letting successful events continue.
    pub fn from_fn<R: HandlerResult>(f: impl Fn(E) -> R + 'static) -> Self {
        Self(Rc::new(move |event| f(event).into_outcome()))
    }
    /// Invoke the handler.
    pub fn call(&self, event: E) -> Result<Handled, UiError> {
        (self.0)(event)
    }
}

/// Return values accepted by effectful handlers.
#[diagnostic::on_unimplemented(
    message = "a handler returns `()`, `Handled`, `Result<(), UiError>` or `Result<Handled, UiError>`",
    note = "use `?` on other errors; they convert to `UiError`"
)]
pub trait HandlerResult: crate::reactive::sealed::HandlerResultSeal {
    #[doc(hidden)]
    fn into_outcome(self) -> Result<Handled, UiError>;
}
impl HandlerResult for () {
    fn into_outcome(self) -> Result<Handled, UiError> {
        Ok(Handled::No)
    }
}
impl HandlerResult for Handled {
    fn into_outcome(self) -> Result<Handled, UiError> {
        Ok(self)
    }
}
impl HandlerResult for Result<(), UiError> {
    fn into_outcome(self) -> Result<Handled, UiError> {
        self.map(|()| Handled::No)
    }
}
impl HandlerResult for Result<Handled, UiError> {
    fn into_outcome(self) -> Result<Handled, UiError> {
        self
    }
}
impl crate::reactive::sealed::HandlerResultSeal for () {}
impl crate::reactive::sealed::HandlerResultSeal for Handled {}
impl crate::reactive::sealed::HandlerResultSeal for Result<(), UiError> {}
impl crate::reactive::sealed::HandlerResultSeal for Result<Handled, UiError> {}
