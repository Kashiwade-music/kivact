//! Role: This module stores motion state and frame loop records.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::Duration;

use kivact_reactive::Signal;
use kivact_reactive::UiError;
use kivact_reactive::Value;
use kivact_reactive::{Owner, Runtime};
use kivact_wire::MotionClock;

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
/// Produced by the framework; read its fields.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct MotionStats {
    /// Live frame loops.
    pub loops: usize,
    /// Live animations.
    pub animations: usize,
    /// Frame ticks delivered so far.
    pub ticks: u64,
}

/// Per-App state of the motion subsystem, driven by the host that runs the
/// App's turns: it installs the state (`default`) as a runtime extension when
/// the App is created, feeds it the host's frame period (`set_frame_period`)
/// and the environment projections it reads (`attach_env`), asks it which
/// frame to request (`plan_frame`) and delivers `FrameTick`s
/// (`on_frame_tick`) and applies a newly enabled reduced-motion preference
/// (`reduce_motion`). The engine's `MotionClock` a tick carries is the one
/// place that says a stop of the frame clock ended; this state keeps no stop
/// of its own.
#[derive(Default)]
pub struct MotionState {
    frames: RefCell<Frames>,
    /// Display refresh period reported by the host (0 before `start`).
    frame_period_ns: Cell<u64>,
    /// The App's reduced-motion projection of the host environment, once
    /// the root is mounted.
    reduced_motion: Cell<Option<Value<bool>>>,
    /// The App's surface-activity projection, once the root is mounted.
    surface_active: Cell<Option<Value<bool>>>,
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

    /// Record the host's display refresh period in ns.
    pub fn set_frame_period(&self, ns: u64) {
        self.frame_period_ns.set(ns);
    }

    /// Attach the environment projections frame planning reads (untracked):
    /// the host's reduced-motion preference and whether a surface can be
    /// drawn on. The view root calls it once the App's environment exists.
    pub fn attach_env(&self, reduced_motion: Value<bool>, surface_active: Value<bool>) {
        self.reduced_motion.set(Some(reduced_motion));
        self.surface_active.set(Some(surface_active));
    }

    /// Whether the host asks for reduced motion (untracked read).
    pub(crate) fn reduced_motion(&self) -> bool {
        self.reduced_motion
            .get()
            .and_then(|v| kivact_reactive::untrack(|| v.try_get()))
            .unwrap_or(false)
    }

    fn surface_active(&self) -> bool {
        self.surface_active
            .get()
            .and_then(|v| kivact_reactive::untrack(|| v.try_get()))
            .unwrap_or(true)
    }

    fn period(&self) -> u64 {
        self.frame_period_ns.get().max(1)
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
                a.resume(paused, clock.resumed_ns);
            }
            for l in self.live_loops() {
                l.start_ns.set(now);
                l.index.set(0);
                l.last_tick.set(None);
            }
        }
    }

    /// Handle a `FrameTick` at host time `now` (ns; `present` is the host's
    /// estimate of when the frame is presented; `clock` the engine's motion
    /// clock at the tick): apply the clock, run due frame loops, then step
    /// animations.
    pub fn on_frame_tick(&self, rt: &Runtime, now: u64, present: u64, clock: &MotionClock) {
        self.apply_clock(now, clock);
        let period = self.period();
        {
            let mut m = self.frames.borrow_mut();
            m.requested = None;
            m.last_tick = Some(now);
            m.ticks += 1;
        }
        for l in self.live_loops() {
            let Some(due) = l.due(period) else { continue };
            if due > now {
                continue;
            }
            let delta = l
                .last_tick
                .get()
                .map(|t| now.saturating_sub(t))
                .unwrap_or(0);
            l.last_tick.set(Some(now));
            let frame = l.frames.get();
            l.frames.set(frame + 1);
            if let Some(FrameRate::Hz(_)) = l.current.get() {
                // advance to the first slot after now (no catch-up bursts)
                let mut i = l.index.get() + 1;
                l.index.set(i);
                while l.due(period).is_some_and(|d| d <= now) {
                    i += 1;
                    l.index.set(i);
                }
            }
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
                if a.active() {
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
        let period = self.period();
        let now = rt.now_ns();
        let last = self.frames.borrow().last_tick;
        let mut want: Option<u64> = None;
        for l in self.live_loops() {
            if let Some(d) = l.due(period) {
                want = Some(want.map_or(d, |w| w.min(d)));
            }
        }
        if self.live_anims().iter().any(|a| a.active()) {
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
    fn active(&self) -> bool;
    fn step(&self, now_ns: u64);
    /// Apply a newly enabled reduced-motion preference to an existing run.
    /// Explicitly paused values keep their state until their owner resumes them.
    fn finish_decorative(&self);
    /// The frame clock stopped at `paused_ns` and went on at `now_ns`:
    /// move the animation's times by the pause, so it goes on from where
    /// it stood (one started during the pause starts at `now_ns`; one after
    /// it is left as it is).
    fn resume(&self, paused_ns: u64, now_ns: u64);
}

pub(crate) struct LoopCell {
    pub(crate) owner: Owner,
    pub(crate) f: Rc<dyn Fn(FrameInfo) -> Result<(), UiError>>,
    pub(crate) rate: Signal<Option<FrameRate>>,
    pub(crate) current: Cell<Option<FrameRate>>,
    pub(crate) start_ns: Cell<u64>,
    pub(crate) index: Cell<u64>,
    pub(crate) last_tick: Cell<Option<u64>>,
    pub(crate) frames: Cell<u64>,
}

impl LoopCell {
    /// Next due time on the host clock.
    pub(crate) fn due(&self, period_ns: u64) -> Option<u64> {
        match self.current.get()? {
            FrameRate::Hz(hz) => Some(
                self.start_ns.get() + ((self.index.get() + 1) as f64 * 1e9 / hz).round() as u64,
            ),
            FrameRate::Display => {
                Some(self.last_tick.get().unwrap_or(self.start_ns.get()) + period_ns)
            }
        }
    }
}
