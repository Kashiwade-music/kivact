//! Role: The turn trace: what each turn did, kept while a recorder is on.

use std::panic::Location;

use crate::reactive::host_types::Stage;
use crate::reactive::identity::NodeId;

/// Which turn something happened in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Turn {
    /// While the App was set up (its first mount), before the first turn.
    #[default]
    Setup,
    /// A turn, numbered from 1.
    Number(u64),
}

/// Which recorders want the turn trace (`Runtime::set_recorder`). The
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

/// The kind of work that started part of a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CauseTag {
    /// Host input.
    Input,
    /// A due timer.
    Timer,
    /// A ready task.
    Task,
    /// Mount work with no host event.
    Mount,
    /// A DevTools call.
    Devtools,
}

/// What started part of a turn: a host event (its kind and node), a timer,
/// a task, mounts or a DevTools call. Formatted only when read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cause {
    /// The kind of work.
    pub tag: CauseTag,
    /// What happened (`"Activate"`, `"timer"`, a DevTools method).
    pub event: &'static str,
    /// The host node the event was for, if any.
    pub node: Option<kivact_types::NodeId>,
}

impl std::fmt::Display for Cause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.node {
            Some(node) => write!(f, "{} n{}", self.event, node.0),
            None => f.write_str(self.event),
        }
    }
}

/// One write of a signal in a turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceWrite {
    /// The written node.
    pub node: NodeId,
    /// The index of the cause it followed, if any.
    pub cause: Option<u32>,
    /// Value type name.
    pub ty: &'static str,
    /// Where the signal was created.
    pub created_at: &'static Location<'static>,
    /// Where it was written.
    pub at: &'static Location<'static>,
    /// Whether the write changed the value.
    pub changed: bool,
}

/// One memo or effect run in a turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceRun {
    /// The node that ran.
    pub node: NodeId,
    /// The index of the cause it followed, if any.
    pub cause: Option<u32>,
    /// `"memo"`, `"render"`, `"internal"` or `"effect"`.
    pub kind: &'static str,
    /// Where the memo or effect was created.
    pub at: &'static Location<'static>,
    /// Whether the run changed its value.
    pub changed: bool,
}

/// One dependency edge a write marked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceEdge {
    /// The changed source or memo.
    pub source: NodeId,
    /// The subscriber marked dirty.
    pub target: NodeId,
}

/// One turn stage and the time it took.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageSpan {
    /// The stage.
    pub stage: Stage,
    /// The App clock (ns) when it started.
    pub start_ns: u64,
    /// How long it took (ns).
    pub duration_ns: u64,
}

/// What one turn did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurnRecord {
    /// `Turn::Setup` for the notes of the App's setup, else the turn's number.
    pub turn: Turn,
    /// Host events delivered.
    pub events: usize,
    /// What started work, in order.
    pub causes: Vec<Cause>,
    /// Writes of signals.
    pub writes: Vec<TraceWrite>,
    /// Memo and effect runs.
    pub reruns: Vec<TraceRun>,
    /// Dependency edges marked by writes.
    pub edges: Vec<TraceEdge>,
    /// Stage spans.
    pub stages: Vec<StageSpan>,
    /// Number of memo and effect runs.
    pub runs: usize,
    /// Writes committed to the engine.
    pub ops: usize,
    /// Time spent in the turn (ns).
    pub duration_ns: u64,
    /// Notes (stale events, duplicate styles, diagnostics).
    pub notes: Vec<String>,
    /// Entries dropped at the per-list cap.
    pub dropped: u32,
}
