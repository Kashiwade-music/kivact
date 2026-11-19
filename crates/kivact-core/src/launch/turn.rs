//! Role: The order of one App turn.
//!
//! One turn: due mount callbacks and deferred reports, then each host
//! event (dispatch, then settle), then commit rounds (commit, mounts,
//! deferred reports, user effects, settle) until no work is left. The
//! only settle is `TurnGuard::settle` (effects, deferred handlers,
//! timers and tasks under one budget per turn).

use kivact_engine::api::{Boundary, Writer};
use kivact_engine::contract::{HostEvent, TurnOutput};
use kivact_runtime::{CauseTag, ErrorKind, Runtime, Stage, TurnGuard, UiError};
use kivact_types::Revision;

use super::runtime::{ClientMode, Launch, commit, output};
use crate::client::state::HostState;
use crate::view::state::ViewState;
use crate::view::text_edits::TextEditsAhead;

/// Commit rounds one turn may run before user effects count as a cycle.
const USER_EFFECT_ROUNDS: usize = 100;

impl Launch {
    /// One turn of `events` (see the module documentation). The
    /// boundaries among the events only decide when the `on_mount`
    /// callbacks of their revisions run.
    pub(super) fn turn(
        &mut self,
        events: &[HostEvent],
        boundaries: &[Boundary],
        now_ns: u64,
    ) -> TurnOutput {
        let rt = self.runtime().clone();
        let host = HostState::of(&rt);
        rt.advance_clock(now_ns);
        // A close requested from any thread since the last turn starts the
        // exit before the idle check, so a bare wake always counts.
        let embedded = matches!(self.mode, ClientMode::Embedded(_));
        super::close::take_close_request(&rt, embedded);
        #[cfg(feature = "devtools")]
        let incoming = crate::devtools::take_incoming(&rt);
        let tree = ViewState::tree(&rt).engine().clone();
        // A host call with no event, no due mount callback and no queued
        // work runs no turn.
        let idle = events.is_empty()
            && boundaries
                .last()
                .is_some_and(|b| !rt.mounts_due(b.revision))
            && !rt.has_work();
        #[cfg(feature = "devtools")]
        let idle = idle && incoming.is_empty();
        if idle {
            return output(rt.end_idle());
        }
        // The revisions done before the events: all but those whose
        // boundaries this turn delivers.
        let before = boundaries
            .first()
            .map_or(tree.stats().revision, |b| b.revision.0.saturating_sub(1));
        let mut stats = host.stats.get();
        stats.turns += 1;
        host.stats.set(stats);
        let turn = rt.begin_turn(events.len());
        if events.is_empty()
            && boundaries.last().is_some_and(|b| rt.mounts_due(b.revision))
            && rt.is_recording()
        {
            turn.cause_with(CauseTag::Mount, "mounts".into());
        }
        turn.run_mounts(Revision(before));
        turn.deliver_reports();
        turn.settle();
        let edits = TextEditsAhead::begin(&mut ViewState::tree_mut(&rt), events);
        let mut boundaries = boundaries.iter().peekable();
        let mut acknowledge = |delivered: usize| {
            while let Some(b) = boundaries.next_if(|b| b.events <= delivered) {
                turn.run_mounts(b.revision);
                turn.settle();
            }
        };
        acknowledge(0);
        #[cfg(feature = "devtools")]
        crate::devtools::run_incoming(&rt, &turn, incoming);
        let mut delivered = 0;
        for event in events {
            if matches!(&self.mode, ClientMode::Embedded(closing) if closing.is_set()) {
                break;
            }
            turn.stage(Stage::Dispatch);
            #[cfg(debug_assertions)]
            crate::client::trace::event(&rt, format_args!("host_events receive {event:?}"));
            edits.deliver(event);
            ViewState::tree(&rt).engine().deliver(event);
            if rt.is_recording() {
                turn.cause_with(CauseTag::Input, describe(event));
            }
            crate::client::dispatch::dispatch(&rt, event);
            turn.settle();
            delivered += 1;
            acknowledge(delivered);
        }
        acknowledge(events.len());
        #[cfg(feature = "devtools")]
        crate::devtools::run_events(&rt);
        let done = Revision(tree.stats().revision);
        turn.check_draining();
        turn.settle();
        let writes = commit_rounds(&turn, &rt, &tree, done);
        let report = turn.finish(writes);
        #[cfg(feature = "devtools")]
        crate::devtools::after_turn(&rt);
        let output = output(report);
        #[cfg(debug_assertions)]
        {
            crate::client::trace::record(&rt);
            crate::client::trace::event(&rt, format_args!("host_events output {output:?}"));
        }
        output
    }
}

