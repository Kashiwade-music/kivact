//! Role: The frame group's state: clocks, frame requests, smooth scrolls and drawability.

use std::collections::BTreeMap;

use kivact_wire::{HostEnv, Insets, NodeId, Point, Size, ThemeHint};

/// One active programmatic scroll; allocated only while animating.
pub(super) struct SmoothScroll {
    pub(super) start: Point,
    pub(super) target: Point,
    pub(super) began: Option<u64>,
}

/// Frame requests, animation clocks and the environment the client sees.
pub(crate) struct Frame {
    pub(super) now: u64,
    /// When the frame clock stopped (no surface), until it resumes.
    pub(super) state_paused_at: Option<u64>,
    /// The time the frame clock stood still in the stops that ended
    /// (`MotionClock::offset_ns`).
    pub(super) offset_ns: u64,
    /// When the last stop ended (`MotionClock::resumed_ns`).
    pub(super) resumed_ns: u64,
    pub(super) frames: Vec<u64>,
    pub(super) smooth_scrolls: Option<BTreeMap<NodeId, SmoothScroll>>,
    pub(super) last_frame: Option<u64>,
    /// The environment as the client last saw it: the one source of the
    /// display period, reduced motion and drawability.
    pub(super) env: HostEnv,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            now: 0,
            state_paused_at: None,
            offset_ns: 0,
            resumed_ns: 0,
            frames: vec![],
            smooth_scrolls: None,
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

    /// The environment before the client starts: written in place, with
    /// no event (the client gets it in `HostInfo`).
    pub(crate) fn start_env(&mut self) -> &mut HostEnv {
        &mut self.env
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
        self.smooth_scrolls = None;
    }
}
