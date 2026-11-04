//! Role: `Animated<T>`: a value that follows a target `Value<T>` with a `Transition`.
//! Its current value is a graph source, so memos and effects
//! track it like any signal.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use super::state::{Anim, MotionState};
use super::transition::{Kind, Transition};
use super::values::Animatable;
use crate::Prop as InputProp;
use crate::{
    AppId, EffectKind, HandlerResult, IntoReactive, IntoValue, Owned, Owner, Runtime, Signal,
    Value, require_effectful,
};
use kivact_types::motion::SpringRun;

pub(crate) struct AnimCell<T: 'static> {
    pub(crate) owner: Owner,
    pub(crate) value: Signal<T>,
    pub(crate) transition: Transition,
    pub(crate) from: RefCell<Vec<f64>>,
    pub(crate) to: RefCell<Vec<f64>>,
    pub(crate) current: RefCell<Vec<f64>>,
    /// A spring's velocity, step time and iteration (`Transition::step`).
    pub(crate) spring: RefCell<SpringRun>,
    pub(crate) phase: Signal<AnimPhase>,
}

/// Where an animated value stands.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnimPhase {
    /// At rest on `to`: no run.
    Idle,
    /// A naturally completed run at its target.
    Completed,
    /// A run toward `to` that started at `start_ns` on the App's clock (its
    /// delay and timing count from here).
    Running { start_ns: u64 },
    /// A run stopped by `Animated::pause` at `at_ns`; `resume` continues it.
    Paused { start_ns: u64, at_ns: u64 },
}

/// Why a run ends.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunEnd {
    /// It reached its target, in time or at once under reduced motion:
    /// `on_complete` runs.
    Completed,
    /// `jump_to`, or a target equal to the current value (no run starts):
    /// nothing is reported.
    Cancelled,
}

/// The time `t` after a stop at `paused` that went on at `now`: times up to
/// the stop move by its length, times during it move to `now`, later ones
/// stay.
fn shift(t: u64, paused: u64, now: u64) -> u64 {
    if t <= paused {
        t.saturating_add(now.saturating_sub(paused))
    } else if t <= now {
        now
    } else {
        t
    }
}

fn comps<T: Animatable>(v: &T) -> Vec<f64> {
    let mut c = Vec::with_capacity(T::COMPONENTS);
    v.to_components(&mut c);
    c
}

impl<T: Animatable> AnimCell<T> {
    fn publish(&self, c: &[f64]) {
        let v = T::from_components(c);
        *self.current.borrow_mut() = c.to_vec();
        self.value.update(move |slot| *slot = v);
    }

    fn set_phase(&self, phase: AnimPhase) {
        self.phase.set(phase);
    }

    fn finish(&self, end: RunEnd) {
        self.set_phase(match end {
            RunEnd::Completed => AnimPhase::Completed,
            RunEnd::Cancelled => AnimPhase::Idle,
        });
        let to = self.to.borrow().clone();
        self.publish(&to);
        self.spring.borrow_mut().velocity.fill(0.0);
    }

    /// Run from `from` toward `to` on `clock`, or end at once: a target equal
    /// to the current value starts no run, and under reduced motion a
    /// decorative run completes at once.
    fn begin(&self, start_ns: u64, reduced: bool) {
        let wire = &self.transition;
        if *self.from.borrow() == *self.to.borrow() && !matches!(wire.kind, Kind::Keyframes { .. })
        {
            self.finish(RunEnd::Cancelled);
        } else if reduced && wire.decorative {
            self.finish(RunEnd::Completed);
        } else {
            self.set_phase(AnimPhase::Running { start_ns });
        }
    }

    /// Start moving from the current value to `target` at `now`.
    pub(crate) fn retarget(&self, target: &T, now: u64, reduced: bool) {
        let to = comps(target);
        let phase = self.phase.get_untracked();
        if *self.to.borrow() == to && !matches!(phase, AnimPhase::Idle | AnimPhase::Completed) {
            return;
        }
        *self.from.borrow_mut() = self.current.borrow().clone();
        *self.to.borrow_mut() = to;
        {
            let mut run = self.spring.borrow_mut();
            if run.velocity.len() != T::COMPONENTS {
                run.velocity = vec![0.0; T::COMPONENTS];
            }
            run.restart(now);
        }
        // A paused run stays paused, now toward the new target, and keeps its
        // current value.
        if let AnimPhase::Paused { .. } = phase {
            self.set_phase(AnimPhase::Paused {
                start_ns: now,
                at_ns: now,
            });
            return;
        }
        self.begin(now, reduced);
    }
}

