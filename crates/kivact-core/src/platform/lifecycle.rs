//! Role: Owner-scoped application lifecycle subscriptions and ordered dispatch.
//!
//! Registrations are passive runtime state. Dispatch snapshots registration
//! order, releases the registry borrow and invokes existing owner-aware callbacks.
//! Disposed owners are skipped even if disposed during an earlier callback.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::{Callback, HandlerResult, Owner, Runtime};
pub use kivact_types::LifecycleEvent;
use kivact_types::cell::{Generation, Registry};

#[derive(Default)]
pub(crate) struct LifecycleState {
    generation: RefCell<Generation>,
    registrations: RefCell<Registry<Registration>>,
}

struct Registration {
    _id: Generation,
    owner: Owner,
    callback: Callback<LifecycleEvent>,
}

/// Subscribe in setup until this component's owner is disposed.
///
/// Notifications run untracked in registration order in their registering
/// owners. Errors reach each owner's error boundary. The first state is
/// resumed, with no synthetic initial callback. Conditional removals requested
/// by callbacks settle after the event: already-disposed owners are skipped,
/// while pending removals suppress subsequent events. Surface loss and visibility
/// changes are observed through `surface_active`, not lifecycle transitions.
#[track_caller]
pub fn on_lifecycle<R: HandlerResult>(f: impl Fn(LifecycleEvent) -> R + 'static) {
    let owner = Owner::current("on_lifecycle", Location::caller());
    let rt = owner.runtime();
    let state = rt.ext::<LifecycleState>();
    let id = state.generation.borrow_mut().next();
    let registration = Rc::new(Registration {
        _id: id,
        owner,
        callback: kivact_runtime::callback(f),
    });
    state.registrations.borrow_mut().insert(&registration);
    owner.on_cleanup(move || drop(registration));
}

pub(crate) fn dispatch(rt: &Runtime, event: LifecycleEvent) {
    let registrations: Vec<_> = rt.ext::<LifecycleState>().registrations.borrow_mut().live();
    for registration in registrations {
        if registration.owner.is_alive() {
            registration.callback.call(event);
        }
    }
}
