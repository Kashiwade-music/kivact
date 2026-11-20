//! Role: Owner-scoped handlers that may veto closing the main window.
//!
//! Registrations are passive runtime state, held the way lifecycle
//! subscriptions are: each lives as long as its owner. The client's close
//! dispatch takes a snapshot in registration order and runs each handler
//! under its owner; this module decides nothing about the outcome.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::driver::Runtime;
use kivact_runtime::{Handler, HandlerResult, Owner};
use kivact_types::cell::Registry;

#[derive(Default)]
pub(crate) struct CloseRequestState {
    registrations: RefCell<Registry<Registration>>,
}

/// One `on_close_request` handler and the owner it runs under.
pub(crate) struct Registration {
    pub(crate) owner: Owner,
    pub(crate) handler: Handler<()>,
}

/// Ask before the App closes: the handler runs when the user asks to close the
/// main window (title bar button, Alt+F4), and the App keeps running if it
/// returns `Handled::Yes` or `Handled::PreventDefault`, or fails (the error is
/// reported). Return `()` or `Handled::No` to let it close. Every live handler
/// runs, in registration order, and the App stays open if any of them keeps it.
///
/// It is unregistered when the owner it was called in is cleaned up. Closing
/// with [`AppHandle::close`](crate::platform::AppHandle::close) and an
/// embedded App never run it.
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
pub fn on_close_request<R: HandlerResult>(f: impl Fn() -> R + 'static) {
    let owner = Owner::running("on_close_request", Location::caller());
    let registration = Rc::new(Registration {
        owner,
        handler: Handler::from_fn(move |()| f()),
    });
    crate::app_state::AppState::of(&owner.runtime())
        .close
        .registrations
        .borrow_mut()
        .insert(&registration);
    owner.on_cleanup(move || drop(registration));
}

/// The live registrations in registration order; dead ones are removed.
pub(crate) fn registrations(rt: &Runtime) -> Vec<Rc<Registration>> {
    crate::app_state::AppState::of(rt)
        .close
        .registrations
        .borrow_mut()
        .live()
}