impl<T: Animatable> Anim for AnimCell<T> {
    fn is_running(&self) -> bool {
        matches!(self.phase.get_untracked(), AnimPhase::Running { .. })
    }

    fn finish_decorative(&self) {
        if self.is_running() && self.transition.decorative {
            self.finish(RunEnd::Completed);
        }
    }

    fn shift_clock(&self, paused: u64, now: u64) {
        match self.phase.get_untracked() {
            AnimPhase::Idle | AnimPhase::Completed => return,
            AnimPhase::Running { start_ns } => self.set_phase(AnimPhase::Running {
                start_ns: shift(start_ns, paused, now),
            }),
            AnimPhase::Paused { start_ns, at_ns } => {
                let at_ns = if at_ns <= now {
                    at_ns.max(paused).saturating_add(now.saturating_sub(paused))
                } else {
                    at_ns
                };
                self.set_phase(AnimPhase::Paused {
                    start_ns: shift(start_ns, paused, now),
                    at_ns,
                });
            }
        }
        let mut run = self.spring.borrow_mut();
        run.last_ns = shift(run.last_ns, paused, now);
    }

    fn step(&self, now: u64) {
        let AnimPhase::Running { start_ns } = self.phase.get_untracked() else {
            return;
        };
        let mut current = self.current.borrow().clone();
        let done = self.transition.step(
            &self.from.borrow(),
            &self.to.borrow(),
            start_ns,
            now,
            &mut current,
            &mut self.spring.borrow_mut(),
        );
        if done {
            self.finish(RunEnd::Completed);
        } else {
            self.publish(&current);
        }
    }
}

/// An animated value (Copy). Use it wherever a `Value<T>` is accepted.
pub struct Animated<T: 'static> {
    app: AppId,
    cell: Owned<AnimCell<T>>,
    value: Value<T>,
    running: Value<bool>,
}

impl<T: 'static> Clone for Animated<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for Animated<T> {}

/// A value following `target` with `transition`; an interrupted
/// animation continues from the current value (and velocity). Owned by
/// the current setup owner; setup only.
///
/// ```ignore
/// let x = animated(move || if open.get() { 240.0 } else { 0.0 }, Transition::spring(Spring::DEFAULT));
/// ```
#[track_caller]
pub fn animated<T: Animatable>(target: impl IntoValue<T>, transition: Transition) -> Animated<T> {
    Animated::create(None, target, transition)
}

impl<T: Animatable> Animated<T> {
    /// Follow a target starting at `start`, including the initial mount.
    #[track_caller]
    pub fn from(start: T, target: impl IntoValue<T>, transition: Transition) -> Self {
        Self::create(Some(start), target, transition)
    }

    #[track_caller]
    fn create(start: Option<T>, target: impl IntoValue<T>, transition: Transition) -> Self {
        let at = Location::caller();
        let owner = Owner::current("animated", at);
        let rt = owner.runtime();
        let target = target.into_prop();
        let explicit_start = start.is_some();
        let initial = start.unwrap_or_else(|| crate::untrack(|| target.get()));
        let c = comps(&initial);
        let app = rt.id();
        let value: Signal<T> = owner.signal(initial, at);
        let cell = Rc::new(AnimCell {
            owner,
            value,
            transition,
            from: RefCell::new(c.clone()),
            to: RefCell::new(c.clone()),
            current: RefCell::new(c),
            spring: RefCell::new(SpringRun {
                velocity: vec![0.0; T::COMPONENTS],
                ..SpringRun::default()
            }),
            phase: owner.signal(AnimPhase::Idle, at),
        });
        let anim: Rc<dyn Anim> = cell.clone();
        MotionState::of(&rt).register_anim(&anim);
        drop(anim);
        let stored = owner.store("Animated", cell.clone(), at);
        let weak = Rc::downgrade(&cell);
        let retarget = move |t: &T| {
            let Some(rt) = Runtime::get(app) else { return };
            let Some(cell) = weak.upgrade() else { return };
            let now = rt.now_ns();
            let reduced = MotionState::of(&rt).reduced_motion(&rt);
            crate::untrack(|| cell.retarget(t, now, reduced));
            rt.request_turn();
        };
        if explicit_start {
            owner.effect(
                EffectKind::Internal,
                move || {
                    let t = crate::pure(|| target.get());
                    retarget(&t);
                },
                at,
            );
        } else {
            crate::reactive::watch(move || target.get(), EffectKind::Internal, retarget);
        }
        let phase = cell.phase;
        let running = crate::memo(move || matches!(phase.get(), AnimPhase::Running { .. }));
        Animated {
            app,
            cell: stored,
            value: Value::from(value),
            running,
        }
    }
}

