//! Role: Per-app trace ring and diagnostics collection.
//!
//! The per-App trace ring: errors routed to the default sink, non-fatal
//! diagnostics, and structured turn records (causes, writes, re-evaluations,
//! edges, stages, ops, duration), mirrored to `tracing`. Recording never
//! changes behaviour. Values are never recorded: only type names and source
//! locations.
//!
//! Invariants:
//! - Errors and diagnostics are always kept; the turn trace is kept only
//!   while a recorder is on ([`Runtime::set_recorder`]). The `tracing`
//!   target is `kivact`.
//! - A turn keeps at most 2000 entries of each list (the rest is counted in
//!   `dropped`) and the ring keeps the last 256 turns.
//! - Notes recorded during the App's setup are sealed, when the setup
//!   ends, into a record of their own (`Turn::Setup`, notes only) instead of
//!   making a host turn.
//! - What is recorded between two turns (a write from a test or a host
//!   callback) belongs to the next turn and is stamped with it.

use std::panic::Location;

use crate::reactive::data::{Diag, Runtime, Turn};
use crate::reactive::detail::{Cause, StageSpan, TraceEdge, TraceRun, TraceWrite, TurnRecord};
use crate::reactive::error::UiError;
use crate::reactive::host_types::Stage;
use crate::reactive::identity::NodeId;

const RING: usize = 256;
/// The most entries one turn keeps in each of its lists.
const CAP: usize = 2000;

/// What a [`Reported`] entry is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportKind {
    /// An error that reached the default sink (no `on_error` took it).
    Error,
    /// A non-fatal diagnostic (a write after dispose, a stale event, ...).
    Diagnostic,
}

/// Which recorders want the turn trace ([`Runtime::set_recorder`]). The
/// trace (turn records, causes, writes, re-runs, edges and stage times) is
/// kept only while one of them is on; errors and diagnostics are always kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecordBits(u8);

impl RecordBits {
    /// The DevTools agent records updates.
    pub const DEVTOOLS: RecordBits = RecordBits(1);
    /// A `kivact::test` screen (`Screen::trace`).
    pub const TEST: RecordBits = RecordBits(2);
    /// The debug comparison trace (`KIVACT_TRACE`).
    pub const TRACE_SINK: RecordBits = RecordBits(4);

    /// Whether any recorder is on.
    pub const fn any(self) -> bool {
        self.0 != 0
    }

    /// These bits with `bits` turned on or off.
    pub const fn with(self, bits: RecordBits, on: bool) -> RecordBits {
        if on {
            RecordBits(self.0 | bits.0)
        } else {
            RecordBits(self.0 & !bits.0)
        }
    }
}

/// One error or diagnostic in report order, as the DevTools reads them.
pub struct Reported {
    /// The report sequence number (the rings share one counter).
    pub seq: u64,
    /// Whether it is an error or a non-fatal diagnostic.
    pub kind: ReportKind,
    /// The error.
    pub error: UiError,
}

impl std::fmt::Display for Turn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Turn::Setup => f.write_str("setup"),
            Turn::Number(n) => write!(f, "turn {n}"),
        }
    }
}

impl std::fmt::Display for TurnRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} op(s), {} re-run(s), causes [",
            self.turn, self.ops, self.runs
        )?;
        for (i, cause) in self.causes.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{cause}")?;
        }
        f.write_str("]")?;
        for w in self.writes.iter().filter(|w| w.changed) {
            write!(
                f,
                "\n  write {} (created at {}) at {}",
                w.ty, w.created_at, w.at
            )?;
        }
        for n in &self.notes {
            write!(f, "\n  note: {n}")?;
        }
        Ok(())
    }
}

/// Push `item` into one list of the current turn unless the list is at the
/// cap; a dropped entry is counted. Returns the index of the entry kept.
fn keep<T>(list: &mut Vec<T>, dropped: &mut u32, item: T) -> Option<u32> {
    if list.len() < CAP {
        list.push(item);
        u32::try_from(list.len() - 1).ok()
    } else {
        *dropped += 1;
        None
    }
}

impl Diag {
    /// An error that reached the default sink.
    pub(crate) fn record_error(&mut self, err: UiError) {
        self.total_errors += 1;
        if self.errors.len() == RING {
            self.errors.pop_front();
        }
        self.next_seq += 1;
        let seq = self.next_seq;
        self.errors.push_back((seq, err));
    }

    /// A non-fatal diagnostic (write after dispose, ...).
    pub(crate) fn record_diagnostic(&mut self, err: UiError) {
        tracing::debug!(target: "kivact", kind = ?err.kind(), at = ?err.location(), "{}", err);
        // Also a note of the turn record, readable through `Screen::trace`.
        if self.record.any() {
            let note = format!("{:?}: {err:?}", err.kind());
            keep(&mut self.current.notes, &mut self.current.dropped, note);
        }
        if self.diagnostics.len() == RING {
            self.diagnostics.pop_front();
        }
        self.next_seq += 1;
        let seq = self.next_seq;
        self.diagnostics.push_back((seq, err));
    }

