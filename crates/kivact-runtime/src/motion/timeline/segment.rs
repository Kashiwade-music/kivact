//! Role: The segments of a timeline: a track's move or a call, with its span.

use kivact_types::motion::Transition;

use crate::Callback;

/// One segment of a timeline's sequence.
pub(super) struct SegmentDef {
    /// Start time (ms) on the timeline.
    pub(super) start: f64,
    /// Length (ms): the delay plus the iterations of one transition; zero for
    /// a call.
    pub(super) len: f64,
    pub(super) kind: SegmentKind,
}

pub(super) enum SegmentKind {
    /// A track moves to `to` with `transition`.
    Track {
        track: usize,
        to: Vec<f64>,
        transition: Transition,
        /// Length of one iteration (ms): the duration, or a spring's rest time.
        one_ms: f64,
    },
    /// A callback at the start time.
    Call(Callback<()>),
}
