//! Role: Frame ticks, smooth scrolls, env and surface changes, and the next deadline.

use super::state::{Clock, SmoothScroll};
use crate::contract::HostEvent;
use crate::model::Model;
use kivact_types::{Environment, MotionClock, NodeId, Point};

impl Model {
    /// Scroll `id` from `start` to `target` over the next frames (at once
    /// with reduced motion or unless `smooth`).
    pub(crate) fn scroll_smoothly(
        &mut self,
        id: NodeId,
        start: Point,
        target: Point,
        smooth: bool,
    ) {
        self.frame.smooth_scrolls.remove(&id);
        if smooth && !self.frame.reduced_motion() {
            self.frame.smooth_scrolls.insert(
                id,
                SmoothScroll {
                    start,
                    target,
                    began: None,
                },
            );
            let next = self
                .frame
                .last_frame
                .unwrap_or(0)
                .saturating_add(self.frame.period_ns());
            self.frame.frames.push(next);
        } else {
            self.set_scroll(id, target);
        }
    }

    /// The environment before the client starts (it gets it in `HostInfo`):
    /// stored with no event; the clock stands still from now when nothing can
    /// be drawn.
    pub(crate) fn start_env(&mut self, env: Environment, surface_active: bool) {
        self.frame.clock = if surface_active {
            Clock::Running
        } else {
            Clock::Stopped {
                since: self.frame.now,
            }
        };
        self.frame.env = env;
        self.frame.surface_active = surface_active;
    }

    /// Change the environment the client sees with `change`; when the
    /// result differs, queue it whole as one `EnvChanged`. A change of
    /// drawability also pauses or resumes the frame clocks.
    pub(crate) fn update_env(&mut self, change: impl FnOnce(&mut Environment, &mut bool)) {
        let mut env = self.frame.env.clone();
        let mut surface_active = self.frame.surface_active;
        change(&mut env, &mut surface_active);
        if env == self.frame.env && surface_active == self.frame.surface_active {
            return;
        }
        match (self.frame.clock, surface_active) {
            (Clock::Stopped { since }, true) => {
                let now = self.frame.now;
                self.shift_state_times(since, now);
                self.frame.motion.offset_ns = self
                    .frame
                    .motion
                    .offset_ns
                    .saturating_add(now.saturating_sub(since));
                self.frame.motion.resumed_ns = now;
                self.frame.clock = Clock::Running;
            }
            (Clock::Running, false) => {
                self.frame.clock = Clock::Stopped {
                    since: self.frame.now,
                };
            }
            _ => {}
        }
        self.a11y.invalidate();
        self.frame.env = env.clone();
        self.frame.surface_active = surface_active;
        debug_assert_eq!(
            self.frame.clock == Clock::Running,
            self.frame.surface_active,
            "invariant: the frame clock stops exactly while nothing can be drawn"
        );
        self.push(HostEvent::EnvChanged {
            environment: env,
            surface_active,
        });
    }

    /// Whether there is a surface to draw on, reported to the client when it
    /// changes.
    pub(crate) fn set_surface(&mut self, active: bool) {
        self.update_env(|_, surface_active| *surface_active = active);
    }

    /// State transitions moved: retarget them, and tick now when no frame
    /// is asked for.
    pub(in crate::model) fn retarget_frame(&mut self) {
        if self.retarget_states(self.frame.now, self.frame.reduced_motion())
            && self.frame.frames.is_empty()
        {
            self.frame.frames.push(self.frame.now);
        }
    }

    /// Deliver the frame tick due by `now`, if any. A tick is stamped with
    /// its due time when the wake is less than a quarter period late, so a
    /// display-rate loop (which asks for the next tick a period after the
    /// last one's stamp) keeps the display's rate instead of adding the
    /// shell's wake latency to every period; a later wake starts a new
    /// schedule at `now`.
    pub(crate) fn tick(&mut self, now: u64) {
        self.frame.now = now;
        let Some(due) = self
            .frame_deadline()
            .filter(|t| self.frame.surface_active() && *t <= now)
        else {
            return;
        };
        let period = self.frame.period_ns();
        let stamp = if now - due < period / 4 { due } else { now };
        self.frame.frames.retain(|t| *t > now);
        self.changes.restack |= self.motion.is_active();
        if self.step_states(stamp, self.frame.reduced_motion()) {
            self.frame.frames.push(stamp.saturating_add(period));
        }
        self.sync();
        if !self.frame.smooth_scrolls.is_empty() {
            let mut scrolls = std::mem::take(&mut self.frame.smooth_scrolls);
            scrolls.retain(|id, scroll| {
                let began = *scroll.began.get_or_insert(stamp);
                let t = if self.frame.reduced_motion() {
                    1.0
                } else {
                    (stamp.saturating_sub(began) as f64 / 240_000_000.0).min(1.0) as f64
                };
                let eased = 1.0 - (1.0 - t).powi(3);
                self.set_scroll(
                    *id,
                    Point {
                        x: scroll.start.x + (scroll.target.x - scroll.start.x) * eased,
                        y: scroll.start.y + (scroll.target.y - scroll.start.y) * eased,
                    },
                );
                t < 1.0 && self.node(*id).is_some()
            });
            if !scrolls.is_empty() {
                self.frame.frames.push(stamp.saturating_add(period));
                self.frame.smooth_scrolls = scrolls;
            }
            self.sync();
        }
        self.frame.last_frame = Some(stamp);
        self.push(HostEvent::FrameTick {
            now_ns: stamp,
            estimated_present_ns: stamp.saturating_add(period),
            clock: self.motion_clock(),
        });
    }

    /// The motion clock the next frame tick carries.
    fn motion_clock(&self) -> MotionClock {
        self.frame.motion
    }

    /// When the next frame tick is due: the earliest request, but at most
    /// one tick per display period. A request up to a quarter period before
    /// the period ends is on time, so a loop at the display rate is neither
    /// delayed by the period's rounding (a 60 Hz `FrameLoop` asks every
    /// 16 666 666 or 16 666 667 ns) nor by a late wake of the shell; an
    /// earlier request waits for the period's end.
    pub(crate) fn frame_deadline(&self) -> Option<u64> {
        let requested = self.frame.frames.iter().min().copied()?;
        let Some(last) = self.frame.last_frame else {
            return Some(requested);
        };
        let period = self.frame.period_ns();
        let next = last.saturating_add(period);
        let on_time = next.saturating_sub(period / 4);
        Some(if requested >= on_time {
            requested
        } else {
            next
        })
    }
}