    /// A free-form note of the current turn.
    pub(crate) fn note(&mut self, msg: String) {
        tracing::trace!(target: "kivact", "{}", msg);
        if self.record.any() {
            keep(&mut self.current.notes, &mut self.current.dropped, msg);
        }
    }

    /// The turn being recorded (`Setup` until the setup is sealed).
    pub(crate) fn turn(&self) -> Turn {
        self.current.turn
    }

    /// Record what started work in the current turn; the writes and runs
    /// that follow name it.
    pub(crate) fn cause(&mut self, cause: Cause) {
        if !self.record.any() {
            return;
        }
        self.cause = keep(&mut self.current.causes, &mut self.current.dropped, cause);
    }

    /// A write of a signal (`changed` says whether it changed the value).
    pub(crate) fn record_write(
        &mut self,
        node: NodeId,
        ty: &'static str,
        created_at: &'static Location<'static>,
        at: &'static Location<'static>,
        changed: bool,
    ) {
        if !self.record.any() {
            return;
        }
        let cause = self.cause;
        keep(
            &mut self.current.writes,
            &mut self.current.dropped,
            TraceWrite {
                node,
                cause,
                ty,
                created_at,
                at,
                changed,
            },
        );
    }

    /// A memo or effect re-evaluation.
    pub(crate) fn record_run(
        &mut self,
        node: NodeId,
        kind: &'static str,
        at: &'static Location<'static>,
        changed: bool,
    ) {
        if !self.record.any() {
            return;
        }
        self.current.runs += 1;
        let cause = self.cause;
        keep(
            &mut self.current.reruns,
            &mut self.current.dropped,
            TraceRun {
                node,
                cause,
                kind,
                at,
                changed,
            },
        );
    }

    /// One dependency edge marked by a notification.
    pub(crate) fn record_edge(&mut self, source: NodeId, target: NodeId) {
        if !self.record.any() {
            return;
        }
        keep(
            &mut self.current.edges,
            &mut self.current.dropped,
            TraceEdge { source, target },
        );
    }

    /// Close the open stage and open `stage` at turn-relative `elapsed_ns`.
    pub(crate) fn stage(&mut self, stage: Stage, base_ns: u64, elapsed_ns: u64) {
        if !self.record.any() {
            return;
        }
        if let Some((prev, prev_elapsed)) = self.open_stage.take() {
            keep(
                &mut self.current.stages,
                &mut self.current.dropped,
                StageSpan {
                    stage: prev,
                    start_ns: base_ns + prev_elapsed,
                    duration_ns: elapsed_ns.saturating_sub(prev_elapsed),
                },
            );
        }
        self.open_stage = Some((stage, elapsed_ns));
    }

    fn push_record(&mut self, record: TurnRecord) {
        if !self.record.any() {
            return;
        }
        if self.turns.len() == RING {
            self.turns.pop_front();
        }
        self.turns.push_back(record);
    }

    /// Seal the setup's notes into a record of their own (none when there are
    /// no notes); the setup's writes and re-runs are not kept. Afterwards the
    /// record being filled is turn 1. A no-op once sealed.
    fn finish_setup_notes(&mut self) {
        if self.current.turn != Turn::Setup {
            return;
        }
        let notes = std::mem::take(&mut self.current.notes);
        if !notes.is_empty() {
            self.push_record(TurnRecord {
                notes,
                ..TurnRecord::default()
            });
        }
        self.current = TurnRecord {
            turn: Turn::Number(1),
            ..TurnRecord::default()
        };
        self.cause = None;
    }

    /// Start recording the next turn, delivering `events` host events; what
    /// was recorded since the last turn stays in it.
    pub(crate) fn begin_turn(&mut self, events: usize) {
        self.finish_setup_notes();
        self.current.events = events;
    }

    /// Finish the current turn and push it into the ring.
    pub(crate) fn end_turn(&mut self, ops: usize, duration_ns: u64, base_ns: u64) {
        let Turn::Number(n) = self.current.turn else {
            unreachable!("invariant: begin_turn sealed the setup record")
        };
        self.current.ops = ops;
        self.current.duration_ns = duration_ns;
        if let Some((stage, start)) = self.open_stage.take() {
            keep(
                &mut self.current.stages,
                &mut self.current.dropped,
                StageSpan {
                    stage,
                    start_ns: base_ns + start,
                    duration_ns: duration_ns.saturating_sub(start),
                },
            );
        }
        self.cause = None;
        let c = &self.current;
        tracing::trace!(
            target: "kivact",
            turn = n,
            events = c.events,
            writes = c.writes.len(),
            runs = c.runs,
            ops,
            duration_ns,
            "turn"
        );
        let next = TurnRecord {
            turn: Turn::Number(n + 1),
            ..TurnRecord::default()
        };
        let done = std::mem::replace(&mut self.current, next);
        self.push_record(done);
    }
}

