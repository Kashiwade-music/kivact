//! Role: The turn's deferred work, drained by named phases.
//!
//! Work that may not run where it was asked for waits in its own queue, each
//! typed for its rule: focus requests (an explicit request wins over a
//! restore), activations, removed nodes, `on_mount` callbacks (released by a
//! revision token), reports kept for the next turn. [`drain`] is the only
//! place that calls their drain functions, and a [`Phase`] says when in the
//! turn each runs. The order of the calls inside a phase is the invariant;
//! this table is where it is written down (design §4「turn」):
//!
//! | phase | drains, in order | runs |
//! |---|---|---|
//! | [`Phase::AfterHandler`] | requested activations (`NodeRef::activate`); if any ran, stabilise, and settle tasks and timers when asked | after each event's handler, and after the tasks of the turn settled: an activation runs the target's default action as a click would, so it needs the handler's writes stabilised, and a handler it triggers may ask for more |
//! | [`Phase::BeforeCommit`] | the frame to request ([`MotionState::plan_frame`]), focus requests, removed nodes (and the asset failures of the nodes that left), the revision token the new `on_mount` callbacks wait for | in `flush`, right before the commit: focus writes and the removals belong to the revision about to be committed, and the token is that revision (or the last one when nothing is written) |
//! | [`Phase::AfterCommit`] | `on_mount` callbacks whose revision was acknowledged | at the turn's start (the revisions done before its events), at each commit boundary among the events, and once the turn's own revisions are laid out: the callbacks of a revision see the tree as laid out |
//! | [`Phase::NextTurn`] | reports kept while the engine held the tree (`Runtime::report_later`) | at the turn's start, outside any engine borrow, so a handler can run |
//!
//! Each queue keeps its data where it lives (core `Requests`, the runtime's
//! mounts and later reports, the tree's removes, `TaskState`, `MotionState`);
//! a phase only decides when they are drained.

use kivact_engine::bridge::Writer;
use kivact_motion::MotionState;
use kivact_reactive::{Runtime, TurnRecorder};

use crate::client::settle::settle_async;
use crate::client::state::HostState;
use crate::view::requests;
use crate::view::state::ViewState;

/// A point of the turn, with what its drains need.
pub(super) enum Phase<'a> {
    /// `settle`: the turn to settle tasks and timers for after the
    /// activations ran (`None`: only stabilise).
    AfterHandler {
        settle: Option<&'a TurnRecorder<'a>>,
    },
    /// `writer`: the engine's writer the commit goes through.
    BeforeCommit {
        writer: &'a Writer,
    },
    /// `revision`: the last revision the host acknowledged.
    AfterCommit {
        revision: u64,
    },
    NextTurn,
}

/// What [`drain`] found.
#[derive(Default)]
pub(super) struct Drained {
    /// `AfterHandler`: whether an activation ran.
    pub activated: bool,
    /// `BeforeCommit`: the engine's operation count to commit (0: nothing).
    pub writes: usize,
}

/// Drain the deferred work of `phase`, in the order of the table above.
pub(super) fn drain(rt: &Runtime, phase: Phase<'_>) -> Drained {
    let mut drained = Drained::default();
    match phase {
        Phase::AfterHandler { settle } => {
            drained.activated = crate::client::dispatch::run_requested_activations(rt);
            if drained.activated {
                rt.stabilize();
                if let Some(turn) = settle {
                    settle_async(rt, turn);
                }
            }
        }
        Phase::BeforeCommit { writer } => {
            let host = HostState::of(rt);
            if let Some(at_ns) = MotionState::of(rt).plan_frame(rt) {
                ViewState::tree(rt).engine().request_frame(at_ns);
            }
            requests::flush_focus(rt);
            let removes = ViewState::tree_mut(rt).take_removes();
            drained.writes = writer.pending_writes();
            if removes {
                // Removed nodes report nothing more: forget their failures,
                // so the set stays as small as the live tree.
                let tree = ViewState::tree(rt);
                host.asset_failures
                    .borrow_mut()
                    .retain(|(id, _)| tree.is_live(*id));
            }
            let revision = writer.stats().revision;
            rt.mounts()
                .defer_until(revision + u64::from(drained.writes != 0));
        }
        Phase::AfterCommit { revision } => rt.mounts().run_acked(revision),
        Phase::NextTurn => rt.deliver_later(),
    }
    drained
}
