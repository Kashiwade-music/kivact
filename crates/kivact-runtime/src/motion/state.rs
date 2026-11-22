//! Role: This module stores motion state and frame loop records.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;
use std::rc::{Rc, Weak};
use std::time::Duration;

use crate::Owner;
use crate::Signal;
use crate::Value;
use crate::driver::Runtime;
use crate::reactive::Handler;
use kivact_types::MotionClock;
use kivact_types::TimeScale;

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

/// What a [`MotionState::animations`] snapshot lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimKind {
    /// An `Animated` value.
    Animated,
    /// A timeline.
    Timeline,
    /// A frame loop.
    FrameLoop,
}

/// Whether one animation is idle, running or paused.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RunStatus {
    /// No run is in progress.
    #[default]
    Idle,
    /// A run is in progress.
    Running,
    /// A run is paused (never for a frame loop).
    Paused,
}

/// One animation of a [`MotionState::animations`] snapshot.
pub struct AnimInfo {
    /// What kind of animation it is.
    pub kind: AnimKind,
    /// Where the owner that owns it was created.
    pub created_at: Option<&'static std::panic::Location<'static>>,
    /// The owner it is registered under.
    pub owner: crate::OwnerId,
    /// The status of its latest run.
    pub status: RunStatus,
    /// Whether reduced motion stops it (the default) or it is essential.
    pub decorative: bool,
}

impl AnimInfo {
    /// Whether a run is in progress.
    pub fn is_running(&self) -> bool {
        self.status == RunStatus::Running
    }

    /// Whether it is paused (never for a frame loop).
    pub fn is_paused(&self) -> bool {
        self.status == RunStatus::Paused
    }
}

/// A loop's or an animation's registration. Its cell holds it; when the
/// cell drops (its owner was disposed), the id leaves every map.
pub(crate) struct MotionToken {
    app: crate::AppId,
    pub(crate) id: u64,
}

impl Drop for MotionToken {
    fn drop(&mut self) {
        if let Some(rt) = Runtime::get(self.app) {
            rt.motion().forget(self.id);
        }
    }
}

// xtask-allow(state): loops and anims hold every registration by id; running_loops and running_anims hold the ids whose phase is running and are written only by those phase changes; clock is the frame request phase; next_id allocates ids and time_scale is a DevTools setting
/// Per-App frame clock and weak registrations driven by host frame ticks.
#[derive(Default)]
pub struct MotionState {
    clock: RefCell<FrameClock>,
    /// Every live frame loop by id (DevTools, stats, the clock shift).
    loops: RefCell<BTreeMap<u64, Weak<LoopCell>>>,
    /// Every live animation and timeline by id.
    anims: RefCell<BTreeMap<u64, Weak<dyn Anim>>>,
    /// The loops that are started; written only by `LoopCell::set_run`.
    running_loops: RefCell<BTreeSet<u64>>,
    /// The animations whose phase is running; written only by their phase changes.
    running_anims: RefCell<BTreeSet<u64>>,
    /// Allocates the ids of registrations (creation order).
    next_id: Cell<u64>,
    /// How fast motion time runs for DevTools.
    time_scale: Cell<TimeScale>,
}

/// The motion clock (ns) of `rt`: host time through the DevTools time scale.
pub(crate) fn motion_now(rt: &Runtime) -> u64 {
    rt.motion().time_scale.get().motion_ns(rt.now_ns())
}

/// Where the owner that owns an animation was created, if its App is still registered.
pub(super) fn owner_source(owner: Owner) -> Option<&'static std::panic::Location<'static>> {
    let rt = Runtime::get(owner.app())?;
    rt.owner_info(owner.id()).map(|info| info.created_at)
}

impl MotionState {
    /// Set how fast motion time runs (0 pauses, 1 is normal) from now on.
    pub fn set_time_scale(&self, rt: &Runtime, scale: f64) {
        self.time_scale
            .set(self.time_scale.get().rescaled(rt.now_ns(), scale));
        rt.request_turn();
    }

    /// The current motion time scale (the DevTools `list_animations` reads it).
    pub fn time_scale(&self) -> f64 {
        self.time_scale.get().scale
    }

