//! Role: Owner-scoped application lifecycle subscriptions and ordered dispatch.
//!
//! Registrations are passive runtime state. Dispatch snapshots registration
//! order, releases the registry borrow and invokes existing owner-aware callbacks.
//! Disposed owners are skipped even if disposed during an earlier callback.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::panic::Location;
use std::rc::Rc;

use kivact_reactive::{Callback, HandlerResult, Owner, Runtime};
pub use kivact_wire::LifecycleEvent;

#[derive(Default)]
pub(crate) struct LifecycleState {
    next: Cell<u64>,
    registrations: RefCell<BTreeMap<u64, (Owner, Callback<LifecycleEvent>)>>,
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
    let id = state.next.get();
    state.next.set(id + 1);
    state
        .registrations
        .borrow_mut()
        .insert(id, (owner, kivact_reactive::callback(f)));
    let weak = Rc::downgrade(&rt);
    owner.on_cleanup(move || {
        if let Some(rt) = weak.upgrade() {
            rt.ext::<LifecycleState>()
                .registrations
                .borrow_mut()
                .remove(&id);
        }
    });
}

pub(crate) fn dispatch(rt: &Runtime, event: LifecycleEvent) {
    let registrations: Vec<_> = rt
        .ext::<LifecycleState>()
        .registrations
        .borrow()
        .values()
        .copied()
        .collect();
    for (owner, callback) in registrations {
        if owner.is_alive() {
            callback.call(event);
        }
    }
}
