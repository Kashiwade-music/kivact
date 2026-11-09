//! Role: The detailed turn trace the DevTools reads (hidden from the public API).

use std::collections::VecDeque;
use std::panic::Location;

use crate::reactive::host_types::Stage;
use crate::reactive::identity::NodeId;

/// Per-list cap of one turn's detailed record.
const CAP: usize = 2000;
/// Cap of finished turns that were not taken yet.
const DONE: usize = 256;

/// The kind of work that started a turn.
// xtask-allow(internal-contract): the core's DevTools reads the detailed trace.
#[doc(hidden)]
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

/// One cause of a turn.
// xtask-allow(internal-contract): the core's DevTools reads the detailed trace.
#[doc(hidden)]
pub struct CauseEntry {
    /// What kind of work started it.
    pub tag: CauseTag,
    /// The label the turn recorder wrote.
    pub label: String,
}

/// One turn stage and the time it took.
// xtask-allow(internal-contract): the core's DevTools reads the detailed trace.
#[doc(hidden)]
pub struct StageSpan {
    /// The stage.
    pub stage: Stage,
    /// The loop clock time (ns) the stage started at.
    pub start_ns: u64,
    /// How long the stage took (ns).
    pub duration_ns: u64,
}

/// One write in a turn.
// xtask-allow(internal-contract): the core's DevTools reads the detailed trace.
#[doc(hidden)]
pub struct DetailWrite {
    /// The written node.
    pub node: NodeId,
    /// The index of the last cause before the write, if any.
    pub cause: Option<u32>,
    /// Value type name.
    pub ty: &'static str,
    /// Where the signal was created.
    // xtask-allow(api-shape): diagnostic source positions retain Rust caller locations.
    pub created_at: &'static Location<'static>,
    /// Where it was written.
    // xtask-allow(api-shape): diagnostic source positions retain Rust caller locations.
    pub at: &'static Location<'static>,
    // xtask-allow(state-fields): passive record - one write or run for DevTools; both fields are independent facts of it.
    /// Whether the write changed the value.
    pub changed: bool,
}

/// One memo or effect run in a turn.
// xtask-allow(internal-contract): the core's DevTools reads the detailed trace.
#[doc(hidden)]
pub struct DetailRun {
    /// The run node.
    pub node: NodeId,
    /// The index of the last cause before the run, if any.
    pub cause: Option<u32>,
    /// `"memo"`, `"render"`, `"internal"` or `"effect"`.
    pub kind: &'static str,
    /// Where the memo or effect was created.
    // xtask-allow(api-shape): diagnostic source positions retain Rust caller locations.
    pub at: &'static Location<'static>,
    // xtask-allow(state-fields): passive record - one write or run for DevTools; both fields are independent facts of it.
    /// Whether the run changed its value.
    pub changed: bool,
}

/// One dependency edge marked in a turn.
// xtask-allow(internal-contract): the core's DevTools reads the detailed trace.
#[doc(hidden)]
pub struct DetailEdge {
    /// The changed source or memo.
    pub source: NodeId,
    /// The subscriber marked dirty.
    pub target: NodeId,
}

/// The detailed record of one turn.
// xtask-allow(internal-contract): the core's DevTools reads the detailed trace.
#[doc(hidden)]
#[derive(Default)]
pub struct TurnDetail {
    /// The turn number.
    pub turn: u64,
    /// How long the turn took (ns).
    pub duration_ns: u64,
    /// What started the work.
    pub causes: Vec<CauseEntry>,
    /// Writes, up to the per-list cap.
    pub writes: Vec<DetailWrite>,
    /// Runs, up to the per-list cap.
    pub runs: Vec<DetailRun>,
    /// Edges, up to the per-list cap.
    pub edges: Vec<DetailEdge>,
    /// Stage spans, up to the per-list cap.
    pub stages: Vec<StageSpan>,
    /// Entries dropped at the per-list cap.
    pub dropped: u32,
}

/// Off, or the detail of the turn being recorded (S-3: nothing exists while off).
#[derive(Default)]
pub(crate) enum DetailLog {
    #[default]
    Off,
    Recording(Box<DetailBuilder>),
}

/// The last cause recorded in the turn being recorded.
#[derive(Clone, Copy, Default)]
enum CauseSlot {
    /// No cause yet.
    #[default]
    Empty,
    /// The index of the last recorded cause.
    At(u32),
    /// A cause was dropped at the per-list cap.
    Dropped,
}

impl CauseSlot {
    /// The index, unless no cause was recorded or one was dropped.
    fn at(self) -> Option<u32> {
        match self {
            Self::At(index) => Some(index),
            Self::Empty | Self::Dropped => None,
        }
    }
}

