//! Role: Text-editing and focus events.

use kivact_reactive::Runtime;
use kivact_style::draw::RegionId;
use kivact_wire::{HostEvent, NodeId, RequestId};

use super::common::{admitted, handler, node_owner, run_handler, stale};
use crate::view::state::ViewState;

pub(super) fn text_changed(
    rt: &Runtime,
    host_event: &HostEvent,
    node: NodeId,
    changes: &[kivact_editing::TextChange],
    session: &kivact_editing::TextSession,
    text: Option<&str>,
) {
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "text", node);
    }
    if !admitted(rt, host_event, "text", node) {
        return;
    }
    if let Some(h) = handler(rt, node, |h| h.text.clone()) {
        let edit = crate::primitive::TextEditEvent {
            changes: changes.to_vec(),
            session: session.clone(),
            text: text.map(str::to_owned),
        };
        run_handler(rt, node_owner(rt, node), || h(edit));
    }
}

pub(super) fn selection_changed(
    rt: &Runtime,
    host_event: &HostEvent,
    node: NodeId,
    session: &kivact_editing::TextSession,
) {
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "selection", node);
    }
    if !admitted(rt, host_event, "selection", node) {
        return;
    }
    if let Some(h) = handler(rt, node, |h| h.selection.clone()) {
        let session = session.clone();
        run_handler(rt, node_owner(rt, node), || h(session));
    }
}

pub(super) fn focus_changed(
    rt: &Runtime,
    node: Option<NodeId>,
    region: Option<RegionId>,
    visible: bool,
) {
    crate::view::focus::set_focused(
        rt,
        node.map(|node| kivact_wire::FocusTarget { node, region }),
        visible,
    );
}

pub(super) fn focus_result(rt: &Runtime, request: RequestId, ok: bool, reason: Option<&str>) {
    crate::view::requests::on_focus_result(rt, request.0, ok, reason.map(str::to_owned));
}
