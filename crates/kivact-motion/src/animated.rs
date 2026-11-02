//! Role: `Animated<T>`: a value that follows a target `Value<T>` with a `Transition`.
//! Its current value is a graph source, so memos and effects
//! track it like any signal.

use std::cell::{Cell, RefCell};
use std::panic::Location;
use std::rc::Rc;

use super::state::{Anim, MotionState};
use super::transition::{Kind, Transition};
use super::values::Animatable;
use kivact_reactive::Prop as InputProp;
use kivact_reactive::{
    AppId, EffectKind, HandlerResult, IntoReactive, IntoValue, Owned, Owner, Runtime, Signal,
    Value, require_effectful,
};

pub(crate) struct AnimCell<T: 'static> {
    pub(crate) owner: Owner,
    pub(crate) value: Signal<T>,
    /// Whether a run is in progress (`Animated::is_running`): written only by
    /// `set_phase`.
    pub(crate) running: Signal<bool>,
    pub(crate) transition: Transition,
    pub(crate) from: RefCell<Vec<f32>>,
    pub(crate) to: RefCell<Vec<f32>>,
    pub(crate) current: RefCell<Vec<f32>>,
    pub(crate) velocity: RefCell<Vec<f32>>,
    pub(crate) phase: Cell<Phase>,
    /// Runs that ended `RunEnd::Completed` so far (`on_complete` follows it).
    pub(crate) completed: Signal<u64>,
}

/// Where an animated value stands.
#[derive(Clone, Copy)]
pub(crate) enum Phase {
    /// At rest on `to`: no run.
    Idle,
    /// A run toward `to`.
    Running(Clock),
    /// A run stopped by `Animated::pause` at `at_ns`; `resume` continues it.
    Paused { clock: Clock, at_ns: u64 },
}

/// The times of a run on the App's clock (ns).
#[derive(Clone, Copy)]
pub(crate) struct Clock {
    /// When the run started (its delay and timing count from here).
    pub(crate) start_ns: u64,
    /// When it was last stepped (a spring integrates from here).
    pub(crate) last_ns: u64,
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

/// `clock` after a stop at `paused` that went on at `now`: times up to the
/// stop move by its length, times during it move to `now`, later ones stay.
fn shift(clock: Clock, paused: u64, now: u64) -> Clock {
    let shift = |t: u64| {
        if t <= paused {
            t.saturating_add(now.saturating_sub(paused))
        } else if t <= now {
            now
        } else {
            t
        }
    };
    Clock {
        start_ns: shift(clock.start_ns),
        last_ns: shift(clock.last_ns),
    }
}

fn comps<T: Animatable>(v: &T) -> Vec<f32> {
    let mut c = Vec::with_capacity(T::COMPONENTS);
    v.to_components(&mut c);
    c
}

impl<T: Animatable> AnimCell<T> {
    fn publish(&self, c: &[f32]) {
        let v = T::from_components(c);
        *self.current.borrow_mut() = c.to_vec();
        self.value.update(move |slot| *slot = v);
    }

    fn set_phase(&self, phase: Phase) {
        self.phase.set(phase);
        self.running.set(matches!(phase, Phase::Running(_)));
    }

    fn finish(&self, end: RunEnd) {
        self.set_phase(Phase::Idle);
        let to = self.to.borrow().clone();
        self.publish(&to);
        self.velocity.borrow_mut().iter_mut().for_each(|v| *v = 0.0);
        if end == RunEnd::Completed {
            self.completed.update(|n| *n += 1);
        }
    }

    /// Run from `from` toward `to` on `clock`, or end at once: a target equal
    /// to the current value starts no run, and under reduced motion a
    /// decorative run completes at once.
    fn begin(&self, clock: Clock, reduced: bool) {
        let wire = self.transition.as_wire();
        if *self.from.borrow() == *self.to.borrow() && !matches!(wire.kind, Kind::Keyframes { .. })
        {
            self.finish(RunEnd::Cancelled);
        } else if reduced && wire.decorative {
            self.finish(RunEnd::Completed);
        } else {
            self.set_phase(Phase::Running(clock));
        }
    }

    /// Start moving from the current value to `target` at `now`.
    pub(crate) fn retarget(&self, target: &T, now: u64, reduced: bool) {
        let to = comps(target);
        let phase = self.phase.get();
        if *self.to.borrow() == to && !matches!(phase, Phase::Idle) {
            return;
        }
        *self.from.borrow_mut() = self.current.borrow().clone();
        *self.to.borrow_mut() = to;
        if self.velocity.borrow().len() != T::COMPONENTS {
            *self.velocity.borrow_mut() = vec![0.0; T::COMPONENTS];
        }
        let clock = Clock {
            start_ns: now,
            last_ns: now,
        };
        // A paused run stays paused, now toward the new target, and keeps its
        // current value.
        if let Phase::Paused { .. } = phase {
            self.set_phase(Phase::Paused { clock, at_ns: now });
            return;
        }
        self.begin(clock, reduced);
    }
}

impl<T: Animatable> Anim for AnimCell<T> {
    fn is_running(&self) -> bool {
        matches!(self.phase.get(), Phase::Running(_))
    }

