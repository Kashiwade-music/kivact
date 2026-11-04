//! Role: This module stores motion state and frame loop records.

use std::cell::{Cell, RefCell};
use std::num::NonZeroU64;
use std::rc::{Rc, Weak};
use std::time::Duration;

use kivact_reactive::Signal;
use kivact_reactive::UiError;
use kivact_reactive::Value;
use kivact_reactive::{Owner, Runtime};
use kivact_types::MotionClock;

use super::values::{FrameInfo, FrameRate};

/// Frame-driven work of an App and the outstanding frame request.
#[derive(Default)]
struct Frames {
    loops: Vec<Weak<LoopCell>>,
    anims: Vec<Weak<dyn Anim>>,
    /// Time of the outstanding `RequestFrame`, if any.
    requested: Option<u64>,
    last_tick: Option<u64>,
    ticks: u64,
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

/// What frame planning reads of the host's environment, attached once by the
/// view root ([`MotionState::attach_env`]); each value is read untracked.
#[derive(Clone, Copy)]
pub struct MotionEnv {
    /// The host asks for reduced motion.
    pub reduced_motion: Value<bool>,
    /// A surface can be drawn on.
    pub surface_active: Value<bool>,
    /// The display refresh period in ns (positive).
    pub frame_period_ns: Value<u64>,
}

/// Per-App state of the motion subsystem, driven by the host that runs the
/// App's turns: it installs the state (`default`) as a runtime extension when
/// the App is created, attaches the environment it reads (`attach_env`:
/// reduced motion, surface activity and the frame period), asks it which
/// frame to request (`plan_frame`) and delivers `FrameTick`s
/// (`on_frame_tick`) and applies a newly enabled reduced-motion preference
/// (`reduce_motion`). The engine's `MotionClock` a tick carries is the one
/// place that says a stop of the frame clock ended; this state keeps no stop
/// of its own.
#[derive(Default)]
pub struct MotionState {
    frames: RefCell<Frames>,
    /// The host's environment, once the view root attached it.
    env: Cell<Option<MotionEnv>>,
    /// The part of the clock's `offset_ns` the animations are already moved
    /// by.
    applied_offset_ns: Cell<u64>,
}

impl MotionState {
    /// The motion state installed in `rt`'s App.
    #[inline]
    pub fn of(rt: &Runtime) -> &MotionState {
        rt.ext::<MotionState>()
    }

    /// Attach the host's environment frame planning reads (untracked). The
    /// view root calls it once, when the App's environment exists.
    pub fn attach_env(&self, env: MotionEnv) {
        let previous = self.env.replace(Some(env));
        assert!(
            previous.is_none(),
            "invariant: the motion environment is attached once"
        );
    }

    fn env(&self) -> MotionEnv {
        self.env.get().expect(
            "invariant: the view root attaches the motion environment before anything moves",
        )
    }

    /// Whether the host asks for reduced motion (untracked; a disposed root
    /// reads as no).
    pub(crate) fn reduced_motion(&self) -> bool {
        let v = self.env().reduced_motion;
        kivact_reactive::untrack(|| v.try_get()).unwrap_or(false)
    }

    /// Whether a surface can be drawn on (untracked; a disposed root reads as
    /// yes).
    fn surface_active(&self) -> bool {
        let v = self.env().surface_active;
        kivact_reactive::untrack(|| v.try_get()).unwrap_or(true)
    }

    /// The display refresh period; `None` once the view root is disposed (the
    /// App is closing and no frame work is left).
    fn period(&self) -> Option<NonZeroU64> {
        let v = self.env().frame_period_ns;
        let ns = kivact_reactive::untrack(|| v.try_get())?;
        Some(NonZeroU64::new(ns).expect(
            "invariant: hosts report a positive frame period (`Environment::frame_period_ns`)",
        ))
    }

    /// Track a frame loop (weakly; it is dropped with its owner).
    pub(crate) fn register_loop(&self, l: Weak<LoopCell>) {
        self.frames.borrow_mut().loops.push(l);
    }

    /// Track an animation (weakly; it is dropped with its owner).
    pub(crate) fn register_anim(&self, a: Weak<dyn Anim>) {
        self.frames.borrow_mut().anims.push(a);
    }

    fn live_loops(&self) -> Vec<Rc<LoopCell>> {
        let mut m = self.frames.borrow_mut();
        m.loops.retain(|w| w.strong_count() > 0);
        m.loops.iter().filter_map(|w| w.upgrade()).collect()
    }

    fn live_anims(&self) -> Vec<Rc<dyn Anim>> {
        let mut m = self.frames.borrow_mut();
        m.anims.retain(|w| w.strong_count() > 0);
        m.anims.iter().filter_map(|w| w.upgrade()).collect()
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
        let applied = self.applied_offset_ns.get();
        if clock.offset_ns > applied {
            self.applied_offset_ns.set(clock.offset_ns);
            let stopped = clock.offset_ns - applied;
            let paused = clock.resumed_ns.saturating_sub(stopped);
            for a in self.live_anims() {
                a.shift_clock(paused, clock.resumed_ns);
            }
            for l in self.live_loops() {
                if let Some(run) = l.run.get() {
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
            let mut m = self.frames.borrow_mut();
            m.requested = None;
            m.last_tick = Some(now);
            m.ticks += 1;
        }
        let Some(period) = self.period() else { return };
        let period = period.get();
        for l in self.live_loops() {
            let Some(run) = l.run.get() else { continue };
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
                if let Err(err) = f(info) {
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
        if !self.surface_active() {
            return None;
        }
        let period = self.period()?.get();
        let now = rt.now_ns();
        let last = self.frames.borrow().last_tick;
        let mut want: Option<u64> = None;
        for l in self.live_loops() {
            if let Some(run) = l.run.get() {
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
        let mut m = self.frames.borrow_mut();
        if m.requested.is_some_and(|r| r <= want) {
            return None;
        }
        m.requested = Some(want);
        Some(want)
    }

    /// Counts of what the subsystem holds now (live frame loops and
    /// animations, frame ticks delivered so far).
    pub fn stats(&self) -> MotionStats {
        MotionStats {
            loops: self.live_loops().len(),
            animations: self.live_anims().len(),
            ticks: self.frames.borrow().ticks,
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

pub(crate) struct LoopCell {
    pub(crate) owner: Owner,
    pub(crate) f: Rc<dyn Fn(FrameInfo) -> Result<(), UiError>>,
    /// The rate `FrameLoop::rate` readers see: written only by `set_run`.
    pub(crate) rate: Signal<Option<FrameRate>>,
    /// The running loop; `None` while stopped.
    pub(crate) run: Cell<Option<LoopRun>>,
    /// Callbacks so far (`FrameInfo::frame`).
    pub(crate) frames: Cell<u64>,
}

/// A running frame loop.
#[derive(Clone, Copy)]
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
        self.run.set(run);
        self.rate.set(run.map(|r| r.rate));
    }
}
