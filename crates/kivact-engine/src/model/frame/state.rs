//! Role: Own frame requests, motion clocks and smooth scroll progress.

use std::collections::{BTreeMap, BTreeSet};

use kivact_types::{Environment, NodeId, Point, ThemeHint, TimeScale};

/// One active programmatic scroll; allocated only while animating.
pub(super) enum SmoothScroll {
    Scheduled {
        start: Point,
        target: Point,
        easing: kivact_types::Easing,
    },
    Fling {
        velocity: Point,
        last_ns: u64,
    },
    Running {
        start: Point,
        target: Point,
        easing: kivact_types::Easing,
        began_ns: u64,
    },
}

#[derive(Default)]
pub(crate) struct FrameRequests(pub(crate) BTreeSet<u64>);

#[derive(Clone, Copy)]
pub(super) enum TickHistory {
    NeverTicked,
    Ticked { last_ns: u64 },
}

/// Whether the frame clock runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Clock {
    /// Ticks are delivered.
    Running,
    /// Nothing can be drawn since `since` (host ns): no tick; the state transitions
    /// are shifted by the stop when it ends.
    Stopped { since: u64 },
}

/// Frame requests, animation clocks and the environment the client sees.
pub(crate) struct Frame {
    pub(super) now: u64,
    /// Whether the frame clock runs; written only by `Model::start_env` and
    /// `Model::update_env`.
    pub(super) clock: Clock,
    pub(super) frames: FrameRequests,
    pub(super) smooth_scrolls: BTreeMap<NodeId, SmoothScroll>,
    pub(super) last_frame: TickHistory,
    /// The environment as the client last saw it: the one source of the
    /// display period, reduced motion and drawability.
    pub(super) env: Environment,
    /// The painted nodes with a shader clock; each needs frames while it is
    /// painted. A projection, written only by `Model::refresh_shader_clocks`
    /// at a paint list rebuild.
    pub(super) shader_clocks: BTreeMap<NodeId, ClockNeed>,
    /// The DevTools' time scale for the motion clock.
    pub(super) time_scale: TimeScale,
}

/// What a shader clock needs from the frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ClockNeed {
    /// A frame at every display tick.
    Display,
    /// Frames at this rate in hertz.
    Hz(f64),
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            now: 0,
            clock: Clock::Running,
            frames: FrameRequests::default(),
            smooth_scrolls: BTreeMap::new(),
            last_frame: TickHistory::NeverTicked,
            env: Environment::default(),
            shader_clocks: BTreeMap::new(),
            time_scale: TimeScale::default(),
        }
    }
}

impl Frame {
    /// Schedule motion at the current host time.
    pub(crate) fn request_motion_frame(&mut self) -> u64 {
        self.frames.0.insert(self.now);
        self.now
    }

    /// The host clock (ns) of the event or turn being handled.
    pub(crate) fn set_now(&mut self, now: u64) {
        self.now = now;
    }

    /// The theme the environment starts with (light unless a test driver
    /// starts dark); `Model::start_env` keeps it.
    pub(crate) fn start_theme(&mut self, theme: ThemeHint) {
        self.env.theme_hint = theme;
    }

    /// The theme the client sees.
    pub(crate) fn theme_hint(&self) -> ThemeHint {
        self.env.theme_hint
    }

    /// The host clock (ns) of the event or turn being handled.
    pub(crate) fn now(&self) -> u64 {
        self.now
    }

    /// The display's frame period.
    pub(crate) fn period_ns(&self) -> u64 {
        self.env.frame_period_ns
    }

    /// Whether decorative transitions end at once.
    pub(crate) fn reduced_motion(&self) -> bool {
        self.env.reduced_motion
    }

    /// Whether there is a surface to draw on.
    pub(crate) fn surface_active(&self) -> bool {
        matches!(self.clock, Clock::Running)
    }

    /// Ask for a tick at or after `at_ns`.
    pub(crate) fn request(&mut self, at_ns: u64) {
        self.frames.0.insert(at_ns);
    }

    /// Stop every smooth scroll where it is (the user took over).
    pub(crate) fn stop_smooth_scrolls(&mut self) {
        self.smooth_scrolls.clear();
    }
}
