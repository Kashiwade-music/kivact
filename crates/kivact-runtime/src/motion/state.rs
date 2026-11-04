//! Role: This module stores motion state and frame loop records.

use std::cell::{Cell, RefCell};
use std::num::NonZeroU64;
use std::rc::Rc;
use std::time::Duration;

use crate::Signal;
use crate::Value;
use crate::reactive::Handler;
use crate::{Owner, Runtime};
use kivact_types::MotionClock;
use kivact_types::cell::Registry;

use super::values::{FrameInfo, FrameRate};

/// Frame request and delivered tick history.
#[derive(Default)]
struct FrameClock {
    phase: FramePhase,
    offset: ClockOffset,
    ticks: u64,
}

#[derive(Default)]
struct ClockOffset(u64);

#[derive(Default)]
enum FramePhase {
    #[default]
    Idle,
    Requested {
        at: u64,
        last_tick: Option<u64>,
    },
    Ticked {
        last: u64,
    },
}

impl FramePhase {
    fn last_tick(&self) -> Option<u64> {
        match *self {
            Self::Idle => None,
            Self::Requested { last_tick, .. } => last_tick,
            Self::Ticked { last } => Some(last),
        }
    }
}

/// Counts of what the motion subsystem holds now ([`MotionState::stats`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MotionStats {
    /// Live frame loops.
    pub loops: usize,
    /// Live animations.
    pub animations: usize,
    /// Frame ticks delivered so far.
    pub ticks: u64,
}

/// Environment values installed at runtime construction and read untracked.
#[derive(Clone, Copy)]
pub struct MotionEnv {
    /// The host asks for reduced motion.
    pub reduced_motion: Value<bool>,
    /// A surface can be drawn on.
    pub surface_active: Value<bool>,
    /// The display refresh period in ns (positive).
    pub frame_period_ns: Value<u64>,
}

/// Per-App frame clock and weak registrations driven by host frame ticks.
#[derive(Default)]
pub struct MotionState {
    clock: RefCell<FrameClock>,
    loops: RefCell<Registry<LoopCell>>,
    anims: RefCell<Registry<dyn Anim>>,
}

impl MotionState {
    /// The motion state installed in `rt`'s App.
    #[inline]
    pub fn of(rt: &Runtime) -> &MotionState {
        rt.ext::<MotionState>()
    }

    /// Whether the host asks for reduced motion (untracked; a disposed root
    /// reads as no).
    pub(crate) fn reduced_motion(&self, rt: &Runtime) -> bool {
        let v = rt.ext::<MotionEnv>().reduced_motion;
        crate::untrack(|| v.try_get()).unwrap_or(false)
    }

    /// Whether a surface can be drawn on (untracked; a disposed root reads as
    /// yes).
    fn surface_active(&self, rt: &Runtime) -> bool {
        let v = rt.ext::<MotionEnv>().surface_active;
        crate::untrack(|| v.try_get()).unwrap_or(true)
    }

    /// The display refresh period; `None` once the view root is disposed (the
    /// App is closing and no frame work is left).
    fn period(&self, rt: &Runtime) -> Option<NonZeroU64> {
        let v = rt.ext::<MotionEnv>().frame_period_ns;
        let ns = crate::untrack(|| v.try_get())?;
        Some(NonZeroU64::new(ns).expect(
            "invariant: hosts report a positive frame period (`Environment::frame_period_ns`)",
        ))
    }

    /// Track a frame loop without owning it.
    pub(crate) fn register_loop(&self, l: &Rc<LoopCell>) {
        self.loops.borrow_mut().insert(l);
    }

    /// Track an animation without owning it.
    pub(crate) fn register_anim(&self, a: &Rc<dyn Anim>) {
        self.anims.borrow_mut().insert(a);
    }

    fn live_loops(&self) -> Vec<Rc<LoopCell>> {
        self.loops.borrow_mut().live()
    }

    fn live_anims(&self) -> Vec<Rc<dyn Anim>> {
        self.anims.borrow_mut().live()
    }

    /// An enabled preference completes active decorative runs immediately.
    /// It does not retarget sources, restart completed keyframes, or stop loops.
    /// The host calls it in the turn that reports the preference on, so no
    /// frame is needed to see it.
    pub fn reduce_motion(&self, rt: &Runtime) {
        rt.enter(|| {
            for animation in self.live_anims() {
                animation.finish_decorative();
            }
        });
    }

    /// Take over what the tick's `clock` says since the last tick: a stop of
    /// the frame clock that ended. Animations go on from where they stood
    /// and frame loops count from `now` again.
    fn apply_clock(&self, now: u64, clock: &MotionClock) {
        let applied = self.clock.borrow().offset.0;
        if clock.offset_ns > applied {
            self.clock.borrow_mut().offset.0 = clock.offset_ns;
            let stopped = clock.offset_ns - applied;
            let paused = clock.resumed_ns.saturating_sub(stopped);
            for a in self.live_anims() {
                a.shift_clock(paused, clock.resumed_ns);
            }
            for l in self.live_loops() {
                if let LoopState::Running(run) = l.state.get_untracked() {
                    l.set_run(Some(LoopRun {
                        start_ns: now,
                        index: 0,
                        last_tick: None,
                        ..run
                    }));
                }
            }
        }
    }

