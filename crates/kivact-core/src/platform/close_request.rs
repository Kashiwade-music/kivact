//! Role: Owner-scoped handlers that may veto closing the window they were registered in.
//!
//! Registrations are passive runtime state, held the way lifecycle
//! subscriptions are: each lives as long as its owner. The client's close
//! dispatch takes a snapshot in registration order and runs each handler
//! under its owner; this module decides nothing about the outcome.

use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::{Handler, IntoHandler, Owner};

/// One `on_close_request` handler and the owner it runs under.
pub(crate) struct Registration {
    pub(crate) owner: Owner,
    pub(crate) handler: Handler<()>,
}

/// Ask before the window this component is in closes: the handler runs when
/// the user asks to close that window (title bar button, Alt+F4) and keeps
/// it open if it returns `Handled::Yes` or `Handled::PreventDefault`, or
/// fails (the error is reported). Return `()` or `Handled::No` to let it
/// close. In the main window a close ends the App; in a `window` or
/// `foreign_window` it then runs the window's `on_dismiss`. Every live
/// handler of the window runs, in registration order.
///
/// Call it in setup; it is removed when the calling component unmounts.
/// Closing with [`AppHandle::close`](crate::platform::AppHandle::close)
/// and an embedded App never run it.
///
/// ```no_run
/// # use kivact_core as kivact;
/// use kivact::platform::on_close_request;
/// use kivact::state::signal;
/// use kivact::view::{Element, Handled, IntoView, text};
///
/// fn app() -> impl IntoView {
///     let dirty = signal(true);
///     on_close_request(move || {
///         if dirty.get() {
///             // Open a "save changes?" dialog here; once the user decides,
///             // `platform::app_handle().close()` ends the App.
///             Handled::Yes
///         } else {
///             Handled::No
///         }
///     });
///     text("Editing")
/// }
/// ```
#[track_caller]
pub fn on_close_request<M>(handler: impl IntoHandler<(), M>) {
    let at = Location::caller();
    let owner = Owner::setup("on_close_request", at);
    let window = crate::view::state::current_window(owner);
    let registration = Rc::new(Registration {
        owner,
        handler: handler.into_handler(),
    });
    let rt = owner.runtime();
    let handlers = crate::view::state::ViewState::of(&rt)
        .with_window(window, |record| record.close_handlers.clone())
        .expect("invariant: the window of a mounting owner is recorded");
    handlers.borrow_mut().insert(&registration);
    owner.on_cleanup(move || drop(registration));
}