/// The turn being recorded and the finished turns waiting for `take`.
pub(crate) struct DetailBuilder {
    /// The turn being recorded (what happens between two turns belongs to the next one).
    current: TurnDetail,
    /// Finished turns waiting for `take`.
    done: VecDeque<TurnDetail>,
    /// The index of the last cause recorded in this turn.
    cause: CauseSlot,
    /// The stage being timed and the turn-relative ns it started at.
    open_stage: Option<(Stage, u64)>,
}

impl DetailLog {
    /// Begin recording into a fresh builder; already recording keeps the turn.
    pub(crate) fn start(&mut self) {
        if matches!(self, Self::Off) {
            *self = Self::Recording(Box::new(DetailBuilder {
                current: TurnDetail::default(),
                done: VecDeque::new(),
                cause: CauseSlot::Empty,
                open_stage: None,
            }));
        }
    }

    /// Stop recording and drop the unfinished turn.
    pub(crate) fn stop(&mut self) {
        *self = Self::Off;
    }

    /// Whether a turn is being recorded.
    pub(crate) fn is_recording(&self) -> bool {
        matches!(self, Self::Recording(_))
    }

    /// Take the finished turns, oldest first.
    pub(crate) fn take(&mut self) -> Vec<TurnDetail> {
        let Self::Recording(builder) = self else {
            return Vec::new();
        };
        std::mem::take(&mut builder.done).into()
    }

    /// Name the turn being recorded.
    pub(crate) fn begin(&mut self, turn: u64) {
        if let Self::Recording(builder) = self {
            builder.current.turn = turn;
        }
    }

    /// Record a cause; writes and runs name the last one recorded.
    pub(crate) fn cause(&mut self, tag: CauseTag, label: String) {
        let Self::Recording(builder) = self else {
            return;
        };
        if builder.current.causes.len() < CAP {
            builder.current.causes.push(CauseEntry { tag, label });
            builder.cause = CauseSlot::At((builder.current.causes.len() - 1) as u32);
        } else {
            builder.cause = CauseSlot::Dropped;
            builder.current.dropped += 1;
        }
    }

    /// Record one write.
    pub(crate) fn write(
        &mut self,
        node: NodeId,
        ty: &'static str,
        created_at: &'static Location<'static>,
        at: &'static Location<'static>,
        changed: bool,
    ) {
        let Self::Recording(builder) = self else {
            return;
        };
        if builder.current.writes.len() < CAP {
            builder.current.writes.push(DetailWrite {
                node,
                cause: builder.cause.at(),
                ty,
                created_at,
                at,
                changed,
            });
        } else {
            builder.current.dropped += 1;
        }
    }

    /// Record one memo or effect run.
    pub(crate) fn run(
        &mut self,
        node: NodeId,
        kind: &'static str,
        at: &'static Location<'static>,
        changed: bool,
    ) {
        let Self::Recording(builder) = self else {
            return;
        };
        if builder.current.runs.len() < CAP {
            builder.current.runs.push(DetailRun {
                node,
                cause: builder.cause.at(),
                kind,
                at,
                changed,
            });
        } else {
            builder.current.dropped += 1;
        }
    }

    /// Record one dependency edge marked by a notification.
    pub(crate) fn edge(&mut self, source: NodeId, target: NodeId) {
        let Self::Recording(builder) = self else {
            return;
        };
        if builder.current.edges.len() < CAP {
            builder.current.edges.push(DetailEdge { source, target });
        } else {
            builder.current.dropped += 1;
        }
    }

    /// Close the open stage and open `stage` at turn-relative `elapsed_ns`.
    pub(crate) fn stage(&mut self, stage: Stage, base_ns: u64, elapsed_ns: u64) {
        let Self::Recording(builder) = self else {
            return;
        };
        if let Some((prev, prev_elapsed)) = builder.open_stage
            && builder.current.stages.len() < CAP
        {
            builder.current.stages.push(StageSpan {
                stage: prev,
                start_ns: base_ns + prev_elapsed,
                duration_ns: elapsed_ns.saturating_sub(prev_elapsed),
            });
        }
        builder.open_stage = Some((stage, elapsed_ns));
    }

    /// Close the open stage at `duration_ns` and move the turn to the finished list.
    pub(crate) fn end_turn(&mut self, base_ns: u64, duration_ns: u64) {
        let Self::Recording(builder) = self else {
            return;
        };
        if let Some((stage, start)) = builder.open_stage
            && builder.current.stages.len() < CAP
        {
            builder.current.stages.push(StageSpan {
                stage,
                start_ns: base_ns + start,
                duration_ns: duration_ns.saturating_sub(start),
            });
        }
        builder.current.duration_ns = duration_ns;
        let next = builder.current.turn + 1;
        let done = std::mem::replace(
            &mut builder.current,
            TurnDetail {
                turn: next,
                ..TurnDetail::default()
            },
        );
        if builder.done.len() == DONE {
            builder.done.pop_front();
        }
        builder.done.push_back(done);
        builder.cause = CauseSlot::Empty;
        builder.open_stage = None;
    }
}