    /// Handle a `FrameTick` at host time `now` (ns; `present` is the host's
    /// estimate of when the frame is presented; `clock` the engine's motion
    /// clock at the tick): apply the clock, run due frame loops, then step
    /// animations.
    pub fn on_frame_tick(&self, rt: &Runtime, now: u64, present: u64, clock: &MotionClock) {
        self.apply_clock(now, clock);
        {
            let mut m = self.clock.borrow_mut();
            m.phase = FramePhase::Ticked { last: now };
            m.ticks += 1;
        }
        let Some(period) = self.period(rt) else {
            return;
        };
        let period = period.get();
        for l in self.live_loops() {
            let LoopState::Running(run) = l.state.get_untracked() else {
                continue;
            };
            if run.due(period) > now {
                continue;
            }
            let delta = run.last_tick.map_or(0, |t| now.saturating_sub(t));
            let mut next = LoopRun {
                last_tick: Some(now),
                ..run
            };
            if let FrameRate::Hz(_) = run.rate {
                // advance to the first slot after now (no catch-up bursts)
                next.index += 1;
                while next.due(period) <= now {
                    next.index += 1;
                }
            }
            l.set_run(Some(next));
            let frame = l.frames.get();
            l.frames.set(frame + 1);
            let info = FrameInfo {
                now: Duration::from_nanos(now),
                delta: Duration::from_nanos(delta),
                frame,
                present: Duration::from_nanos(present),
            };
            let f = l.f.clone();
            l.owner.run_handler(|| {
                if let Err(err) = f.call(info) {
                    l.owner.report(err);
                }
            });
        }
        // Animation steps run no user code.
        rt.enter(|| {
            for a in self.live_anims() {
                if a.is_running() {
                    a.step(now);
                }
            }
        });
    }

    /// Frame time to request from the host, if any frame-driven work exists.
    fn wanted_frame(&self, rt: &Runtime) -> Option<u64> {
        if !self.surface_active(rt) {
            return None;
        }
        let period = self.period(rt)?.get();
        let now = rt.now_ns();
        let last = self.clock.borrow().phase.last_tick();
        let mut want: Option<u64> = None;
        for l in self.live_loops() {
            if let LoopState::Running(run) = l.state.get_untracked() {
                let d = run.due(period);
                want = Some(want.map_or(d, |w| w.min(d)));
            }
        }
        if self.live_anims().iter().any(|a| a.is_running()) {
            let d = last.map(|t| t + period).unwrap_or(now).max(now);
            want = Some(want.map_or(d, |w| w.min(d)));
        }
        want
    }

    /// The time (ns) of a new `RequestFrame` the host must send, if new frame
    /// work appeared (recorded as the outstanding request).
    pub fn plan_frame(&self, rt: &Runtime) -> Option<u64> {
        let want = self.wanted_frame(rt)?;
        let mut m = self.clock.borrow_mut();
        if matches!(m.phase, FramePhase::Requested { at, .. } if at <= want) {
            return None;
        }
        m.phase = FramePhase::Requested {
            at: want,
            last_tick: m.phase.last_tick(),
        };
        Some(want)
    }

    /// Counts of what the subsystem holds now (live frame loops and
    /// animations, frame ticks delivered so far).
    pub fn stats(&self) -> MotionStats {
        MotionStats {
            loops: self.live_loops().len(),
            animations: self.live_anims().len(),
            ticks: self.clock.borrow().ticks,
        }
    }
}

/// Type-erased animation stepping for the motion driver.
pub(crate) trait Anim {
    /// A run is in progress (not paused).
    fn is_running(&self) -> bool;
    fn step(&self, now_ns: u64);
    /// Apply a newly enabled reduced-motion preference to an existing run.
    /// Explicitly paused values keep their state until their owner resumes them.
    fn finish_decorative(&self);
    /// The frame clock stopped at `paused_ns` and went on at `now_ns`:
    /// move the animation's times by the pause, so it goes on from where
    /// it stood (one started during the pause starts at `now_ns`; one after
    /// it is left as it is).
    fn shift_clock(&self, paused_ns: u64, now_ns: u64);
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum LoopState {
    Stopped,
    Running(LoopRun),
}

pub(crate) struct LoopCell {
    pub(crate) owner: Owner,
    pub(crate) f: Handler<FrameInfo>,
    pub(crate) state: Signal<LoopState>,
    pub(crate) frames: Cell<u64>,
}

/// A running frame loop.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct LoopRun {
    pub(crate) rate: FrameRate,
    /// When the loop started at this rate (host ns; `Hz` ticks count from here).
    pub(crate) start_ns: u64,
    /// Ticks at this rate so far.
    pub(crate) index: u64,
    /// Host time of the last callback; `None` before the first one.
    pub(crate) last_tick: Option<u64>,
}

impl LoopRun {
    /// Next due time on the host clock.
    pub(crate) fn due(&self, period_ns: u64) -> u64 {
        match self.rate {
            FrameRate::Hz(hz) => {
                self.start_ns + ((self.index + 1) as f64 * 1e9 / hz).round() as u64
            }
            FrameRate::Display => self.last_tick.unwrap_or(self.start_ns) + period_ns,
        }
    }
}

impl LoopCell {
    /// Start, change or stop (`None`) the loop.
    pub(crate) fn set_run(&self, run: Option<LoopRun>) {
        self.state
            .set(run.map_or(LoopState::Stopped, LoopState::Running));
    }
}
