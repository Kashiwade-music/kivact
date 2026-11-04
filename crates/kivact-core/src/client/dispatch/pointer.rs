//! Role: Pointer, wheel, hover, file-drop and activation events.

use kivact_engine::contract::HostEvent;
use kivact_reactive::Runtime;
use kivact_style::draw::RegionId;
use kivact_types::{FilePhase, NodeId, Point};

use super::common::{admitted, handler, is_disabled, node_owner, run_handler, stale};
use crate::view::input::{Handled, PointerEvent, WheelEvent};
use crate::view::requests;
use crate::view::state::ViewState;

pub(super) fn activate(rt: &Runtime, id: NodeId, region: Option<RegionId>) -> bool {
    if is_disabled(rt, id) {
        return false;
    }
    let Some(h) = handler(rt, id, |h| h.activate.clone()) else {
        return false;
    };
    run_handler(rt, node_owner(rt, id), || h(region));
    true
}

/// Run the activations `NodeRef::activate` asked for (`requests`): the
/// default action of each target that is still mounted, enabled and
/// admitted by the open modal, as a click would run it. Returns whether
/// there were any. An activation asked for by one of them waits for the
/// next call.
pub(crate) fn run_requested_activations(rt: &Runtime) -> bool {
    let pending = requests::take_activations(rt);
    let any = !pending.is_empty();
    for target in pending {
        let in_scope = ViewState::tree(rt).reader().in_modal_scope(target.node);
        if ViewState::tree(rt).get(target.node).is_none() || !in_scope {
            continue;
        }
        activate(rt, target.node, target.region);
    }
    any
}

pub(super) fn pointer(
    rt: &Runtime,
    host_event: &HostEvent,
    routed: kivact_engine::contract::Routed<kivact_types::PointerSample>,
) {
    let Some(node) = routed.target else {
        return;
    };
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "pointer", node);
    }
    if !admitted(rt, host_event, "pointer", node) {
        return;
    }
    let event = routed.input;
    let e = PointerEvent { routed };
    let found = {
        let tree = ViewState::tree(rt);
        let stop = tree.modal_boundary(node);
        tree.find_up(node, stop, |n| {
            n.handlers.as_ref().and_then(|h| h.pointer.if_set())
        })
    };
    if let Some((id, h)) = found
        && (!is_disabled(rt, id)
            || matches!(
                event.phase,
                kivact_types::PointerPhase::Enter
                    | kivact_types::PointerPhase::Leave
                    | kivact_types::PointerPhase::Cancel
            ))
    {
        let handled = run_handler(rt, node_owner(rt, id), || h.deliver(e));
        if handled != Some(Handled::No) {
            return;
        }
    }
    if matches!(
        e.phase(),
        crate::view::PointerPhase::Down(press) | crate::view::PointerPhase::Up(press)
            if press.button == crate::view::MouseButton::Secondary
    ) {
        let fallback = {
            let tree = ViewState::tree(rt);
            let stop = tree.modal_boundary(node);
            tree.find_up(node, stop, |n| {
                n.handlers
                    .as_ref()
                    .and_then(|h| h.secondary_context.clone())
            })
        };
        if let Some((id, handler)) = fallback
            && !is_disabled(rt, id)
        {
            run_handler(rt, node_owner(rt, id), || handler(e));
        }
    }
}

pub(super) fn wheel(
    rt: &Runtime,
    host_event: &HostEvent,
    routed: kivact_engine::contract::Routed<kivact_types::WheelInput>,
) {
    let Some(node) = routed.target else {
        return;
    };
    let delta = routed.input.delta;
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "wheel", node);
    }
    if !admitted(rt, host_event, "wheel", node) {
        return;
    }
    let e = WheelEvent { routed };
    let stop = ViewState::tree(rt).modal_boundary(node);
    let mut cur = Some(node);
    let mut handled = Handled::No;
    while let Some(id) = cur {
        let h = handler(rt, id, |h| h.wheel.if_set());
        if let Some(h) = h
            && !is_disabled(rt, id)
        {
            handled = handled
                .and(run_handler(rt, node_owner(rt, id), || h.deliver(e)).unwrap_or(Handled::Yes));
            if handled == Handled::Yes {
                break;
            }
        }
        if Some(id) == stop {
            break; // wheels do not leave an open modal
        }
        cur = ViewState::tree(rt).get(id).and_then(|n| n.parent);
    }
    if handled == Handled::No && ViewState::tree(rt).get(node).is_some() {
        // Default action: the host continues its native scroll
        // from the listener upward (it never scrolls past a wheel
        // listener on its own, so `Handled::Yes` and
        // `Handled::PreventDefault` really stop it, nor out of an
        // open modal the listener lies in).
        ViewState::tree(rt).engine().scroll_by(node, delta);
    }
}

pub(super) fn file(
    rt: &Runtime,
    host_event: &HostEvent,
    node: NodeId,
    phase: FilePhase,
    files: &[std::path::PathBuf],
    window_pos: Point,
) {
    if ViewState::tree(rt).get(node).is_none() || !admitted(rt, host_event, "file", node) {
        return;
    }
    let stop = ViewState::tree(rt).modal_boundary(node);
    let found = ViewState::tree(rt).find_up(node, stop, |n| {
        n.handlers.as_ref().and_then(|h| h.file.if_set())
    });
    if let Some((id, handler)) = found {
        let local = ViewState::tree(rt).reader().local_point(id, window_pos);
        let event = crate::view::FileEvent {
            phase,
            files: files.to_vec(),
            pos: local,
            window_pos,
        };
        run_handler(rt, node_owner(rt, id), || handler.deliver(event));
    }
}

pub(super) fn activate_event(
    rt: &Runtime,
    host_event: &HostEvent,
    node: NodeId,
    region: Option<RegionId>,
    pointer: Option<crate::view::PointerId>,
) {
    // A click whose gesture a canvas adopted (`part_pointer`) activates
    // nothing; the engine remembers which, and answers once.
    if let Some(pointer) = pointer
        && ViewState::tree(rt)
            .reader()
            .take_adopted_activation(node, region, pointer)
    {
        return;
    }
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "activate", node);
    }
    if !admitted(rt, host_event, "activate", node) {
        return;
    }
    activate(rt, node, region);
}

pub(super) fn hover_changed(rt: &Runtime, node: Option<NodeId>) {
    crate::view::hover::set_hovered(rt, node);
}