impl Runtime {
    /// Record a note in the current turn (also `tracing` trace level).
    pub fn note(&self, msg: String) {
        self.diag.borrow_mut().note(msg);
    }

    /// A `tracing` warning that is also recorded as a note of the turn.
    pub fn warn(&self, msg: String) {
        tracing::warn!(target: "kivact", "{}", msg);
        self.note(msg);
    }

    /// Record what started work in the current turn (kept only while a
    /// recorder is on).
    pub(crate) fn record_cause(&self, cause: Cause) {
        if !self.is_recording() {
            return;
        }
        self.diag.borrow_mut().cause(cause);
    }

    /// Record the time since `started` as the active turn stage, in the
    /// detailed trace (only while a recorder is on).
    pub(crate) fn record_stage(&self, stage: Stage, started: std::time::Instant) {
        let mut diag = self.diag.borrow_mut();
        if !diag.record.any() {
            return;
        }
        let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
        diag.stage(stage, self.now_ns(), elapsed);
    }

    /// Publish setup notes without creating a host turn.
    pub fn finish_setup_notes(&self) {
        self.diag.borrow_mut().finish_setup_notes();
    }

    /// Start recording a turn that delivers `events` host events.
    pub(crate) fn begin_turn_record(&self, events: usize) {
        self.diag.borrow_mut().begin_turn(events);
    }

    /// Finish the turn record with the number of writes committed and its duration.
    pub(crate) fn end_turn(&self, ops: usize, duration_ns: u64) {
        self.diag
            .borrow_mut()
            .end_turn(ops, duration_ns, self.now_ns());
    }

    /// Turn recorder `bits` on or off. While no recorder is on, no turn
    /// trace is kept: turning the last one off drops what was recorded.
    pub fn set_recorder(&self, bits: RecordBits, on: bool) {
        let mut diag = self.diag.borrow_mut();
        let was = diag.record.any();
        diag.record = diag.record.with(bits, on);
        if was && !diag.record.any() {
            diag.turns.clear();
            diag.current = TurnRecord {
                turn: diag.current.turn,
                ..TurnRecord::default()
            };
            diag.cause = None;
            diag.open_stage = None;
        }
    }

    /// Whether a recorder is on (the turn trace is being kept).
    pub fn is_recording(&self) -> bool {
        self.diag.borrow().record.any()
    }

    /// Errors that reached the default sink, oldest first (bounded ring).
    pub fn errors(&self) -> Vec<UiError> {
        self.diag
            .borrow()
            .errors
            .iter()
            .map(|(_, error)| error.clone())
            .collect()
    }

    /// Errors and diagnostics in the order they were reported (bounded rings).
    pub fn reported(&self) -> Vec<Reported> {
        let diag = self.diag.borrow();
        let mut reported = Vec::with_capacity(diag.errors.len() + diag.diagnostics.len());
        reported.extend(diag.errors.iter().map(|(seq, error)| Reported {
            seq: *seq,
            kind: ReportKind::Error,
            error: error.clone(),
        }));
        reported.extend(diag.diagnostics.iter().map(|(seq, error)| Reported {
            seq: *seq,
            kind: ReportKind::Diagnostic,
            error: error.clone(),
        }));
        reported.sort_by_key(|entry| entry.seq);
        reported
    }

    /// Errors and diagnostics reported after sequence number `since`, in
    /// report order (bounded rings).
    pub fn reported_since(&self, since: u64) -> Vec<Reported> {
        let diag = self.diag.borrow();
        let mut reported = Vec::new();
        for (seq, error) in diag.errors.iter().rev().take_while(|(seq, _)| *seq > since) {
            reported.push(Reported {
                seq: *seq,
                kind: ReportKind::Error,
                error: error.clone(),
            });
        }
        for (seq, error) in diag
            .diagnostics
            .iter()
            .rev()
            .take_while(|(seq, _)| *seq > since)
        {
            reported.push(Reported {
                seq: *seq,
                kind: ReportKind::Diagnostic,
                error: error.clone(),
            });
        }
        reported.sort_by_key(|entry| entry.seq);
        reported
    }

    /// The current setup or host turn.
    pub fn turn(&self) -> Turn {
        self.diag.borrow().turn()
    }

    /// Recent turn records, oldest first (bounded ring).
    pub fn turns(&self) -> Vec<TurnRecord> {
        self.diag.borrow().turns.iter().cloned().collect()
    }

    /// The newest turn record, if a recorder is on and a turn ended.
    pub fn last_turn(&self) -> Option<TurnRecord> {
        self.diag.borrow().turns.back().cloned()
    }
}
