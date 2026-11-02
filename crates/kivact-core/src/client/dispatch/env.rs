//! Role: Window and environment events: the environment, the frame clock,
//! close, and the geometry reports (`ScrollChanged`, `LayoutChanged`).

use kivact_motion::MotionState;
use kivact_reactive::Runtime;
use kivact_wire::{HostEnv, MotionClock, NodeId, Point, Rect, Size};

use super::common::{handler, node_owner, run_handler, stale};
use crate::client::state::HostState;
use crate::view::state::ViewState;
use crate::view::tree::ScrollState;

/// The host reports its environment, whole: the one signal changes, and
/// only the readers of the fields that differ run again.
pub(super) fn env_changed(rt: &Runtime, env: &HostEnv) {
    let state = ViewState::of(rt).env();
    // Only the env signal is written here: no owner applies.
    rt.enter(|| state.set(env.clone()));
    // Reduced motion ends the running decorative values in this turn, not
    // at the next tick: nothing needs another frame to see it.
    if env.reduced_motion {
        MotionState::of(rt).reduce_motion(rt);
    }
}

pub(super) fn frame_tick(
    rt: &Runtime,
    now_ns: u64,
    estimated_present_ns: u64,
    clock: &MotionClock,
) {
    HostState::of(rt).advance_clock(now_ns);
    MotionState::of(rt).on_frame_tick(rt, now_ns, estimated_present_ns, clock);
}

pub(super) fn scroll_changed(rt: &Runtime, node: NodeId, offset: Point, viewport: Size) {
    if let Some(h) = handler(rt, node, |h| h.scroll.clone()) {
        let s = ScrollState { offset, viewport };
        run_handler(rt, node_owner(rt, node), || {
            h(s);
            Ok(())
        });
    }
}

pub(super) fn layout_changed(rt: &Runtime, node: NodeId, rect: Rect, size: Size) {
    if ViewState::tree(rt).get(node).is_none() {
        return stale(rt, "layout", node);
    }
    if let Some(h) = handler(rt, node, |h| h.layout.clone()) {
        run_handler(rt, node_owner(rt, node), || {
            h(rect, size);
            Ok(())
        });
    }
}

pub(super) fn close_requested(rt: &Runtime) {
    HostState::of(rt).request_close(rt.now_ns());
}
