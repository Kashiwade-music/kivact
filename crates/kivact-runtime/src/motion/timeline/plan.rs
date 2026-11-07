//! Role: The segments of a timeline and their per-track plan: each track's
//! segments in start order, and the value each one starts from (design
//! §12.6, the track value rule).

use std::cell::RefCell;

use kivact_types::motion::Transition;

use super::super::values::Animatable;
use super::curve::seg_value;
use crate::{Callback, Signal};

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

/// A track's publishing side, type-erased so one timeline holds tracks of
/// several types.
pub(super) trait TrackSlot {
    /// The track's value before any segment.
    fn initial(&self) -> Vec<f64>;
    /// Publish `value` unless it is the value last published.
    fn publish(&self, value: &[f64]);
}

pub(super) struct TrackState<T: Animatable> {
    pub(super) value: Signal<T>,
    pub(super) initial: Vec<f64>,
    pub(super) last: RefCell<Vec<f64>>,
}

impl<T: Animatable> TrackSlot for TrackState<T> {
    fn initial(&self) -> Vec<f64> {
        self.initial.clone()
    }

    fn publish(&self, value: &[f64]) {
        if self.last.borrow().as_slice() == value {
            return;
        }
        *self.last.borrow_mut() = value.to_vec();
        let v = T::from_components(value);
        self.value.update(move |slot| *slot = v);
    }
}

/// The order of each track's segments and the value each one starts from.
/// Rebuilt whenever a segment or a track is added or moved.
#[derive(Default)]
pub(super) struct Plan {
    /// For each track, its segments' indices in start order (ties in add order).
    pub(super) order: Vec<Vec<usize>>,
    /// For each segment, the track value at its start (from the earlier segments).
    pub(super) from: Vec<Vec<f64>>,
}

pub(super) fn span_of(seg: &SegmentDef) -> (f64, f64) {
    (seg.start, seg.start + seg.len)
}

/// The per-track order and starting values of the segments (§12.6, the
/// track value rule).
pub(super) fn build_plan(segments: &[SegmentDef], tracks: &[Box<dyn TrackSlot>]) -> Plan {
    let mut order: Vec<Vec<usize>> = vec![Vec::new(); tracks.len()];
    for (index, seg) in segments.iter().enumerate() {
        if let SegmentKind::Track { track, .. } = &seg.kind {
            order[*track].push(index);
        }
    }
    let mut from: Vec<Vec<f64>> = vec![Vec::new(); segments.len()];
    for (track, indices) in order.iter_mut().enumerate() {
        indices.sort_by(|a, b| segments[*a].start.total_cmp(&segments[*b].start));
        let mut carried = tracks[track].initial();
        let mut previous: Option<usize> = None;
        for &index in indices.iter() {
            if let Some(p) = previous {
                carried = seg_value(
                    &segments[p],
                    &from[p],
                    segments[index].start - segments[p].start,
                );
            }
            from[index] = carried.clone();
            previous = Some(index);
        }
    }
    Plan { order, from }
}
