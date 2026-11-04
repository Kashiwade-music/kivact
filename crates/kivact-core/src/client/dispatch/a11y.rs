//! Role: Assistive-technology requests and overlay dismissals, both run
//! through the node's `a11y` handlers.

use kivact_engine::contract::HostEvent;
use kivact_runtime::Runtime;
use kivact_style::draw::RegionId;
use kivact_types::{A11yRequest, NodeId};

use super::common::{admitted, handler, is_disabled, node_owner, run_handler, stale};
use super::pointer::activate;
use crate::view::A11yEvent;
use crate::view::state::ViewState;

/// Run the handler of `request`'s action on `node` (not `Activate`), unless
/// the node is disabled.
pub(super) fn run_a11y(rt: &Runtime, node: NodeId, region: Option<RegionId>, request: A11yRequest) {
    let action = request.action();
    let h = handler(rt, node, |h| {
        h.a11y
            .iter()
            .find(|(a, _)| *a == action)
            .map(|(_, f)| f.clone())
    });
    if let Some(h) = h
        && !is_disabled(rt, node)
    {
        run_handler(rt, node_owner(rt, node), || {
            h.call(A11yEvent { request, region })
        });
    }
}

pub(super) fn a11y(
    rt: &Runtime,
    host_event: &HostEvent,
    node: NodeId,
    region: Option<RegionId>,
    request: &A11yRequest,
) {
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "a11y", node);
    }
    if !admitted(rt, host_event, "a11y", node) {
        return;
    }
    if *request == A11yRequest::Activate {
        activate(rt, node, region);
        return;
    }
    run_a11y(rt, node, region, request.clone());
}