    /// Snapshot of the live animations (animated values, timelines and frame loops).
    pub fn animations(&self) -> Vec<AnimInfo> {
        let mut list: Vec<AnimInfo> = self.live_anims().iter().map(|a| a.info()).collect();
        for l in self.live_loops() {
            list.push(AnimInfo {
                kind: AnimKind::FrameLoop,
                created_at: owner_source(l.owner),
                owner: l.owner.id(),
                status: if l.run.get().is_some() {
                    RunStatus::Running
                } else {
                    RunStatus::Idle
                },
                decorative: l.motion.get() == LoopMotion::Decorative,
            });
        }
        list
    }

    /// Whether the host asks for reduced motion (untracked; a disposed root
    /// reads as no).
    pub(crate) fn reduced_motion(&self, rt: &Runtime) -> bool {
        let v = rt.motion_env().reduced_motion;
        crate::untrack(|| v.try_get()).unwrap_or(false)
    }

    /// Whether a surface can be drawn on (untracked; a disposed root reads as
    /// yes).
    fn surface_active(&self, rt: &Runtime) -> bool {
        let v = rt.motion_env().surface_active;
        crate::untrack(|| v.try_get()).unwrap_or(true)
    }

    /// The display refresh period; `None` once the view root is disposed (the
    /// App is closing and no frame work is left).
    fn period(&self, rt: &Runtime) -> Option<NonZeroU64> {
        let v = rt.motion_env().frame_period_ns;
        let ns = crate::untrack(|| v.try_get())?;
        Some(NonZeroU64::new(ns).expect(
            "invariant: hosts report a positive frame period (`Environment::frame_period_ns`)",
        ))
    }

    /// A new registration id of `app`.
    pub(crate) fn token(&self, app: crate::AppId) -> MotionToken {
        let id = self.next_id.get() + 1;
        self.next_id.set(id);
        MotionToken { app, id }
    }

    /// Remove `id` from every map (`MotionToken::drop`).
    fn forget(&self, id: u64) {
        self.loops.borrow_mut().remove(&id);
        self.anims.borrow_mut().remove(&id);
        self.running_loops.borrow_mut().remove(&id);
        self.running_anims.borrow_mut().remove(&id);
    }

    /// Track a frame loop without owning it.
    pub(crate) fn register_loop(&self, id: u64, l: &Rc<LoopCell>) {
        self.loops.borrow_mut().insert(id, Rc::downgrade(l));
    }

    /// Track an animation without owning it.
    pub(crate) fn register_anim(&self, id: u64, a: &Rc<dyn Anim>) {
        self.anims.borrow_mut().insert(id, Rc::downgrade(a));
    }

    /// A loop started (`true`) or stopped.
    pub(crate) fn set_loop_running(&self, id: u64, running: bool) {
        let mut set = self.running_loops.borrow_mut();
        if running {
            set.insert(id);
        } else {
            set.remove(&id);
        }
    }

    /// An animation's phase became running (`true`) or left it.
    pub(crate) fn set_anim_running(&self, id: u64, running: bool) {
        let mut set = self.running_anims.borrow_mut();
        if running {
            set.insert(id);
        } else {
            set.remove(&id);
        }
    }

    // The maps hold only live registrations (`MotionToken::drop` removes
    // the others), so reading them prunes nothing.
    fn live_loops(&self) -> Vec<Rc<LoopCell>> {
        self.loops
            .borrow()
            .values()
            .filter_map(Weak::upgrade)
            .collect()
    }

    fn live_anims(&self) -> Vec<Rc<dyn Anim>> {
        self.anims
            .borrow()
            .values()
            .filter_map(Weak::upgrade)
            .collect()
    }

    /// The started loops, in creation order.
    fn running_loops(&self) -> Vec<Rc<LoopCell>> {
        let loops = self.loops.borrow();
        self.running_loops
            .borrow()
            .iter()
            .filter_map(|id| loops.get(id).and_then(Weak::upgrade))
            .collect()
    }

    /// The running animations, in creation order.
    fn running_anims(&self) -> Vec<Rc<dyn Anim>> {
        let anims = self.anims.borrow();
        self.running_anims
            .borrow()
            .iter()
            .filter_map(|id| anims.get(id).and_then(Weak::upgrade))
            .collect()
    }

    /// An enabled preference completes active decorative runs immediately.
    /// It does not retarget sources, restart completed keyframes, or stop loops.
    /// The host calls it in the turn that reports the preference on, so no
    /// frame is needed to see it.
    pub fn reduce_motion(&self, rt: &Runtime) {
        rt.enter(|| {
            for animation in self.running_anims() {
                animation.finish_decorative();
            }
        });
    }

