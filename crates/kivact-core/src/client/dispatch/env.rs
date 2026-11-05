//! Role: Window and environment events: the environment, the frame clock,
//! close, and the geometry reports (`ScrollChanged`, `LayoutChanged`).

use kivact_runtime::MotionState;
use kivact_runtime::Runtime;
use kivact_types::{Environment, MotionClock, NodeId, Point, Rect, Size};

use super::common::{handler, node_owner, run_handler, stale};
use crate::view::state::ViewState;
use crate::view::tree::ScrollState;

/// The host reports its environment, whole: the one signal changes, and
/// only the readers of the fields that differ run again.
pub(super) fn env_changed(rt: &Runtime, env: &Environment, surface_active: bool) {
    let state = ViewState::of(rt).env();
    // Only the env signal is written here: no owner applies.
    rt.enter(|| {
        state.set(env.clone());
        state.surface_active.set(surface_active);
        let families = ViewState::tree(rt).reader().font_families();
        ViewState::of(rt).fonts_value.set(families);
    });
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
    rt.advance_clock(now_ns);
    MotionState::of(rt).on_frame_tick(rt, now_ns, estimated_present_ns, clock);
}

pub(super) fn scroll_changed(
    rt: &Runtime,
    node: NodeId,
    offset: Point,
    viewport: Size,
    content: Size,
) {
    if let Some(h) = handler(rt, node, |h| h.scroll.clone()) {
        let content = ViewState::tree(rt).reader().with_node(node, |props| {
            let Some(layout) = props.and_then(|props| props.layout.as_deref()) else {
                return viewport;
            };
            Size::new(
                if layout.overflow_x == kivact_types::Overflow::Scroll {
                    content.width
                } else {
                    viewport.width
                },
                if layout.overflow_y == kivact_types::Overflow::Scroll {
                    content.height
                } else {
                    viewport.height
                },
            )
        });
        let s = ScrollState {
            offset,
            viewport,
            content,
        };
        run_handler(rt, node_owner(rt, node), || {
            h.call(s)?;
            Ok(())
        });
    }
}

pub(super) fn layout_changed(
    rt: &Runtime,
    node: NodeId,
    rect: Rect,
    size: Size,
    text_truncated: bool,
) {
    let observation = {
        let tree = ViewState::tree(rt);
        let Some(node) = tree.get(node) else {
            return stale(rt, "layout", node);
        };
        node.observation
    };
    let layout = handler(rt, node, |h| h.layout.clone());
    if observation.is_some() || layout.is_some() {
        run_handler(rt, node_owner(rt, node), || {
            if let Some(observation) = observation {
                observation.text_truncated.set(text_truncated);
            }
            if let Some(layout) = layout {
                layout.call((rect, size))?;
            }
            Ok(())
        });
    }
}

pub(super) fn close_requested(rt: &Runtime) {
    rt.request_exit();
}
