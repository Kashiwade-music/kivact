//! Role: Incoming DevTools calls, queued engine events, reset and after-turn work.

use kivact_devtools_agent::Incoming;
use kivact_devtools_protocol as p;
use kivact_runtime::{CauseTag, ErrorKind, Runtime, Stage, TurnGuard, UiError};
use kivact_types::{NodeId, WindowId};

use super::console;
use super::emulation;
use super::input;
use super::motion;
use super::overlay;
use super::query;
use super::styles;
use super::target;
use super::tree;
use super::updates;
use crate::devtools::inspect_mode::Inspector as Inspect;
use crate::devtools::state::{AgentSlot, DevtoolsEvent, DevtoolsState, Inspector, Pending};
use crate::view::state::ViewState;

/// Everything the agent received since the last turn. The first turn after the
/// agent starts promotes it.
pub(crate) fn take_incoming(rt: &Runtime) -> Vec<Incoming> {
    let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
        return Vec::new();
    };
    i.promote(rt);
    match &mut *i.agent.borrow_mut() {
        AgentSlot::Ready(agent) => agent.take_incoming(),
        AgentSlot::Starting(_) | AgentSlot::Failed => Vec::new(),
    }
}

/// Run the calls and connection endings that arrived before this turn's events.
pub(crate) fn run_incoming(rt: &Runtime, turn: &TurnGuard<'_>, incoming: Vec<Incoming>) {
    for item in incoming {
        match item {
            Incoming::Call(call, reply) => {
                turn.stage(Stage::Dispatch);
                turn.cause_with(CauseTag::Devtools, format!("devtools {}", call.method()));
                rt.root().run_handler(|| target::dispatch(rt, call, reply));
            }
            Incoming::Closed { remaining: 0 } => rt.root().run_handler(|| reset(rt)),
            Incoming::Closed { .. } => {}
        }
    }
    turn.stabilize();
}

/// Restore every state the DevTools set, keeping the review.
fn reset(rt: &Runtime) {
    overlay::clear(rt);
    styles::clear_all(rt);
    emulation::clear(rt);
    motion::reset_time_scale(rt);
    updates::stop(rt);
}

/// The work of a turn after `turn.finish` fixed its records. The engine's change
/// records say which nodes and windows the turn touched.
pub(crate) fn after_turn(rt: &Runtime) {
    let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
        return;
    };
    if i.agent().is_none() {
        return;
    }
    let changes = ViewState::tree(rt).reader().take_tree_changes();
    let closed = tree::notify_windows_changed(rt);
    sweep_closed_windows(rt, i, &closed);
    tree::notify_tree_changes(rt, &changes);
    prune_removed(i, &changes.removed);
    let records = ViewState::tree(rt).reader().take_engine_records();
    query::after_turn(
        rt,
        !changes.upserted.is_empty() || !changes.removed.is_empty(),
    );
    updates::after_turn(rt, &records);
    overlay::after_turn(rt, &records);
    console::after_turn(rt);
}

/// Forget the counts and forced states of the nodes the turn destroyed.
fn prune_removed(i: &Inspector, removed: &[(WindowId, NodeId)]) {
    if removed.is_empty() {
        return;
    }
    {
        let mut counts = i.session.update_counts.borrow_mut();
        for (_, id) in removed {
            counts.remove(id);
        }
    }
    i.hooks.forced.update(|all| {
        for (_, id) in removed {
            all.remove(id);
        }
    });
}

/// Answer the calls that waited for a window that has closed.
fn sweep_closed_windows(rt: &Runtime, i: &Inspector, closed: &[WindowId]) {
    if closed.is_empty() {
        return;
    }
    let pending = std::mem::take(&mut *i.session.pending.borrow_mut());
    for item in pending {
        let gone = item.window().is_some_and(|window| closed.contains(&window));
        if gone {
            match item {
                Pending::Find { reply, .. }
                | Pending::Inject { reply, .. }
                | Pending::Turns { reply, .. } => reply.send(Err(p::RpcError::new(
                    p::code::WINDOW_NOT_FOUND,
                    "the window closed",
                ))),
            }
        } else {
            i.session.pending.borrow_mut().push(item);
        }
    }
    if let Inspect::Picking { window, .. } = i.session.inspector.get()
        && closed.contains(&window)
    {
        i.session.end_pick();
        ViewState::tree(rt).engine().set_pick(None);
    }
    i.session
        .frames
        .borrow_mut()
        .retain(|window, _| !closed.contains(window));
    i.hooks
        .focus_history
        .borrow_mut()
        .retain(|window, _| !closed.contains(window));
    i.session
        .sent_overlays
        .borrow_mut()
        .retain(|window, _| !closed.contains(window));
}

/// Handle the engine events the client dispatch queued.
pub(crate) fn run_events(rt: &Runtime) {
    let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
        return;
    };
    let events = std::mem::take(&mut *i.session.events.borrow_mut());
    for event in events {
        rt.root().run_handler(|| match event {
            DevtoolsEvent::Key { window, pick } => {
                if pick {
                    overlay::start_pick_in(rt, window);
                }
                let opened = i.agent().map(|mut agent| agent.open_devtools());
                if let Some(Err(message)) = opened
                    && i.session.open_failed.note()
                {
                    rt.report(
                        Some(rt.root()),
                        UiError::new(ErrorKind::Platform, message, None),
                    );
                }
            }
            DevtoolsEvent::Pick {
                window,
                node,
                phase,
            } => overlay::pick_event(rt, window, node, phase),
            DevtoolsEvent::InjectDelivered { request } => input::delivered(rt, request),
        });
    }
}