    fn finish_decorative(&self) {
        if self.is_running() && self.transition.as_wire().decorative {
            self.finish(RunEnd::Completed);
        }
    }

    fn shift_clock(&self, paused: u64, now: u64) {
        match self.phase.get() {
            Phase::Idle => {}
            Phase::Running(clock) => self.set_phase(Phase::Running(shift(clock, paused, now))),
            Phase::Paused { clock, at_ns } => {
                let at_ns = if at_ns <= now {
                    at_ns.max(paused).saturating_add(now.saturating_sub(paused))
                } else {
                    at_ns
                };
                self.set_phase(Phase::Paused {
                    clock: shift(clock, paused, now),
                    at_ns,
                });
            }
        }
    }

    fn step(&self, now: u64) {
        let Phase::Running(clock) = self.phase.get() else {
            return;
        };
        let elapsed_ms = now.saturating_sub(clock.start_ns) as f32 / 1e6;
        match &self.transition.as_wire().kind {
            Kind::Spring(s) => {
                self.set_phase(Phase::Running(Clock {
                    last_ns: now,
                    ..clock
                }));
                if elapsed_ms < self.transition.as_wire().delay_ms {
                    return;
                }
                let dt = now.saturating_sub(clock.last_ns) as f32 / 1e9;
                let to = self.to.borrow().clone();
                let mut x = self.current.borrow().clone();
                let mut v = self.velocity.borrow().clone();
                if x.len() != to.len() {
                    x = to.clone();
                }
                v.resize(to.len(), 0.0);
                let settled = kivact_wire::motion::spring_step(*s, &mut x, &mut v, &to, dt);
                *self.velocity.borrow_mut() = v.clone();
                if settled {
                    self.finish(RunEnd::Completed);
                } else {
                    self.publish(&x);
                }
            }
            _ => {
                let mut values = vec![0.0; self.to.borrow().len()];
                let done = self.transition.as_wire().sample_timed(
                    &self.from.borrow(),
                    &self.to.borrow(),
                    elapsed_ms,
                    &mut values,
                );
                if done {
                    self.finish(RunEnd::Completed);
                } else {
                    self.publish(&values);
                }
            }
        }
    }
}

/// An animated value (Copy). Use it wherever a `Value<T>` is accepted.
pub struct Animated<T: 'static> {
    app: AppId,
    cell: Owned<AnimCell<T>>,
    value: Value<T>,
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
        let initial = start.unwrap_or_else(|| kivact_reactive::untrack(|| target.get()));
        let c = comps(&initial);
        let app = rt.id();
        let value: Signal<T> = owner.signal(initial, at);
        let cell = Rc::new(AnimCell {
            owner,
            value,
            running: owner.signal(false, at),
            transition,
            from: RefCell::new(c.clone()),
            to: RefCell::new(c.clone()),
            current: RefCell::new(c),
            velocity: RefCell::new(vec![0.0; T::COMPONENTS]),
            phase: Cell::new(Phase::Idle),
            completed: owner.signal(0u64, at),
        });
        let anim: Rc<dyn Anim> = cell.clone();
        MotionState::of(&rt).register_anim(Rc::downgrade(&anim));
        drop(anim);
        let stored = owner.store("Animated", cell.clone(), at);
        let weak = Rc::downgrade(&cell);
        let first_cell = Cell::new(true);
        owner.effect(
            EffectKind::Internal,
            move || {
                let Some(rt) = Runtime::get(app) else { return };
                // The target only computes a value: pure.
                let t = kivact_reactive::pure(|| target.get());
                if first_cell.replace(false) && !explicit_start {
                    return;
                }
                let Some(cell) = weak.upgrade() else { return };
                let now = rt.now_ns();
                let reduced = MotionState::of(&rt).reduced_motion();
                kivact_reactive::untrack(|| cell.retarget(&t, now, reduced));
                rt.request_turn();
            },
            at,
        );
        Animated {
            app,
            cell: stored,
            value: Value::from(value),
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
        if let Phase::Running(clock) = cell.phase.get() {
            cell.set_phase(Phase::Paused {
                clock,
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
        if let Phase::Paused { clock, at_ns } = cell.phase.get() {
            let now = rt.now_ns();
            let reduced = MotionState::of(&rt).reduced_motion();
            cell.begin(shift(clock, at_ns, now), reduced);
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
        let completed = cell.completed;
        let seen = Cell::new(completed.get_untracked());
        drop(rt);
        owner.effect(
            EffectKind::Internal,
            move || {
                let n = completed.get();
                if seen.replace(n) == n {
                    return;
                }
                owner.run_handler(|| {
                    if let Err(err) = f().into_result() {
                        owner.report(err);
                    }
                });
            },
            at,
        );
        self
    }

    /// Whether an animation is in progress (tracked).
    #[track_caller]
    pub fn is_running(&self) -> bool {
        self.cell(Location::caller()).running.get()
    }
}

impl<T: Animatable> IntoValue<T> for Animated<T> {
    fn into_prop(self) -> InputProp<T> {
        kivact_reactive::Prop::value(self.value())
    }
}

impl<T: Animatable> IntoReactive<T> for Animated<T> {}