    /// Reduced motion was turned off: the running decorative frame loops
    /// count from now, as they do when a surface resumes (the counterpart of
    /// [`MotionState::reduce_motion`]). The host calls it only for the change
    /// from on to off, in the turn that reports it.
    pub fn restore_motion(&self, rt: &Runtime) {
        let now = motion_now(rt);
        rt.enter(|| {
            for l in self.running_loops() {
                if l.motion.get() != LoopMotion::Decorative {
                    continue;
                }
                if let Some(run) = l.run.get() {
                    l.set_run(Some(LoopRun {
                        rate: run.rate,
                        cadence: Cadence::Fresh { start_ns: now },
                    }));
                }
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
                if let Some(run) = l.run.get() {
                    l.set_run(Some(LoopRun {
                        rate: run.rate,
                        cadence: Cadence::Fresh { start_ns: now },
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
        let reduced = self.reduced_motion(rt);
        for l in self.running_loops() {
            if !l.is_present() || skips_under(&l, reduced) {
                continue;
            }
            let Some(run) = l.run.get() else {
                continue;
            };
            if run.due(period) > now {
                continue;
            }
            let delta = run.last_tick().map_or(0, |t| now.saturating_sub(t));
            let mut index = run.index();
            if let FrameRate::Hz(_) = run.rate {
                // advance to the first slot after now (no catch-up bursts)
                index += 1;
                let mut next = run.ticked(index, now);
                while next.due(period) <= now {
                    index += 1;
                    next = run.ticked(index, now);
                }
                l.set_run(Some(next));
            } else {
                l.set_run(Some(run.ticked(index, now)));
            }
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
            for a in self.running_anims() {
                a.step(now);
            }
        });
    }

    /// Frame time to request from the host, if any frame-driven work exists.
    fn wanted_frame(&self, rt: &Runtime) -> Option<u64> {
        if !self.surface_active(rt) {
            return None;
        }
        let period = self.period(rt)?.get();
        let now = motion_now(rt);
        let last = self.clock.borrow().phase.last_tick();
        let reduced = self.reduced_motion(rt);
        let mut want: Option<u64> = None;
        for l in self.running_loops() {
            if !l.is_present() || skips_under(&l, reduced) {
                continue;
            }
            if let Some(run) = l.run.get() {
                let d = run.due(period);
                want = Some(want.map_or(d, |w| w.min(d)));
            }
        }
        if !self.running_anims().is_empty() {
            let d = last.map(|t| t + period).unwrap_or(now).max(now);
            want = Some(want.map_or(d, |w| w.min(d)));
        }
        want
    }

    /// The time (ns) of a new `RequestFrame` the host must send, if new frame
    /// work appeared (recorded as the outstanding request).
    pub fn plan_frame(&self, rt: &Runtime) -> Option<u64> {
        if self.running_loops.borrow().is_empty() && self.running_anims.borrow().is_empty() {
            return None;
        }
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
    /// A snapshot of this animation for the DevTools.
    fn info(&self) -> AnimInfo;
    /// Apply a newly enabled reduced-motion preference to an existing run.
    /// Explicitly paused values keep their state until their owner resumes them.
    fn finish_decorative(&self);
    /// The frame clock stopped at `paused_ns` and went on at `now_ns`:
    /// move the animation's times by the pause, so it goes on from where
    /// it stood (one started during the pause starts at `now_ns`; one after
    /// it is left as it is).
    fn shift_clock(&self, paused_ns: u64, now_ns: u64);
}

/// An optional host observation restricts scheduling independently of run state.
#[derive(Clone, Copy)]
pub(crate) enum LoopGate {
    Always,
    While(Value<bool>),
}

/// Whether a frame loop or a timeline is decorative (stopped under reduced
/// motion, the default) or essential (kept running).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum LoopMotion {
    #[default]
    Decorative,
    Essential,
}

// xtask-allow(state): run is the loop's phase (None when stopped); rate is its reactive projection, written only when the requested rate changes, never per tick; frames counts callbacks; gate and motion are setup options; registration is the motion token
pub(crate) struct LoopCell {
    /// Its registration: dropping the cell removes it from every map.
    pub(crate) registration: MotionToken,
    pub(crate) owner: Owner,
    pub(crate) f: Handler<FrameInfo>,
    /// The running schedule (`None`: stopped), advanced at each tick.
    pub(crate) run: Cell<Option<LoopRun>>,
    /// The requested rate, for `FrameLoop::rate`.
    pub(crate) rate: Signal<Option<FrameRate>>,
    pub(crate) frames: Cell<u64>,
    pub(crate) gate: Cell<LoopGate>,
    /// Set in setup by `FrameLoop::essential`.
    pub(crate) motion: Cell<LoopMotion>,
}

/// A running frame loop.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct LoopRun {
    pub(crate) rate: FrameRate,
    pub(crate) cadence: Cadence,
}

/// What a running loop has done at its current rate: nothing yet, or some
/// callbacks (the first one makes the last tick known).
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Cadence {
    /// No callback yet at this rate.
    Fresh { start_ns: u64 },
    /// Callbacks have run at this rate.
    Ticking {
        start_ns: u64,
        index: u64,
        last_tick: u64,
    },
}

impl LoopRun {
    /// When the loop started at this rate (host ns; `Hz` ticks count from here).
    fn start_ns(&self) -> u64 {
        match self.cadence {
            Cadence::Fresh { start_ns } | Cadence::Ticking { start_ns, .. } => start_ns,
        }
    }

    /// Host time of the last callback; `None` before the first one.
    pub(crate) fn last_tick(&self) -> Option<u64> {
        match self.cadence {
            Cadence::Fresh { .. } => None,
            Cadence::Ticking { last_tick, .. } => Some(last_tick),
        }
    }

    /// Ticks at this rate so far.
    fn index(&self) -> u64 {
        match self.cadence {
            Cadence::Fresh { .. } => 0,
            Cadence::Ticking { index, .. } => index,
        }
    }

    /// This run with a callback delivered at `now`.
    fn ticked(&self, index: u64, now: u64) -> Self {
        Self {
            rate: self.rate,
            cadence: Cadence::Ticking {
                start_ns: self.start_ns(),
                index,
                last_tick: now,
            },
        }
    }

    /// Next due time on the host clock.
    pub(crate) fn due(&self, period_ns: u64) -> u64 {
        match self.rate {
            FrameRate::Hz(hz) => {
                self.start_ns() + ((self.index() + 1) as f64 * 1e9 / hz).round() as u64
            }
            FrameRate::Display => self.last_tick().unwrap_or(self.start_ns()) + period_ns,
        }
    }
}

/// Whether a frame loop is held back now: a decorative loop under reduced
/// motion requests no frame and gets no callback.
fn skips_under(l: &LoopCell, reduced: bool) -> bool {
    reduced && l.motion.get() == LoopMotion::Decorative
}

impl LoopCell {
    fn is_present(&self) -> bool {
        match self.gate.get() {
            LoopGate::Always => true,
            LoopGate::While(gate) => gate.get_untracked(),
        }
    }

    /// Start, change or stop (`None`) the loop.
    pub(crate) fn set_run(&self, run: Option<LoopRun>) {
        let before = self.run.get();
        self.run.set(run);
        // The rate signal changes only with the requested rate, never per tick.
        if before.map(|r| r.rate) != run.map(|r| r.rate) {
            self.rate.set(run.map(|r| r.rate));
        }
        if before.is_some() != run.is_some()
            && let Some(rt) = Runtime::get(self.owner.app())
        {
            rt.motion()
                .set_loop_running(self.registration.id, run.is_some());
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: A registration leaves every motion map when its owner is disposed.

    use std::panic::Location;
    use std::rc::Rc;

    use crate::reactive::host_types::Host;

    struct Quiet;
    impl Host for Quiet {
        fn wake(&self) {}
    }

    #[test]
    fn a_disposed_frame_loop_leaves_every_motion_map() {
        let rt = crate::driver::Runtime::register(
            Rc::new(Quiet),
            0,
            crate::driver::TaskState::new(None, crate::driver::ThreadWaker::new(|| {})),
        );
        let at = Location::caller();
        assert_eq!(rt.create_owner(None, at), rt.root);
        rt.enter(|| {
            let child = rt.root().child(at);
            child.with_setup(|| {
                let looped = crate::frame_loop(|_| ());
                looped.start(crate::FrameRate::Display);
            });
            assert_eq!(rt.motion().loops.borrow().len(), 1);
            assert_eq!(rt.motion().running_loops.borrow().len(), 1);
            child.dispose();
            assert!(
                rt.motion().loops.borrow().is_empty(),
                "the registration left with its owner"
            );
            assert!(rt.motion().running_loops.borrow().is_empty());
        });
        rt.shutdown();
    }
}
