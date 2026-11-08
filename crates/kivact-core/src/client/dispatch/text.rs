//! Role: Text-editing and focus events.

use kivact_engine::contract::HostEvent;
use kivact_runtime::Runtime;
use kivact_style::draw::RegionId;
use kivact_types::{NodeId, RequestId};

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
    notify_geometry(rt, node);
    let phase = handler(rt, node, |h| h.edit_phase.clone());
    let committed = session.marked().is_none();
    if committed && let Some(observer) = &phase {
        run_handler(rt, node_owner(rt, node), || {
            observer.call(crate::view::EditPhase::Begin)
        });
    }
    if let Some(h) = handler(rt, node, |h| h.text.clone()) {
        let edit = crate::custom::TextEditEvent {
            changes: changes.to_vec(),
            session: session.clone(),
            text: text.map(str::to_owned),
        };
        run_handler(rt, node_owner(rt, node), || h.call(edit));
    }
    if committed && let Some(observer) = &phase {
        run_handler(rt, node_owner(rt, node), || {
            observer.call(crate::view::EditPhase::Commit)
        });
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
    notify_geometry(rt, node);
    if let Some(h) = handler(rt, node, |h| h.selection.clone()) {
        let session = session.clone();
        run_handler(rt, node_owner(rt, node), || h.call(session));
    }
}

pub(super) fn focus_changed(
    rt: &Runtime,
    window: kivact_types::WindowId,
    node: Option<NodeId>,
    region: Option<RegionId>,
    visible: bool,
) {
    crate::view::focus::set_focused(
        rt,
        window,
        node.map(|node| kivact_types::FocusTarget { node, region }),
        visible,
    );
}

pub(super) fn focus_result(rt: &Runtime, request: RequestId, result: Result<(), String>) {
    crate::view::requests::on_focus_result(rt, request.0, result);
}

fn notify_geometry(rt: &Runtime, node: NodeId) {
    let observation = ViewState::tree(rt)
        .get(node)
        .and_then(|node| node.observation);
    if let Some(observation) = observation {
        observation.editing_revision.notify();
    }
}
