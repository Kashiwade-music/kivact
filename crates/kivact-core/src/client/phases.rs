//! Role: The turn's deferred work, drained by named phases.
//!
//! Work that may not run where it was asked for waits in its own queue, each
//! typed for its rule: focus requests (an explicit request wins over a
//! restore), activations, removed nodes, `on_mount` callbacks (released by a
//! revision token), reports kept for the next turn. [`drain`] and
//! [`commit`] are the only places that call their drain functions, and a [`Phase`] says when in the
//! turn each runs. The order of the calls inside a phase is the invariant;
//! this table is where it is written down (design §4「turn」):
//!
//! | phase | drains, in order | runs |
//! |---|---|---|
//! | [`Phase::AfterHandler`] | requested activations (`NodeRef::activate`); if any ran, stabilise, and settle tasks and timers when asked | after each event's handler, and after the tasks of the turn settled: an activation runs the target's default action as a click would, so it needs the handler's writes stabilised, and a handler it triggers may ask for more |
//! | [`commit`] | the frame to request ([`MotionState::plan_frame`]), focus requests, removed nodes (and the asset failures of the nodes that left), the commit, and the revision token the new `on_mount` callbacks wait for (the committed revision, or the last one when nothing is written) | in every commit of the turn: focus writes and the removals belong to the revision being committed |
//! | [`Phase::AfterCommit`] | `on_mount` callbacks whose revision was acknowledged | at the turn's start (the revisions done before its events), at each commit boundary among the events, and once the turn's own revisions are laid out: the callbacks of a revision see the tree as laid out |
//! | [`Phase::NextTurn`] | reports kept while the engine held the tree (`Runtime::report_later`) | at the turn's start, outside any engine borrow, so a handler can run |
//!
//! Each queue keeps its data where it lives (core `Requests`, the runtime's
//! mounts and later reports, the tree's removes, `TaskState`, `MotionState`);
//! a phase only decides when they are drained.

use kivact_engine::api::Writer;
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
    /// `revision`: the last revision the host acknowledged.
    AfterCommit {
        revision: u64,
    },
    NextTurn,
}

/// Drain the deferred work of `phase`, in the order of the table above.
pub(super) fn drain(rt: &Runtime, phase: Phase<'_>) {
    #[cfg(debug_assertions)]
    crate::client::trace::stage(
        rt,
        match &phase {
            Phase::AfterHandler { .. } => "AfterHandler",
            Phase::AfterCommit { .. } => "AfterCommit",
            Phase::NextTurn => "NextTurn",
        },
    );
    match phase {
        Phase::AfterHandler { settle } => {
            if crate::client::dispatch::run_requested_activations(rt) {
                rt.stabilize();
                if let Some(turn) = settle {
                    settle_async(rt, turn);
                }
            }
        }
        Phase::AfterCommit { revision } => rt.mounts().run_acked(revision),
        Phase::NextTurn => rt.deliver_later(),
    }
}

/// Commit the engine's pending changes as the next revision, after the
/// work that belongs to it (the table's `commit` row). Returns the number of
/// operations committed (0: nothing was written and no revision was made).
pub(super) fn commit(rt: &Runtime, writer: &Writer) -> usize {
    if let Some(at_ns) = MotionState::of(rt).plan_frame(rt) {
        writer.request_frame(at_ns);
    }
    requests::flush_focus(rt);
    if ViewState::tree_mut(rt).take_removes() {
        // Removed nodes report nothing more: forget their failures,
        // so the set stays as small as the live tree.
        let tree = ViewState::tree(rt);
        HostState::of(rt)
            .asset_failures
            .borrow_mut()
            .retain(|(id, _)| tree.contains(*id));
    }
    let writes = writer.pending_writes();
    // The new `on_mount` callbacks wait for the revision these writes
    // become, or for the last one when nothing was written.
    let revision = if writes == 0 {
        writer.stats().revision
    } else {
        writer.commit().0
    };
    #[cfg(debug_assertions)]
    {
        crate::client::trace::stage(rt, "Commit");
        crate::client::trace::event(
            rt,
            format_args!("host_events commit revision={revision} writes={writes}"),
        );
    }
    rt.mounts().defer_until(revision);
    writes
}
