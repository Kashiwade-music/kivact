//! Role: One run of a transition between two lists of numbers, timed in motion ns.

use kivact_types::motion::{SpringRun, Transition};

/// One run: from `from` to `to`, drawn at `current`, started at `start`
/// (motion ns, `frame::state::Now`).
#[derive(Debug)]
pub(crate) struct MotionRun {
    pub(crate) from: Vec<f64>,
    pub(crate) to: Vec<f64>,
    /// The numbers drawn now.
    pub(crate) current: Vec<f64>,
    /// How it moves.
    pub(crate) timing: Transition,
    /// When it started (motion ns).
    pub(crate) start: u64,
    /// A spring's velocity, step time and iteration.
    pub(crate) spring: SpringRun,
}

impl MotionRun {
    /// A run from `from` to `to` starting at `now` (motion ns); `spring`
    /// keeps an interrupted spring's velocity.
    pub(crate) fn start(
        from: Vec<f64>,
        to: Vec<f64>,
        timing: Transition,
        mut spring: SpringRun,
        now: u64,
    ) -> Self {
        spring.start(now, &timing, &from, &to);
        MotionRun {
            current: from.clone(),
            from,
            to,
            timing,
            start: now,
            spring,
        }
    }

    /// Step to `now` (motion ns); `true` when the run ended (at `to`, or
    /// where an alternate count leaves it). A decorative run under
    /// reduced motion ends at `to` at once.
    pub(crate) fn step(&mut self, now: u64, reduced: bool) -> bool {
        if reduced && self.timing.decorative {
            self.current.clone_from(&self.to);
            return true;
        }
        self.timing.step(
            &self.from,
            &self.to,
            self.start,
            now,
            &mut self.current,
            &mut self.spring,
        )
    }
}
