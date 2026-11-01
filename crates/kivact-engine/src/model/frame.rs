//! Role: Frame clocks, frame requests, smooth scrolls and drawability.
//!
//! The frame group: the host clock, the frame ticks kivact and the model
//! asked for, the programmatic scrolls animating, the display period and
//! whether there is a surface to draw on.
//!
//! Invariants:
//! - At most one tick per display period: a request due at or after a
//!   quarter period before the period's end is ticked at its own time, an
//!   earlier one at the period's end ([`Model::frame_deadline`]). The
//!   period is the window's display period; a change of it reaches the
//!   core's frame planning as an environment change.
//! - Without a surface no tick is delivered, and state transitions keep
//!   their place in time: the pause is skipped on resume. The same stop is
//!   the client's: every tick carries the engine's one `MotionClock` (the
//!   time stood still so far, when the last stop ended), a snapshot taken
//!   when the tick is queued.
//! - A wake less than a quarter period late stamps the tick with its due
//!   time, so a display-rate loop does not add the shell's wake latency to
//!   every period. Missed ticks are never made up: a later wake (after a
//!   pause too) delivers one tick stamped now, and the schedule starts
//!   again from it ([`Model::tick`]).
//! - The environment the client sees (`HostEnv`: the display period,
//!   reduced motion, drawability, and the window's scale, size, fullscreen,
//!   theme, safe area and locale) lives here, once. [`Model::update_env`]
//!   queues one `EnvChanged` per change of any of it, and none when the
//!   value did not change.
//! - A smooth scroll allocates its state only while it animates; reduced
//!   motion makes it a jump. It eases out (cubic) over 240 ms.

use crate::model::Model;
use kivact_wire::{HostEnv, HostEvent, MotionClock, NodeId, Point};
use state::SmoothScroll;

pub(crate) mod state;

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
        if let Some(scrolls) = &mut self.frame.smooth_scrolls {
            scrolls.remove(&id);
        }
        if smooth && !self.frame.reduced_motion() {
            self.frame
                .smooth_scrolls
                .get_or_insert_with(Default::default)
                .insert(
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

    /// Change the environment the client sees with `change`; when the
    /// result differs, queue it whole as one `EnvChanged`. A change of
    /// drawability also pauses or resumes the frame clocks.
    pub(crate) fn update_env(&mut self, change: impl FnOnce(&mut HostEnv)) {
        let mut env = self.frame.env.clone();
        change(&mut env);
        if env == self.frame.env {
            return;
        }
        if env.surface_active != self.frame.env.surface_active {
            if env.surface_active {
                if let Some(paused) = self.frame.state_paused_at.take() {
                    let now = self.frame.now;
                    self.shift_state_times(paused, now);
                    self.frame.offset_ns = self
                        .frame
                        .offset_ns
                        .saturating_add(now.saturating_sub(paused));
                    self.frame.resumed_ns = now;
                }
            } else {
                self.frame.state_paused_at = Some(self.frame.now);
            }
        }
        self.a11y.invalidate();
        self.frame.env = env.clone();
        self.push(HostEvent::EnvChanged(env));
    }

    /// Whether there is a surface to draw on, reported to the client when it
    /// changes.
    pub(crate) fn set_surface(&mut self, active: bool) {
        self.update_env(|env| env.surface_active = active);
    }

    /// State transitions moved: retarget them, and tick now when no frame
    /// is asked for.
    pub(super) fn retarget_frame(&mut self) {
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
        if let Some(mut scrolls) = self.frame.smooth_scrolls.take() {
            scrolls.retain(|id, scroll| {
                let began = *scroll.began.get_or_insert(stamp);
                let t = if self.frame.reduced_motion() {
                    1.0
                } else {
                    (stamp.saturating_sub(began) as f64 / 240_000_000.0).min(1.0) as f32
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
                self.frame.smooth_scrolls = Some(scrolls);
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
        MotionClock {
            offset_ns: self.frame.offset_ns,
            resumed_ns: self.frame.resumed_ns,
        }
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
