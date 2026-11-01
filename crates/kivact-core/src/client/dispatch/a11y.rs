//! Role: Assistive-technology actions and overlay dismissal, both run
//! through the node's `a11y` handlers.

use kivact_reactive::Runtime;
use kivact_style::draw::RegionId;
use kivact_wire::{A11yAction, HostEvent, NodeId};

use super::common::{admitted, handler, is_disabled, node_owner, run_handler, stale};
use super::pointer::activate;
use crate::view::state::ViewState;
use crate::view::{A11yEvent, DismissReason};

/// Run the handler of `action` on `node` (not `Activate`),
/// unless the node is disabled.
pub(super) fn run_a11y(
    rt: &Runtime,
    node: NodeId,
    action: A11yAction,
    region: Option<RegionId>,
    value: Option<String>,
    dismiss_reason: Option<DismissReason>,
) {
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
            h(A11yEvent {
                action,
                region,
                value,
                dismiss_reason,
            })
        });
    }
}

pub(super) fn a11y(
    rt: &Runtime,
    host_event: &HostEvent,
    node: NodeId,
    region: Option<RegionId>,
    action: A11yAction,
    value: &Option<String>,
) {
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "a11y", node);
    }
    if !admitted(rt, host_event, "a11y", node) {
        return;
    }
    if action == A11yAction::Activate {
        activate(rt, node, region);
        return;
    }
    run_a11y(
        rt,
        node,
        action,
        region,
        value.clone(),
        (action == A11yAction::Dismiss).then_some(DismissReason::Accessibility),
    );
}

pub(super) fn overlay_dismiss(rt: &Runtime, host_event: &HostEvent, node: NodeId) {
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "dismiss", node);
    }
    if !admitted(rt, host_event, "dismiss", node) {
        return;
    }
    run_a11y(
        rt,
        node,
        A11yAction::Dismiss,
        None,
        None,
        Some(DismissReason::OutsidePress),
    );
}
