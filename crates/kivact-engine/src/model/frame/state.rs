//! Role: The frame group's state: clocks, frame requests, smooth scrolls and drawability.

use std::collections::BTreeMap;

use kivact_wire::{HostEnv, Insets, MotionClock, NodeId, Point, Size, ThemeHint};

/// One active programmatic scroll; allocated only while animating.
pub(super) struct SmoothScroll {
    pub(super) start: Point,
    pub(super) target: Point,
    pub(super) began: Option<u64>,
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
    /// The stops that ended, as the client sees them in each tick.
    pub(super) motion: MotionClock,
    pub(super) frames: Vec<u64>,
    pub(super) smooth_scrolls: BTreeMap<NodeId, SmoothScroll>,
    pub(super) last_frame: Option<u64>,
    /// The environment as the client last saw it: the one source of the
    /// display period, reduced motion and drawability.
    pub(super) env: HostEnv,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            now: 0,
            clock: Clock::Running,
            motion: MotionClock::default(),
            frames: vec![],
            smooth_scrolls: BTreeMap::new(),
            last_frame: None,
            env: HostEnv {
                scale: 1.0,
                reduced_motion: false,
                theme_hint: ThemeHint::Light,
                frame_period_ns: arvane::DEFAULT_FRAME_PERIOD_NS,
                safe_area: Insets::default(),
                window: Size::default(),
                locale: None,
                fullscreen: false,
                surface_active: true,
            },
        }
    }
}

impl Frame {
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
        self.env.surface_active
    }

    /// Ask for a tick at or after `at_ns`.
    pub(crate) fn request(&mut self, at_ns: u64) {
        self.frames.push(at_ns);
    }

    /// Stop every smooth scroll where it is (the user took over).
    pub(crate) fn stop_smooth_scrolls(&mut self) {
        self.smooth_scrolls.clear();
    }
}
