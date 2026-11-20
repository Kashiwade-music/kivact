//! Role: The order of one App turn.
//!
//! One turn: due mount callbacks and deferred reports, then each host
//! event (dispatch, then settle), then commit rounds (commit, mounts,
//! deferred reports, user effects, settle) until no work is left. The
//! only settle is `TurnGuard::settle` (effects, deferred handlers,
//! timers and tasks under one budget per turn).

use kivact_engine::api::{Boundary, Writer};
use kivact_engine::contract::{HostEvent, TurnOutput};
use kivact_runtime::driver::{Runtime, TurnGuard};
use kivact_runtime::{Cause, CauseTag, ErrorKind, Stage, UiError};
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
        // A host call with no event, no due mount callback, no queued work
        // and no mail from other threads runs no turn.
        let idle = events.is_empty()
            && boundaries
                .last()
                .is_some_and(|b| !rt.mounts_due(b.revision))
            && !rt.has_work()
            && !rt.has_mail();
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
        if mounts_only(&rt, events, boundaries) && rt.is_recording() {
            turn.cause(Cause {
                tag: CauseTag::Mount,
                event: "mounts",
                node: None,
            });
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
            turn.cause(cause_of(event));
            crate::client::dispatch::dispatch(&rt, event);
            turn.settle();
            delivered += 1;
            acknowledge(delivered);
        }
        acknowledge(events.len());
        #[cfg(feature = "devtools")]
        crate::devtools::run_events(&rt);
        let done = Revision(tree.stats().revision);
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

/// Whether this turn has no event and only runs the due mount callbacks.
fn mounts_only(rt: &Runtime, events: &[HostEvent], boundaries: &[Boundary]) -> bool {
    events.is_empty() && boundaries.last().is_some_and(|b| rt.mounts_due(b.revision))
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

/// What a host event says for the turn trace (its kind and node, no values).
fn cause_of(e: &HostEvent) -> Cause {
    use kivact_engine::contract::HostEvent as E;
    let (event, node) = match e {
        E::Pointer(route) => ("Pointer", route.target.map(|t| t.node)),
        E::Wheel(route) => ("Wheel", route.target.map(|t| t.node)),
        E::File(route) => ("File", route.target.map(|t| t.node)),
        E::Key(route) => ("Key", route.target.map(|t| t.node)),
        E::Activate { node, .. } => ("Activate", Some(*node)),
        E::FocusChanged { node, .. } => ("FocusChanged", *node),
        E::HoverChanged { target, .. } => ("HoverChanged", target.map(|t| t.node)),
        E::TextChanged { node, .. } => ("TextChanged", Some(*node)),
        E::SelectionChanged { node, .. } => ("SelectionChanged", Some(*node)),
        E::Ime { node, .. } => ("Ime", Some(*node)),
        E::ScrollChanged { node, .. } => ("ScrollChanged", Some(*node)),
        E::LayoutChanged { node, .. } => ("LayoutChanged", Some(*node)),
        E::A11y { node, .. } => ("A11y", Some(*node)),
        E::StatesChanged { node, .. } => ("StatesChanged", Some(*node)),
        E::NodeFlagsChanged { node, .. } => ("NodeFlagsChanged", Some(*node)),

        E::PresenceChanged { .. } => ("PresenceChanged", None),
        E::ExitComplete { .. } => ("ExitComplete", None),
        E::ModifiersChanged { .. } => ("ModifiersChanged", None),
        E::MousePosition { .. } => ("MousePosition", None),
        E::FrameTick { .. } => ("FrameTick", None),
        E::EffectResult { .. } => ("EffectResult", None),
        E::SnapshotResult { .. } => ("SnapshotResult", None),
        E::FocusResult { .. } => ("FocusResult", None),
        E::Lifecycle(_) => ("Lifecycle", None),
        E::EnvChanged { .. } => ("EnvChanged", None),
        E::HostFailure { .. } => ("HostFailure", None),
        E::TurnsCut { .. } => ("TurnsCut", None),
        E::ImageStatusChanged { .. } => ("ImageStatusChanged", None),
        E::AssetFailed { .. } => ("AssetFailed", None),
        E::DrawDropped { .. } => ("DrawDropped", None),
        E::DrawableNotHosted { .. } => ("DrawableNotHosted", None),
        E::DrawableDrawnTwice { .. } => ("DrawableDrawnTwice", None),
        E::LayoutWarning { .. } => ("LayoutWarning", None),
        E::DuplicateLayoutId { .. } => ("DuplicateLayoutId", None),
        E::CloseRequested { .. } => ("CloseRequested", None),
        E::WindowOpened { .. } => ("WindowOpened", None),
        E::WindowFailed { .. } => ("WindowFailed", None),
        E::Foreign { .. } => ("Foreign", None),
        E::InjectDelivered { .. } => ("InjectDelivered", None),
        E::DevtoolsPick { .. } => ("DevtoolsPick", None),
        E::DevtoolsKey { .. } => ("DevtoolsKey", None),
        E::AppMotion { .. } => ("AppMotion", None),
        E::ExternalImported { .. } => ("ExternalImported", None),
        E::GpuAdapterChanged { .. } => ("GpuAdapterChanged", None),
        E::DrawableFrame(_) => ("DrawableFrame", None),
        E::DrawableResized { .. } => ("DrawableResized", None),
        E::DrawableTextures { .. } => ("DrawableTextures", None),
        E::DrawableExportLost(_) => ("DrawableExportLost", None),
        E::GpuExportAvailable => ("GpuExportAvailable", None),
    };
    Cause {
        tag: CauseTag::Input,
        event,
        node,
    }
}
