//! Role: Own frame requests, motion clocks and smooth scroll progress.

use std::collections::{BTreeMap, BTreeSet};

use kivact_types::{Environment, NodeId, Point, ThemeHint};

use crate::contract::InputOrigin;

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

/// The instant of the event, turn or tick being handled, in both time bases.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Now {
    /// The host clock (ns): frame deadlines and wakes are in it.
    pub(crate) host_ns: u64,
    /// The App's motion time (ns) at `host_ns`: every motion record starts and steps in it.
    pub(crate) motion_ns: u64,
    /// Motion ns per host ns at `host_ns` (0 while the App clock stands still).
    pub(crate) rate: f64,
}

impl Now {
    /// The instant `host_ns` read on the App clock `clock`.
    pub(crate) fn at(host_ns: u64, clock: &kivact_types::clock::Clock) -> Self {
        Now {
            host_ns,
            motion_ns: clock.motion_ns(host_ns),
            rate: clock.rate(),
        }
    }
}

// xtask-allow(state): drawable is the drawability last reported to the client (start_env/update_env only); last_frame is the host stamp of the last delivered tick (tick only); independent, all four combinations occur
/// Frame requests, animation clocks and the environment the client sees.
pub(crate) struct Frame {
    /// The instant being handled (`Frame::set_now`).
    pub(super) now: Now,
    /// Whether the window can draw, as last reported to the client.
    pub(super) drawable: bool,
    pub(super) frames: FrameRequests,
    pub(super) smooth_scrolls: BTreeMap<NodeId, SmoothScroll>,
    /// The host stamp (ns) of the last delivered tick (`None`: none yet).
    pub(super) last_frame: Option<u64>,
    /// The environment as the client last saw it: the one source of the
    /// display period, reduced motion and drawability.
    pub(super) env: Environment,
    /// The painted nodes with a shader clock; each needs frames while it is
    /// painted. A projection, written only by `Model::refresh_shader_clocks`
    /// at a paint list rebuild.
    pub(super) shader_clocks: BTreeMap<NodeId, ClockNeed>,
    /// Where the shell event being handled came from (`Frame::set_origin`).
    pub(super) origin: InputOrigin,
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
            now: Now::default(),
            drawable: true,
            frames: FrameRequests::default(),
            smooth_scrolls: BTreeMap::new(),
            last_frame: None,
            env: Environment::default(),
            shader_clocks: BTreeMap::new(),
            origin: InputOrigin::Platform,
        }
    }
}

impl Frame {
    /// Schedule motion now: a frame at the current host time; returns the motion time a run starting now starts at.
    pub(crate) fn request_motion_frame(&mut self) -> u64 {
        self.frames.0.insert(self.now.host_ns);
        self.now.motion_ns
    }

    /// The instant of the event or turn being handled.
    pub(crate) fn set_now(&mut self, now: Now) {
        self.now = now;
    }

    /// The motion time (ns) of the event or turn being handled.
    pub(crate) fn now(&self) -> u64 {
        self.now.motion_ns
    }

    /// The host time (ns) of the event or turn being handled.
    pub(crate) fn host_now(&self) -> u64 {
        self.now.host_ns
    }

    /// The shell event being handled came from `origin`.
    pub(crate) fn set_origin(&mut self, origin: InputOrigin) {
        self.origin = origin;
    }

    /// Where the shell event being handled came from.
    pub(crate) fn origin(&self) -> InputOrigin {
        self.origin
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

    /// The display's frame period.
    pub(crate) fn period_ns(&self) -> u64 {
        self.env.frame_period_ns
    }

    /// Whether decorative transitions end at once.
    pub(crate) fn reduced_motion(&self) -> bool {
        self.env.reduced_motion
    }

    /// The host time (ns) at which motion time reaches `motion_ns`, read
    /// at the current instant: now when it already has; a display period
    /// from now while motion time stands still.
    pub(crate) fn deadline_after(&self, motion_ns: u64) -> u64 {
        if motion_ns <= self.now.motion_ns {
            return self.now.host_ns;
        }
        if self.now.rate > 0.0 {
            let wait = (motion_ns - self.now.motion_ns) as f64 / self.now.rate;
            self.now.host_ns.saturating_add(wait.ceil() as u64)
        } else {
            self.now.host_ns.saturating_add(self.period_ns())
        }
    }

    /// Whether there is a surface to draw on.
    pub(crate) fn surface_active(&self) -> bool {
        self.drawable
    }

    /// Ask for a tick at or after the host time `at_ns`.
    pub(crate) fn request(&mut self, at_ns: u64) {
        self.frames.0.insert(at_ns);
    }

    /// Stop every smooth scroll where it is (the user took over).
    pub(crate) fn stop_smooth_scrolls(&mut self) {
        self.smooth_scrolls.clear();
    }
}
