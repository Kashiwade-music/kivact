//! Role: What the host event modules share: the stale and modal drops,
//! the lookup of a node's handler and owner, and the failure-safe handler run.

use kivact_engine::contract::HostEvent;
use kivact_reactive::{Owner, Runtime, UiError};
use kivact_types::{KeyPhase, NodeId, PointerPhase};

use crate::client::state::HostState;
use crate::view::state::ViewState;
use crate::view::tree::Handlers;

/// Note the input modality: a pointer press switches to pointer, a key
/// press to keyboard.
pub(super) fn track_modality(rt: &Runtime, host_event: &HostEvent) {
    match host_event {
        HostEvent::Pointer(route) if matches!(route.input.phase, PointerPhase::Down(_)) => {
            ViewState::of(rt).set_pointer_modality(true)
        }
        HostEvent::Key(route) if matches!(route.input.phase, KeyPhase::Down { .. }) => {
            ViewState::of(rt).set_pointer_modality(false)
        }
        _ => {}
    }
}

/// Note the modifier keys the host reported last (`view::current_modifiers`):
/// key, pointer and wheel events carry them, and `ModifiersChanged` reports a
/// change between events. Recorded before admission, so an event a modal
/// drops still updates them.
pub(super) fn track_modifiers(rt: &Runtime, host_event: &HostEvent) {
    let modifiers = match host_event {
        HostEvent::Key(route) => route.input.modifiers,
        HostEvent::Wheel(route) => route.input.modifiers,
        HostEvent::Pointer(route) => route.input.modifiers,
        HostEvent::ModifiersChanged(modifiers) => *modifiers,
        _ => return,
    };
    ViewState::of(rt).set_modifiers(modifiers);
}

pub(super) fn node_owner(rt: &Runtime, id: NodeId) -> Option<Owner> {
    ViewState::tree(rt).get(id).map(|n| n.owner)
}

pub(super) fn stale(rt: &Runtime, what: &str, id: NodeId) {
    let host = HostState::of(rt);
    host.stale_events.set(host.stale_events.get() + 1);
    rt.note(format!("stale {what} event for removed node {}", id.0));
}

/// Whether the engine admits `event`, aimed at `id`, now; an event it does
/// not admit is dropped and counted like a stale event. Asked exactly once
/// per event, immediately before the event is dispatched, so a modal an
/// earlier event of the turn opened already applies (and the engine records
/// the pointer drag the event admits).
pub(super) fn admitted(rt: &Runtime, event: &HostEvent, what: &str, id: NodeId) -> bool {
    if ViewState::tree(rt).reader().admit_input(event) {
        return true;
    }
    drop_outside_modal(rt, what, id);
    false
}

/// Count and note an event for `id` dropped because a modal is open.
pub(super) fn drop_outside_modal(rt: &Runtime, what: &str, id: NodeId) {
    let host = HostState::of(rt);
    host.stale_events.set(host.stale_events.get() + 1);
    rt.note(format!(
        "{what} event for node {} outside the open modal overlay dropped",
        id.0
    ));
}

/// Run a handler untracked; an `Err` goes to the node owner's sink. `None`
/// means failure, so input routes stop rather than falling through.
pub(super) fn run_handler<R>(
    rt: &Runtime,
    owner: Option<Owner>,
    f: impl FnOnce() -> Result<R, UiError>,
) -> Option<R> {
    let run = || match f() {
        Ok(r) => Some(r),
        Err(e) => {
            rt.report(owner, e);
            None
        }
    };
    match owner {
        Some(owner) => owner.run_handler(run),
        None => rt.enter(run),
    }
}

pub(super) fn handler<T: Clone>(
    rt: &Runtime,
    id: NodeId,
    f: impl Fn(&Handlers) -> Option<T>,
) -> Option<T> {
    let tree = ViewState::tree(rt);
    tree.get(id).and_then(|n| n.handlers.as_deref()).and_then(f)
}

pub(super) fn is_disabled(rt: &Runtime, id: NodeId) -> bool {
    ViewState::tree(rt).disabled(id) == Some(true)
}
