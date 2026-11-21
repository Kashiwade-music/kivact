//! Role: The host states a node's style asked for, as one reactive source per node (ST-05).
//!
//! The engine reports them (`HostEvent::StatesChanged`) only for a node whose
//! compiled state styles set more than paint; the node's style effect reads
//! the source and compiles those declarations in.

use std::panic::Location;

use kivact_runtime::driver::Runtime;
use kivact_runtime::{Owner, Signal};
use kivact_types::{NodeId, StateSet};

use crate::view::state::ViewState;

/// The source of `id`'s reported states, made under `owner` (the node's
/// owner) on the first call and dropped with it.
pub(crate) fn source(
    rt: &Runtime,
    id: NodeId,
    owner: Owner,
    at: &'static Location<'static>,
) -> Signal<StateSet> {
    if let Some(sig) = ViewState::of(rt).node_states.borrow().get(&id).copied() {
        return sig;
    }
    let sig = owner.signal(StateSet::EMPTY, at);
    ViewState::of(rt).node_states.borrow_mut().insert(id, sig);
    let app = rt.id();
    owner.on_cleanup(move || {
        if let Some(rt) = Runtime::get(app) {
            ViewState::of(&rt).node_states.borrow_mut().remove(&id);
        }
    });
    sig
}

/// The host reported `states` for `node`: set its source. A node without
/// one (it does not ask for reports, or it is gone) is ignored.
pub(crate) fn states_changed(rt: &Runtime, node: NodeId, states: StateSet) {
    let sig = ViewState::of(rt).node_states.borrow().get(&node).copied();
    if let Some(sig) = sig {
        rt.enter(|| sig.set(states));
    }
}