impl<T: Animatable> Animated<T> {
    fn cell(&self, at: &'static Location<'static>) -> Rc<AnimCell<T>> {
        self.cell.get_at(at)
    }

    /// Current value (tracked).
    #[track_caller]
    pub fn get(&self) -> T {
        self.value().get()
    }

    /// Current value without tracking.
    #[track_caller]
    pub fn get_untracked(&self) -> T {
        self.value().get_untracked()
    }

    /// Borrow the current value (tracked).
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.value().with(f)
    }

    /// The current value as a read-only [`Value`].
    pub fn value(&self) -> Value<T> {
        self.value
    }

    /// Jump to `v` without animating (also retargets to `v`).
    #[track_caller]
    pub fn jump_to(&self, v: T) {
        let at = Location::caller();
        let cell = self.cell(at);
        // Checked before any state changes.
        require_effectful("Animated::jump_to");
        *cell.to.borrow_mut() = comps(&v);
        cell.finish(RunEnd::Cancelled);
    }

    /// Pause this animation. Retargeting while paused keeps it paused.
    #[track_caller]
    pub fn pause(&self) {
        let at = Location::caller();
        let cell = self.cell(at);
        let rt = cell.owner.runtime();
        require_effectful("Animated::pause");
        if let AnimPhase::Running { start_ns } = cell.phase.get_untracked() {
            cell.set_phase(AnimPhase::Paused {
                start_ns,
                at_ns: rt.now_ns(),
            });
        }
    }

    /// Resume this animation without counting the paused time.
    #[track_caller]
    pub fn resume(&self) {
        let at = Location::caller();
        let cell = self.cell(at);
        let rt = cell.owner.runtime();
        require_effectful("Animated::resume");
        if let AnimPhase::Paused { start_ns, at_ns } = cell.phase.get_untracked() {
            let now = rt.now_ns();
            let reduced = MotionState::of(&rt).reduced_motion(&rt);
            {
                let mut run = cell.spring.borrow_mut();
                run.last_ns = shift(run.last_ns, at_ns, now);
            }
            cell.begin(shift(start_ns, at_ns, now), reduced);
            rt.request_turn();
        }
    }

    /// Register a setup-owned callback for natural completion of a run (and
    /// one that reduced motion completes at once). A new target, `jump_to`, a
    /// target equal to the current value (no run starts) and owner disposal do
    /// not notify; a finite repeat notifies once.
    /// `f` may return `()` or `Result<(), UiError>`; an `Err` goes to the
    /// nearest error handler.
    #[track_caller]
    pub fn on_complete<R: HandlerResult>(self, f: impl Fn() -> R + 'static) -> Self {
        let at = Location::caller();
        let rt = Owner::current("Animated::on_complete", at).runtime();
        assert_eq!(
            rt.id(),
            self.app,
            "Animated::on_complete must be registered in its App"
        );
        let cell = self.cell(at);
        let owner = cell.owner;
        let phase = cell.phase;
        drop(rt);
        owner.with_setup(|| {
            crate::reactive::watch(
                move || phase.get(),
                EffectKind::Internal,
                move |phase| {
                    if matches!(phase, AnimPhase::Completed) {
                        owner.run_handler(|| {
                            if let Err(err) = f().into_outcome().map(|_| ()) {
                                owner.report(err);
                            }
                        });
                    }
                },
            )
        });
        self
    }

    /// Whether an animation is in progress (tracked).
    #[track_caller]
    pub fn is_running(&self) -> bool {
        self.running.get()
    }
}

impl<T: Animatable> IntoValue<T> for Animated<T> {
    fn into_prop(self) -> InputProp<T> {
        crate::Prop::value(self.value())
    }
}

impl<T: Animatable> IntoReactive<T> for Animated<T> {}
