//! Role: Key events: the handler route up the tree (stopped by an open
//! modal), the default actions `Handled::No` leaves (Enter/Space activation,
//! Escape dismissal), and the report of a press nobody used.

use kivact_engine::contract::HostEvent;
use kivact_engine::contract::HostRequest;
use kivact_runtime::Runtime;
use kivact_types::{
    A11yRequest, DismissReason, KeyCode, KeyPhase, NodeId, RegionId, RequestId, Role, WindowId,
};

use super::a11y::run_a11y;
use super::common::{handler, is_disabled, node_owner, run_handler, stale};
use super::pointer::activate;
use crate::view::handler_layers::EventHandlers;
use crate::view::input::{Handled, Key as UKey, KeyEvent, Modifiers};
use crate::view::overlays;
use crate::view::state::ViewState;

pub(super) fn key(rt: &Runtime, host_event: &HostEvent) {
    let HostEvent::Key(routed) = host_event else {
        unreachable!("dispatch routes key input here");
    };
    let node = routed.target.map(|target| target.node);
    let region = routed.target.and_then(|target| target.region);
    let window = routed.window;
    let phase = &routed.input.phase;
    let released = matches!(phase, KeyPhase::Up);
    let route = overlays::key_route(rt, window, node);
    let target = route.target();
    if ViewState::tree(rt).get(target).is_none() {
        if !released {
            report_unhandled(rt, window, routed.input.code);
        }
        return stale(rt, "key", target);
    }
    let mut handler_route = routed.clone();
    if Some(target) != node
        && let Some(route) = &mut handler_route.target
    {
        route.region = None;
    }
    let e = KeyEvent {
        routed: handler_route,
        target_role: target_role(rt, target, region),
    };
    let handled = run_handlers(rt, &e, released, target, route);
    let default_ran =
        handled == Handled::No && run_defaults(rt, &e, phase, released, target, route, window);
    if !released && handled == Handled::No && !default_ran {
        report_unhandled(rt, window, routed.input.code);
    }
}

/// Run the `on_key`/`on_key_up` handlers up the route; a failed handler stops
/// it like `Yes`.
fn run_handlers(
    rt: &Runtime,
    e: &KeyEvent,
    released: bool,
    target: NodeId,
    route: overlays::KeyRoute,
) -> Handled {
    let mut handled = Handled::No;
    let mut cur = Some(target);
    while let Some(id) = cur {
        if let Some(h) = handler(rt, id, |h| {
            if released {
                h.key_up.if_set()
            } else {
                h.key.if_set()
            }
        }) && !is_disabled(rt, id)
        {
            let outcome = run_key_handler(rt, id, h, e.clone(), released);
            handled = handled.and(outcome);
            if handled == Handled::Yes {
                break;
            }
        }
        if Some(id) == route.stop() {
            break; // keys do not leave an open modal
        }
        cur = ViewState::tree(rt).reader().parent(id);
    }
    handled
}

/// Run the default actions `Handled::No` leaves; whether any ran.
fn run_defaults(
    rt: &Runtime,
    e: &KeyEvent,
    phase: &KeyPhase,
    released: bool,
    target: NodeId,
    route: overlays::KeyRoute,
    window: WindowId,
) -> bool {
    let mut ran = false;
    if route.activates()
        && matches!(phase, KeyPhase::Down { repeat: false, .. })
        && matches!(e.key(), UKey::Enter | UKey::Space)
        && e.modifiers() == Modifiers::default()
    {
        // Default action of the focused control (Enter/Space → Activate).
        let region = e
            .origin()
            .filter(|origin| origin.node == target)
            .and_then(|origin| origin.region);
        ran |= activate(rt, target, region);
    }
    if !released && let Some(o) = overlays::escape_dismisses(rt, window, e) {
        // Nobody handled Escape: dismiss the topmost overlay.
        run_a11y(rt, o, None, A11yRequest::Dismiss(DismissReason::Escape));
        ran = true;
    }
    ran
}

/// Tell the host that nobody handled the press of `code` (`HostRequest::UnhandledKey`).
fn report_unhandled(rt: &Runtime, window: WindowId, code: KeyCode) {
    let id = ViewState::of(rt).requests.borrow_mut().request_id();
    ViewState::tree(rt)
        .engine()
        .effect(RequestId(id), HostRequest::UnhandledKey { window, code });
    rt.request_turn();
}

/// The role of the routed region on `target`, or the node's own role.
fn target_role(rt: &Runtime, target: NodeId, region: Option<RegionId>) -> Option<Role> {
    region
        .and_then(|region| {
            let tree = ViewState::tree(rt);
            [
                kivact_engine::contract::SceneLayer::Draw,
                kivact_engine::contract::SceneLayer::DrawOver,
            ]
            .into_iter()
            .filter_map(|layer| tree.scene(target, layer))
            .find_map(|scene| {
                scene
                    .regions
                    .iter()
                    .find(|r| r.id == region)
                    .and_then(|r| r.semantics.role)
            })
        })
        .or_else(|| ViewState::tree(rt).role(target))
}

/// Run one key handler and record its outcome for the DevTools input log.
fn run_key_handler(
    rt: &Runtime,
    id: NodeId,
    handler: EventHandlers<KeyEvent>,
    event: KeyEvent,
    released: bool,
) -> Handled {
    let outcome = run_handler(rt, node_owner(rt, id), || handler.deliver(event));
    crate::view::devtools_hooks::handler_ran(
        rt,
        id,
        if released { "on_key_up" } else { "on_key" },
        outcome,
    );
    outcome.unwrap_or(Handled::Yes)
}
