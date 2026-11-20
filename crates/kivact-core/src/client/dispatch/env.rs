//! Role: Window and environment events: the environment, the frame clock,
//! close, and the geometry reports (`ScrollChanged`, `LayoutChanged`).

use kivact_runtime::driver::Runtime;
use kivact_runtime::{ErrorKind, UiError};
use kivact_types::{DismissReason, Environment, MotionClock, NodeId, Point, Rect, Size, WindowId};

use super::common::{handler, node_owner, run_handler, stale};
use crate::platform::close_request;
use crate::view::input::Handled;
use crate::view::state::ViewState;
use crate::view::tree::ScrollState;

/// The host reports the environment of `window`, whole: the one signal
/// changes, and only the readers of the fields that differ run again.
pub(super) fn env_changed(rt: &Runtime, window: WindowId, env: &Environment, surface_active: bool) {
    let Some(state) = ViewState::of(rt).env(window) else {
        return;
    };
    let was_reduced = state.reduced_motion();
    // Only the env signal is written here: no owner applies.
    rt.enter(|| {
        state.set(env.clone());
        state.surface_active.set(surface_active);
    });
    ViewState::of(rt).refresh_fonts(rt);
    // Reduced motion ends the running decorative values in this turn, not
    // at the next tick: nothing needs another frame to see it. Turning it
    // off counts the decorative loops from now, as a resumed surface does.
    if env.reduced_motion {
        rt.motion().reduce_motion(rt);
    } else if was_reduced {
        rt.motion().restore_motion(rt);
    }
}

/// The host opened `window` with this environment.
pub(super) fn window_opened(
    rt: &Runtime,
    window: WindowId,
    env: &Environment,
    surface_active: bool,
) {
    let Some(state) = ViewState::of(rt).env(window) else {
        return;
    };
    rt.enter(|| {
        state.set(env.clone());
        state.surface_active.set(surface_active);
    });
}

pub(super) fn frame_tick(
    rt: &Runtime,
    now_ns: u64,
    estimated_present_ns: u64,
    clock: &MotionClock,
) {
    // The turn advanced the App's clock with host time; `now_ns` is the
    // frame (motion) clock and is not applied to the App's clock again.
    rt.motion()
        .on_frame_tick(rt, now_ns, estimated_present_ns, clock);
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

/// The engine reports a new visibility or live-region fact of a watched node.
pub(super) fn node_flags_changed(
    rt: &Runtime,
    node: NodeId,
    flags: kivact_engine::contract::NodeFlags,
) {
    let observation = ViewState::tree(rt)
        .get(node)
        .and_then(crate::view::tree::CoreNode::observation);
    if let Some(observation) = observation {
        rt.enter(|| observation.flags.set(flags));
    }
}

pub(super) fn layout_changed(
    rt: &Runtime,
    node: NodeId,
    rect: Rect,
    size: Size,
    text_truncated: bool,
    measured_layout: Option<Rect>,
) {
    let observation = {
        let tree = ViewState::tree(rt);
        match tree.get(node).map(crate::view::tree::CoreNode::observation) {
            None => {
                drop(tree);
                return stale(rt, "layout", node);
            }
            Some(Some(observation)) => Some(observation),
            // No observation yet: `observe_node` reads the engine's last report.
            Some(None) => None,
        }
    };
    let layout_handler = handler(rt, node, |h| h.layout.clone());
    if observation.is_some() || layout_handler.is_some() {
        run_handler(rt, node_owner(rt, node), || {
            if let Some(observation) = observation {
                observation.text_truncated.set(text_truncated);
                observation.measured.set(
                    measured_layout
                        .map(|layout_rect| crate::view::tree::Measured { rect, layout_rect }),
                );
            }
            if let Some(layout) = layout_handler {
                layout.call((rect, size))?;
            }
            Ok(())
        });
    }
}

pub(super) fn close_requested(rt: &Runtime, window: WindowId) {
    if window == WindowId::MAIN {
        if keeps_main_open(rt) {
            return;
        }
        rt.request_exit();
        ViewState::of(rt).close_foreign_for_exit();
        return;
    }
    let Some(record) = ViewState::of(rt).window_record(window) else {
        return;
    };
    let close = match &record.decl.dismiss {
        None => true,
        Some(handler) => {
            run_handler(rt, Some(record.declared_in), || {
                handler.call(DismissReason::WindowClose)
            }) == Some(Handled::No)
        }
    };
    if close {
        record.decl.open.set(false);
    }
}

/// Run every live `on_close_request` handler in registration order, each under
/// its owner. `true` when one answers `Yes` or `PreventDefault`, or fails (its
/// error is reported): the App keeps running. Every handler runs either way.
fn keeps_main_open(rt: &Runtime) -> bool {
    let mut keeps_open = false;
    for registration in close_request::registrations(rt) {
        if !registration.owner.is_alive() {
            continue;
        }
        let answer = run_handler(rt, Some(registration.owner), || {
            registration.handler.call(())
        });
        keeps_open |= answer != Some(Handled::No);
    }
    keeps_open
}

/// The platform refused to open `window`: report it where the window was
/// declared and write `open` false (the window's effect then closes it).
pub(super) fn window_failed(rt: &Runtime, window: WindowId, reason: &str) {
    let Some(record) = ViewState::of(rt).window_record(window) else {
        return;
    };
    rt.report(
        Some(record.declared_in),
        UiError::new(
            ErrorKind::Platform,
            format!("the window could not open: {reason}"),
            None,
        ),
    );
    record.decl.open.set(false);
}
