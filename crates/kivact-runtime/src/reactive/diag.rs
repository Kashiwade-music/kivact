//! Role: Per-app trace ring and diagnostics collection.
//!
//! The per-App trace ring: errors routed to the default sink, non-fatal
//! diagnostics, and structured turn records (causes, writes, re-evaluations,
//! ops, duration), mirrored to `tracing`. Recording never changes behaviour.
//! Values are never recorded: only type names and source locations.
//!
//! Invariants:
//! - Recording is always on (no feature or level gates it); the `tracing`
//!   target is `kivact`.
//! - A turn keeps at most 64 entries of each kind (causes, writes, re-runs,
//!   notes) and the ring keeps the last 256 turns.
//! - Notes recorded during the App's setup are sealed, when the setup
//!   ends, into a record of their own (`Turn::Setup`, notes only) instead of
//!   making a host turn.
//! - What is recorded between two turns (a write from a test or a host
//!   callback) belongs to the next turn and is stamped with it.

use std::panic::Location;

use crate::reactive::data::{Diag, Runtime};
use crate::reactive::data::{TraceRun, TraceWrite, Turn, TurnRecord};
use crate::reactive::error::UiError;

const RING: usize = 256;
const PER_TURN: usize = 64;

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
            "{}: {} op(s), {} re-run(s), causes {:?}",
            self.turn, self.ops, self.runs, self.causes
        )?;
        for w in &self.writes {
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

impl Diag {
    /// An error that reached the default sink.
    pub(crate) fn record_error(&mut self, err: UiError) {
        self.total_errors += 1;
        if self.errors.len() == RING {
            self.errors.pop_front();
        }
        self.errors.push_back(err);
    }

    /// A non-fatal diagnostic (write after dispose, ...).
    pub(crate) fn record_diagnostic(&mut self, err: UiError) {
        tracing::debug!(target: "kivact", kind = ?err.kind(), at = ?err.location(), "{}", err);
        // Also a note of the turn record, readable through `Screen::trace`.
        if self.current.notes.len() < PER_TURN {
            self.current
                .notes
                .push(format!("{:?}: {err:?}", err.kind()));
        }
        if self.diagnostics.len() == RING {
            self.diagnostics.pop_front();
        }
        self.diagnostics.push_back(err);
    }

    /// A free-form note of the current turn.
    pub(crate) fn note(&mut self, msg: String) {
        tracing::trace!(target: "kivact", "{}", msg);
        if self.current.notes.len() < PER_TURN {
            self.current.notes.push(msg);
        }
    }

    /// The turn being recorded (`Setup` until the setup is sealed).
    pub(crate) fn turn(&self) -> Turn {
        self.current.turn
    }

    /// What started work in the current turn.
    pub(crate) fn cause(&mut self, c: String) {
        if self.current.causes.len() < PER_TURN {
            self.current.causes.push(c);
        }
    }

    /// A value-changing write.
    pub(crate) fn record_write(
        &mut self,
        ty: &'static str,
        created_at: &'static Location<'static>,
        at: &'static Location<'static>,
    ) {
        if self.current.writes.len() < PER_TURN {
            self.current.writes.push(TraceWrite { ty, created_at, at });
        }
    }

    /// A memo or effect re-evaluation.
    pub(crate) fn record_run(&mut self, kind: &'static str, at: &'static Location<'static>) {
        self.current.runs += 1;
        if self.current.reruns.len() < PER_TURN {
            self.current.reruns.push(TraceRun { kind, at });
        }
    }

    fn push_record(&mut self, record: TurnRecord) {
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
    }

    /// Start recording the next turn, delivering `events` host events; what
    /// was recorded since the last turn stays in it.
    pub(crate) fn begin_turn(&mut self, events: usize) {
        self.finish_setup_notes();
        self.current.events = events;
    }

    /// Finish the current turn and push it into the ring.
    pub(crate) fn end_turn(&mut self, ops: usize, duration_ns: u64) {
        let Turn::Number(n) = self.current.turn else {
            unreachable!("invariant: begin_turn sealed the setup record")
        };
        self.current.ops = ops;
        self.current.duration_ns = duration_ns;
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

    /// Record what started work in the current turn (`"timer"`, an event, ...).
    pub(crate) fn record_cause(&self, c: String) {
        self.diag.borrow_mut().cause(c);
    }

    /// Publish setup notes without creating a host turn.
    pub fn finish_setup_notes(&self) {
        self.diag.borrow_mut().finish_setup_notes();
    }

    /// Start recording a turn that delivers `events` host events.
    pub(crate) fn begin_turn(&self, events: usize) {
        self.diag.borrow_mut().begin_turn(events);
    }

    /// Finish the turn record with the number of writes committed and its duration.
    pub(crate) fn end_turn(&self, ops: usize, duration_ns: u64) {
        self.diag.borrow_mut().end_turn(ops, duration_ns);
    }

    /// Errors that reached the default sink, oldest first (bounded ring).
    pub fn errors(&self) -> Vec<UiError> {
        self.diag.borrow().errors.iter().cloned().collect()
    }

    /// Recent turn records, oldest first (bounded ring).
    pub fn turns(&self) -> Vec<TurnRecord> {
        self.diag.borrow().turns.iter().cloned().collect()
    }
}
