//! Role: The calls a playback move crosses, in every repetition.

use super::super::Repeat;
use super::segment::{SegmentDef, SegmentKind};

/// The repetitions of a playback: a call is due once in each one it reaches.
pub(super) struct Reps {
    /// Length of one repetition (ms).
    dur: f64,
    /// Index of the last repetition (infinite for an endless playback).
    last: f64,
    /// Whether an odd repetition runs mirrored (alternate).
    mirror: bool,
}

/// Whether a move reached the end of the playback.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Reach {
    /// Stopped short of the end.
    Within,
    /// Reached the end, which then counts as crossed.
    End,
}

/// A move of the playback from `from` to `to` (ms of playback time).
pub(super) struct Move {
    pub(super) from: f64,
    pub(super) to: f64,
    pub(super) forward: bool,
    pub(super) reach: Reach,
}

impl Move {
    /// Whether the move crosses the playback time `at`: a forward move the
    /// times in `[from, to)` (in `[from, to]` when it reached the end), a
    /// backward move the mirror of that.
    fn crosses(&self, at: f64) -> bool {
        let end = self.reach == Reach::End;
        if self.forward {
            self.from <= at && if end { at <= self.to } else { at < self.to }
        } else {
            at <= self.from && if end { at >= self.to } else { at > self.to }
        }
    }
}

impl Reps {
    /// The repetitions of a playback of `repeat` times, each `dur` ms long.
    pub(super) fn new(dur: f64, repeat: Repeat, mirror: bool) -> Reps {
        let last = match repeat {
            Repeat::Times(n) => f64::from(n.saturating_sub(1)),
            Repeat::Forever => f64::INFINITY,
        };
        Reps { dur, last, mirror }
    }

    /// The indices of the call segments that `mv` crosses in some repetition,
    /// in playback order.
    pub(super) fn crossed(&self, segments: &[SegmentDef], mv: &Move) -> Vec<usize> {
        if !(mv.from.is_finite() && mv.to.is_finite()) {
            return Vec::new();
        }
        let (lo, hi) = (mv.from.min(mv.to), mv.from.max(mv.to));
        let mut due = Vec::new();
        for (index, seg) in segments.iter().enumerate() {
            if !matches!(seg.kind, SegmentKind::Call(_)) {
                continue;
            }
            for time in self.times(seg.start, lo, hi) {
                if mv.crosses(time) {
                    due.push((time, index));
                }
            }
        }
        let sign = if mv.forward { 1.0 } else { -1.0 };
        due.sort_by(|a, b| (sign * a.0).total_cmp(&(sign * b.0)));
        due.into_iter().map(|(_, index)| index).collect()
    }

    /// The playback times of a call at local time `at` in each repetition
    /// that the span `lo..=hi` touches. A mirrored repetition runs its
    /// local time backwards, as the tracks do.
    fn times(&self, at: f64, lo: f64, hi: f64) -> Vec<f64> {
        if self.dur <= 0.0 {
            return vec![at];
        }
        let first = (lo / self.dur).floor().clamp(0.0, self.last);
        let end = (hi / self.dur).floor().clamp(0.0, self.last);
        let mut times = Vec::new();
        let mut rep = first;
        while rep <= end {
            let local = if self.mirror && rep % 2.0 == 1.0 {
                self.dur - at
            } else {
                at
            };
            times.push(rep * self.dur + local);
            rep += 1.0;
        }
        times
    }
}