/// Commit, then run what the commit released (mount callbacks, deferred
/// reports, user effects) until no work is left. Returns the writes.
fn commit_rounds(turn: &TurnGuard<'_>, rt: &Runtime, tree: &Writer, done: Revision) -> usize {
    let mut writes = 0;
    let mut round = 0;
    loop {
        writes += commit(turn, rt, tree);
        let mounts = rt.mounts_due(done);
        // The commit may queue work too (a failed focus request reaching
        // an error boundary): it runs and commits in this turn.
        if !mounts && !rt.has_work() {
            break;
        }
        round += 1;
        if round > USER_EFFECT_ROUNDS {
            let abandoned = turn.abandon_user_effects();
            let path: Vec<_> = abandoned
                .iter()
                .filter_map(|k| rt.node_info(*k).map(|n| n.at))
                .collect();
            rt.report(
                None,
                UiError::new(
                    ErrorKind::Cycle { path: path.clone() },
                    "user effects did not settle within 100 rounds",
                    path.first().copied(),
                ),
            );
            break;
        }
        if mounts {
            turn.run_mounts(done);
        }
        turn.deliver_reports();
        turn.run_user_effects();
        turn.settle();
    }
    writes
}

/// The name of a pointer phase without its press data (trace labels carry
/// no values).
fn phase_name(phase: kivact_types::PointerPhase) -> &'static str {
    use kivact_types::PointerPhase as P;
    match phase {
        P::Down(_) => "Down",
        P::Move => "Move",
        P::Up(_) => "Up",
        P::Cancel => "Cancel",
        P::Enter => "Enter",
        P::Leave => "Leave",
    }
}

/// Short cause label of an event for the structured trace (no values).
fn describe(e: &kivact_engine::contract::HostEvent) -> String {
    use kivact_engine::contract::HostEvent as E;
    match e {
        E::Pointer(route) => format!(
            "Pointer {} {:?}",
            phase_name(route.input.phase),
            route.target.map(|target| target.node)
        ),
        E::Wheel(route) => format!("Wheel {:?}", route.target.map(|target| target.node)),
        E::Key(route) => format!(
            "Key {:?} {:?} region {:?}",
            route.input.key,
            route.target.map(|target| target.node),
            route.target.and_then(|target| target.region)
        ),
        E::Activate { node, region, .. } => {
            format!("Activate n{} region {:?}", node.0, region.map(|r| r.0))
        }
        E::FocusChanged {
            node,
            region,
            visible,
            ..
        } => format!(
            "FocusChanged {:?} region {:?} visible {visible}",
            node.map(|n| n.0),
            region.map(|r| r.0)
        ),
        E::HoverChanged { target, .. } => {
            format!("HoverChanged {:?}", target.map(|t| (t.node.0, t.region)))
        }
        E::MousePosition { position, .. } => format!("MousePosition {position:?}"),
        E::TextChanged { node, .. } => format!("TextChanged n{}", node.0),
        E::SelectionChanged { node, .. } => format!("SelectionChanged n{}", node.0),
        E::Ime { node, .. } => format!("Ime n{}", node.0),
        E::ScrollChanged { node, .. } => format!("ScrollChanged n{}", node.0),
        E::LayoutChanged { node, .. } => format!("LayoutChanged n{}", node.0),
        E::A11y { node, request, .. } => format!("A11y {:?} n{}", request.action(), node.0),
        E::SnapshotResult { request, .. } => format!("SnapshotResult {}", request.0),
        other => {
            let s = format!("{other:?}");
            s.split([' ', '{', '('])
                .next()
                .unwrap_or_default()
                .to_string()
        }
    }
}
