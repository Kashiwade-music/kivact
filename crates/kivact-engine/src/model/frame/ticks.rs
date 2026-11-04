//! Role: Deliver frame ticks, smooth scrolling and environment changes.

use super::state::{Clock, SmoothScroll, TickHistory};
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
        easing: Option<kivact_types::Easing>,
    ) {
        self.frame.smooth_scrolls.remove(&id);
        if let Some(easing) = easing.filter(|_| !self.frame.reduced_motion()) {
            self.frame.smooth_scrolls.insert(
                id,
                SmoothScroll::Scheduled {
                    start,
                    target,
                    easing,
                },
            );
            let last = match self.frame.last_frame {
                TickHistory::NeverTicked => 0,
                TickHistory::Ticked { last_ns } => last_ns,
            };
            let next = last.saturating_add(self.frame.period_ns());
            self.frame.frames.0.insert(next);
        } else {
            self.set_scroll(id, target);
        }
    }

    pub(crate) fn start_scroll_fling(&mut self, id: NodeId, velocity: Point) {
        if self.frame.reduced_motion()
            || velocity.x.hypot(velocity.y) < self.frame.env.pointer.fling_min_velocity
        {
            self.settle_scroll_snap(id);
            return;
        }
        self.frame.smooth_scrolls.insert(
            id,
            SmoothScroll::Fling {
                velocity,
                last_ns: self.frame.now,
            },
        );
        self.frame
            .frames
            .0
            .insert(self.frame.now.saturating_add(self.frame.period_ns()));
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
    }

    /// Change the environment the client sees with `change`; when the
    /// result differs, queue it whole as one `EnvChanged`. A change of
    /// drawability also pauses or resumes the frame clocks.
    pub(crate) fn update_env(&mut self, change: impl FnOnce(&mut Environment, &mut bool)) {
        let mut env = self.frame.env.clone();
        let mut surface_active = self.frame.surface_active();
        change(&mut env, &mut surface_active);
        if env == self.frame.env && surface_active == self.frame.surface_active() {
            return;
        }
        match (self.frame.clock, surface_active) {
            (Clock::Stopped { since }, true) => {
                let now = self.frame.now;
                self.shift_state_times(since, now);
                self.shift_size_times(since, now);
                self.shift_layout_times(since, now);
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
            && self.frame.frames.0.is_empty()
        {
            self.frame.frames.0.insert(self.frame.now);
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
        self.frame.frames.0.retain(|t| *t > now);
        self.changes.restack |= self.motion.is_active();
        let sizes = self.step_sizes(stamp, self.frame.reduced_motion());
        let layout = self.step_layout_motion(stamp, self.frame.reduced_motion());
        if self.step_states(stamp, self.frame.reduced_motion()) || sizes || layout {
            self.frame.frames.0.insert(stamp.saturating_add(period));
        }
        self.sync();
        if !self.frame.smooth_scrolls.is_empty() {
            let mut scrolls = std::mem::take(&mut self.frame.smooth_scrolls);
            scrolls.retain(|id, scroll| {
                if let SmoothScroll::Fling { velocity, last_ns } = scroll {
                    let dt = stamp.saturating_sub(*last_ns) as f64 / 1_000_000.0;
                    *last_ns = stamp;
                    let settings = self.frame.env.pointer;
                    let decay = if settings.fling_deceleration.is_finite()
                        && settings.fling_deceleration > 0.0
                        && settings.fling_deceleration < 1.0
                    {
                        settings.fling_deceleration
                    } else {
                        0.998
                    };
                    let retention = decay.powf(dt);
                    let distance = (retention - 1.0) / decay.ln();
                    let Some(node) = self.node(*id) else {
                        return false;
                    };
                    let at = node.scroll();
                    let moved = self.set_scroll(
                        *id,
                        Point {
                            x: at.x + velocity.x * distance,
                            y: at.y + velocity.y * distance,
                        },
                    );
                    velocity.x *= retention;
                    velocity.y *= retention;
                    let active = moved
                        && !self.frame.reduced_motion()
                        && velocity.x.hypot(velocity.y) >= settings.fling_min_velocity.max(0.001);
                    if !active {
                        self.settle_scroll_snap(*id);
                    }
                    return active;
                }
                let (start, target, began, easing) = match *scroll {
                    SmoothScroll::Scheduled {
                        start,
                        target,
                        easing,
                    } => {
                        *scroll = SmoothScroll::Running {
                            start,
                            target,
                            easing,
                            began_ns: stamp,
                        };
                        (start, target, stamp, easing)
                    }
                    SmoothScroll::Fling { .. } => unreachable!("invariant: fling handled above"),
                    SmoothScroll::Running {
                        start,
                        target,
                        easing,
                        began_ns,
                    } => (start, target, began_ns, easing),
                };
                let t = if self.frame.reduced_motion() {
                    1.0
                } else {
                    (stamp.saturating_sub(began) as f64 / 240_000_000.0).min(1.0)
                };
                let eased = easing.apply(t);
                self.set_scroll(
                    *id,
                    Point {
                        x: start.x + (target.x - start.x) * eased,
                        y: start.y + (target.y - start.y) * eased,
                    },
                );
                t < 1.0 && self.node(*id).is_some()
            });
            if !scrolls.is_empty() {
                self.frame.frames.0.insert(stamp.saturating_add(period));
                self.frame.smooth_scrolls = scrolls;
            }
            self.sync();
        }
        self.frame.last_frame = TickHistory::Ticked { last_ns: stamp };
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
        let requested = self.frame.frames.0.first().copied()?;
        let TickHistory::Ticked { last_ns: last } = self.frame.last_frame else {
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
