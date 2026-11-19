//! Role: The App's motion time: host time less the stops, run at the DevTools' rate.
//!
//! Invariants:
//! - Motion time stands still while the clock is stopped (no window can
//!   draw) and goes on from where it stood when it resumes: the stopped
//!   host time is summed in `stopped_ns`, never added to the start of a
//!   run afterwards.
//! - Motion time passes `scale` times as fast as running host time; a new
//!   scale continues from the motion time of the moment it is set.
//! - Motion time never decreases.

use crate::motion::{MotionClock, TimeScale};

/// Whether the App's motion time runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockPhase {
    /// It runs; `resumed_ns` (host ns) is when the last stop ended (0: none yet).
    Running {
        /// Host time (ns) the last stop ended at.
        resumed_ns: u64,
    },
    /// It stands still since `since_ns` (host ns): no window can draw.
    Stopped {
        /// Host time (ns) the stop began at.
        since_ns: u64,
        /// Host time (ns) the stop before it ended at.
        resumed_ns: u64,
    },
}

/// The App's one motion clock: host time (ns) less the time no window
/// could draw, scaled by the DevTools' time scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    /// The DevTools' rate, anchored in running time (`running_ns`).
    scale: TimeScale,
    phase: ClockPhase,
    /// The host time (ns) every ended stop stood still, summed.
    stopped_ns: u64,
}

impl Clock {
    /// A running clock at rate 1 that never stopped.
    pub const fn new() -> Self {
        Clock {
            scale: TimeScale {
                anchor_host_ns: 0,
                anchor_motion_ns: 0,
                scale: 1.0,
            },
            phase: ClockPhase::Running { resumed_ns: 0 },
            stopped_ns: 0,
        }
    }

    /// Whether it runs or stands still, and since when.
    pub fn phase(&self) -> ClockPhase {
        self.phase
    }

    /// The host time (ns) that ran up to `host_ns`: the stops left out.
    pub fn running_ns(&self, host_ns: u64) -> u64 {
        let at = match self.phase {
            ClockPhase::Running { .. } => host_ns,
            ClockPhase::Stopped { since_ns, .. } => since_ns.min(host_ns),
        };
        at.saturating_sub(self.stopped_ns)
    }

    /// The motion time (ns) at host time `host_ns`.
    pub fn motion_ns(&self, host_ns: u64) -> u64 {
        self.scale.motion_ns(self.running_ns(host_ns))
    }

    /// Motion ns per host ns now: the scale while it runs, 0 while it stands still.
    pub fn rate(&self) -> f64 {
        match self.phase {
            ClockPhase::Running { .. } => self.scale.scale,
            ClockPhase::Stopped { .. } => 0.0,
        }
    }

    /// Run at `scale` from host time `host_ns` on (the DevTools' time scale).
    pub fn set_scale(&mut self, host_ns: u64, scale: f64) {
        self.scale = self.scale.rescaled(self.running_ns(host_ns), scale);
    }

    /// Stand still from host time `host_ns` (nothing when already stopped).
    pub fn stop(&mut self, host_ns: u64) {
        if let ClockPhase::Running { resumed_ns } = self.phase {
            self.phase = ClockPhase::Stopped {
                since_ns: host_ns,
                resumed_ns,
            };
        }
    }

    /// Run again from host time `host_ns` (nothing when running).
    pub fn resume(&mut self, host_ns: u64) {
        if let ClockPhase::Stopped { since_ns, .. } = self.phase {
            self.stopped_ns = self
                .stopped_ns
                .saturating_add(host_ns.saturating_sub(since_ns));
            self.phase = ClockPhase::Running {
                resumed_ns: host_ns,
            };
        }
    }

    /// The snapshot a `FrameTick` carries to the client.
    pub fn motion_clock(&self) -> MotionClock {
        let resumed_ns = match self.phase {
            ClockPhase::Running { resumed_ns } | ClockPhase::Stopped { resumed_ns, .. } => {
                resumed_ns
            }
        };
        MotionClock {
            offset_ns: self.stopped_ns,
            resumed_ns,
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
