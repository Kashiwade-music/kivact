//! Role: Estimate release velocity from recent pointer samples.

use std::collections::VecDeque;
use std::time::Duration;

use crate::geometry::Point;

/// Position samples in one drag's coordinate space.
#[derive(Clone, Debug, PartialEq)]
pub struct DragVelocity {
    window_ns: u64,
    samples: VecDeque<(u64, Point)>,
}

impl DragVelocity {
    /// Use position changes over the most recent `window` to estimate speed.
    pub fn new(window: Duration) -> Self {
        Self {
            window_ns: window.as_nanos().min(u128::from(u64::MAX)) as u64,
            samples: VecDeque::new(),
        }
    }

    /// Drop the previous gesture and record the initial sample.
    pub fn reset(&mut self, at_ns: u64, position: Point) {
        self.samples.clear();
        self.record(at_ns, position);
    }

    /// Record a sample in monotonic nanoseconds. When events share a timestamp,
    /// the last position is used for that time.
    pub fn record(&mut self, at_ns: u64, position: Point) {
        if self.samples.back().is_some_and(|(last, _)| *last > at_ns) {
            return;
        }
        while self
            .samples
            .front()
            .is_some_and(|(at, _)| at_ns - *at > self.window_ns)
        {
            self.samples.pop_front();
        }
        if self.samples.back().is_some_and(|(at, _)| *at == at_ns) {
            self.samples.pop_back();
        }
        self.samples.push_back((at_ns, position));
    }

    /// Estimate velocity in logical pixels per millisecond. Returns zero when
    /// there is no sample pair or the pointer has rested beyond the window.
    pub fn pixels_per_ms(&self, at_ns: u64) -> Point {
        let Some(((first, start), (last, end))) = self.samples.front().zip(self.samples.back())
        else {
            return Point::new(0.0, 0.0);
        };
        if first == last || at_ns.saturating_sub(*last) > self.window_ns {
            return Point::new(0.0, 0.0);
        }
        let span_ms = (last - first) as f64 / 1_000_000.0;
        Point::new((end.x - start.x) / span_ms, (end.y - start.y) / span_ms)
    }
}
