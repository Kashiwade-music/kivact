//! Role: Input method events for a node's IME session.

use kivact_engine::contract::HostEvent;
use kivact_runtime::Runtime;
use kivact_types::{ImeInput, NodeId};

use super::common::{admitted, handler, is_disabled, node_owner, run_handler, stale};
use crate::view::state::ViewState;

/// Deliver an IME `event` to the handlers of `node`, which has an IME
/// session. A failed handler is reported once, like any other.
pub(super) fn ime(rt: &Runtime, host_event: &HostEvent, node: NodeId, event: &ImeInput) {
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "ime", node);
    }
    if !admitted(rt, host_event, "ime", node) || is_disabled(rt, node) {
        return;
    }
    if let Some(handlers) = handler(rt, node, |h| h.ime.if_set()) {
        let event = event.clone();
        run_handler(rt, node_owner(rt, node), || handlers.deliver(event));
    }
}
