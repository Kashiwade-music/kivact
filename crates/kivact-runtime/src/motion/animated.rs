//! Role: `Animated<T>`: a value that follows a target `Value<T>` with a `Transition`.
//! Its current value is a graph source, so memos and effects
//! track it like any signal.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use super::Repeat;
use super::state::{Anim, MotionState};
use super::transition::{Kind, Transition, settles_at_start};
use super::values::Animatable;
use crate::Prop as InputProp;
use crate::{
    AppId, EffectKind, ErrorKind, IntoReactive, IntoValue, Owned, Owner, Runtime, Signal, UiError,
    Value, require_effectful,
};
use kivact_types::motion::{PlayDirection, SpringRun};

/// One run: fixed endpoints and timing, and the spring's integration state.
pub(crate) struct Run {
    pub(crate) from: Vec<f64>,
    pub(crate) to: Vec<f64>,
    pub(crate) timing: Transition,
    pub(crate) spring: RefCell<SpringRun>,
}

pub(crate) struct AnimCell<T: 'static> {
    pub(crate) owner: Owner,
    pub(crate) value: Signal<T>,
    pub(crate) transition: Transition,
    /// The value now (always valid; published to `value`).
    pub(crate) current: RefCell<Vec<f64>>,
    pub(crate) phase: Signal<AnimPhase>,
}

/// Where an animated value stands; the run it moves by is in its payload.
#[derive(Clone)]
pub(crate) enum AnimPhase {
    /// Not running; the target is `current`.
    Idle,
    /// The last run completed. `target` is the value last given to `set`;
    /// `current` may differ from it (alternate with an even count).
    Completed { target: Vec<f64> },
    /// A run toward its target that started at `start_ns` on the App's clock.
    Running { start_ns: u64, run: Rc<Run> },
    /// A run stopped by `Animated::pause` at `at_ns`; `resume` continues it.
    Paused {
        start_ns: u64,
        at_ns: u64,
        run: Rc<Run>,
    },
}

impl PartialEq for AnimPhase {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Idle, Self::Idle) => true,
            (Self::Completed { target: a }, Self::Completed { target: b }) => a == b,
            (
                Self::Running {
                    start_ns: a,
                    run: ra,
                },
                Self::Running {
                    start_ns: b,
                    run: rb,
                },
            ) => a == b && Rc::ptr_eq(ra, rb),
            (
                Self::Paused {
                    start_ns: a,
                    at_ns: ta,
                    run: ra,
                },
                Self::Paused {
                    start_ns: b,
                    at_ns: tb,
                    run: rb,
                },
            ) => a == b && ta == tb && Rc::ptr_eq(ra, rb),
            _ => false,
        }
    }
}

impl Run {
    /// Move the spring's last step time with a stop of the frame clock.
    pub(crate) fn shift_last(&self, paused: u64, now: u64) {
        let mut spring = self.spring.borrow_mut();
        spring.last_ns = shift(spring.last_ns, paused, now);
    }
}

/// The time `t` after a stop at `paused` that went on at `now`: times up to
/// the stop move by its length, times during it move to `now`, later ones
/// stay.
pub(super) fn shift(t: u64, paused: u64, now: u64) -> u64 {
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

/// The value a finished run settles on: the end of its last iteration (§12.3).
fn settled(run: &Run) -> Vec<f64> {
    if settles_at_start(&run.timing) {
        run.from.clone()
    } else {
        run.to.clone()
    }
}

/// A copy of `transition` for a run that moves once, forward, without delay.
fn one_shot(transition: &Transition) -> Transition {
    Transition {
        repeat: Repeat::Times(1),
        direction: PlayDirection::Normal,
        delay_ms: 0.0,
        ..transition.clone()
    }
}

impl<T: Animatable> AnimCell<T> {
    fn publish(&self, c: &[f64]) {
        let v = T::from_components(c);
        *self.current.borrow_mut() = c.to_vec();
        self.value.update(move |slot| *slot = v);
    }

    /// The target: the value `set` last gave (the current value when idle).
    fn target(&self) -> Vec<f64> {
        match self.phase.get_untracked() {
            AnimPhase::Idle => self.current.borrow().clone(),
            AnimPhase::Completed { target } => target,
            AnimPhase::Running { run, .. } | AnimPhase::Paused { run, .. } => run.to.clone(),
        }
    }

    /// Stop with no run, the current value being `value`.
    fn settle(&self, value: Vec<f64>) {
        self.publish(&value);
        self.phase.set(AnimPhase::Idle);
    }

    /// End `run` at its settled value and report the completion.
    fn complete(&self, run: &Run) {
        self.publish(&settled(run));
        let target = run.to.clone();
        self.phase
            .update(|phase| *phase = AnimPhase::Completed { target });
    }

    /// Start `run` at `start_ns`, or complete it at once under reduced motion.
    fn start_run(&self, run: Rc<Run>, start_ns: u64, reduced: bool) {
        if reduced && run.timing.decorative {
            self.complete(&run);
        } else {
            self.phase.set(AnimPhase::Running { start_ns, run });
        }
    }

    /// Begin `run`, or end it at once when nothing moves: a target equal to
    /// the current value starts no run (a keyframes run always plays).
    fn begin(&self, run: Rc<Run>, start_ns: u64, reduced: bool) {
        if run.from == run.to && !matches!(run.timing.kind, Kind::Keyframes { .. }) {
            self.settle(run.to.clone());
        } else {
            self.start_run(run, start_ns, reduced);
        }
    }

    /// A run of `timing` from the current value toward `to`, keeping `velocity`.
    fn make_run(&self, now: u64, timing: Transition, to: Vec<f64>, velocity: Vec<f64>) -> Rc<Run> {
        let from = self.current.borrow().clone();
        let mut spring = SpringRun {
            velocity,
            ..SpringRun::default()
        };
        spring.start(now, &timing, &from, &to);
        Rc::new(Run {
            from,
            to,
            timing,
            spring: RefCell::new(spring),
        })
    }

    /// Start moving from the current value to `target` at `now`.
    pub(crate) fn retarget(&self, target: &T, now: u64, reduced: bool) {
        let to = comps(target);
        let phase = self.phase.get_untracked();
        if !matches!(phase, AnimPhase::Idle) && self.target() == to {
            return;
        }
        let (timing, velocity) = match &phase {
            AnimPhase::Running { run, .. } | AnimPhase::Paused { run, .. } => {
                let timing =
                    self.transition
                        .retargeted(&run.from, &run.to, &self.current.borrow(), &to);
                (timing, run.spring.borrow().velocity.clone())
            }
            AnimPhase::Idle | AnimPhase::Completed { .. } => {
                (self.transition.clone(), vec![0.0; T::COMPONENTS])
            }
        };
        let run = self.make_run(now, timing, to, velocity);
        // A paused run stays paused, now toward the new target, and keeps its
        // current value.
        if let AnimPhase::Paused { .. } = phase {
            self.phase.set(AnimPhase::Paused {
                start_ns: now,
                at_ns: now,
                run,
            });
            return;
        }
        self.begin(run, now, reduced);
    }
}

impl<T: Animatable> Anim for AnimCell<T> {
    fn is_running(&self) -> bool {
        matches!(self.phase.get_untracked(), AnimPhase::Running { .. })
    }

    fn finish_decorative(&self) {
        if let AnimPhase::Running { run, .. } = self.phase.get_untracked()
            && self.transition.decorative
        {
            self.complete(&run);
        }
    }

    fn shift_clock(&self, paused: u64, now: u64) {
        match self.phase.get_untracked() {
            AnimPhase::Idle | AnimPhase::Completed { .. } => {}
            AnimPhase::Running { start_ns, run } => {
                run.shift_last(paused, now);
                self.phase.set(AnimPhase::Running {
                    start_ns: shift(start_ns, paused, now),
                    run,
                });
            }
            AnimPhase::Paused {
                start_ns,
                at_ns,
                run,
            } => {
                let at_ns = shift(at_ns, paused, now);
                run.shift_last(paused, now);
                self.phase.set(AnimPhase::Paused {
                    start_ns: shift(start_ns, paused, now),
                    at_ns,
                    run,
                });
            }
        }
    }

    fn step(&self, now: u64) {
        let AnimPhase::Running { start_ns, run } = self.phase.get_untracked() else {
            return;
        };
        let mut current = self.current.borrow().clone();
        let done = {
            let mut spring = run.spring.borrow_mut();
            run.timing
                .step(&run.from, &run.to, start_ns, now, &mut current, &mut spring)
        };
        if done {
            self.complete(&run);
        } else {
            self.publish(&current);
        }
    }
}

/// An animated value (Copy). Use it wherever a `Value<T>` is accepted.
pub struct Animated<T: 'static> {
    app: AppId,
    cell: Owned<Rc<AnimCell<T>>>,
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
        if let Kind::Spring(spring) = &transition.kind
            && let Some(problem) = spring.problem()
        {
            owner.report(UiError::new(
                ErrorKind::Message,
                format!("spring: {problem}"),
                Some(at),
            ));
        }
        let value: Signal<T> = owner.signal(initial, at);
        let cell = Rc::new(AnimCell {
            owner,
            value,
            transition,
            current: RefCell::new(c),
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
        cell.settle(comps(&v));
    }

    /// Pause this animation. Retargeting while paused keeps it paused.
    #[track_caller]
    pub fn pause(&self) {
        let at = Location::caller();
        let cell = self.cell(at);
        let rt = cell.owner.runtime();
        require_effectful("Animated::pause");
        if let AnimPhase::Running { start_ns, run } = cell.phase.get_untracked() {
            cell.phase.set(AnimPhase::Paused {
                start_ns,
                at_ns: rt.now_ns(),
                run,
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
        if let AnimPhase::Paused {
            start_ns,
            at_ns,
            run,
        } = cell.phase.get_untracked()
        {
            let now = rt.now_ns();
            let reduced = MotionState::of(&rt).reduced_motion(&rt);
            run.shift_last(at_ns, now);
            cell.begin(run, shift(start_ns, at_ns, now), reduced);
            rt.request_turn();
        }
    }

    /// Give the spring a velocity (units per second, per component) now: a
    /// running or paused spring keeps its target and goes on with the
    /// velocity; an idle or completed one starts a run from its current value
    /// toward its target (for a completed alternate run, the value last given
    /// to `set`), once, forward and without delay. Not a spring transition:
    /// reported (`ErrorKind::Message`) and ignored.
    #[track_caller]
    pub fn set_velocity(&self, velocity: T) {
        let at = Location::caller();
        let cell = self.cell(at);
        let rt = cell.owner.runtime();
        // Checked before any state changes.
        require_effectful("Animated::set_velocity");
        if !matches!(cell.transition.kind, Kind::Spring(_)) {
            cell.owner.report(UiError::new(
                ErrorKind::Message,
                "set_velocity needs a spring transition",
                Some(at),
            ));
            return;
        }
        let v = comps(&velocity);
        let now = rt.now_ns();
        match cell.phase.get_untracked() {
            AnimPhase::Running { run, .. } | AnimPhase::Paused { run, .. } => {
                run.spring.borrow_mut().velocity = v;
            }
            AnimPhase::Idle | AnimPhase::Completed { .. } => {
                let target = cell.target();
                let run = cell.make_run(now, one_shot(&cell.transition), target, v);
                let reduced = MotionState::of(&rt).reduced_motion(&rt);
                cell.start_run(run, now, reduced);
            }
        }
        rt.request_turn();
    }

    /// Register a setup-owned callback for natural completion of a run (and
    /// one that reduced motion completes at once). A new target, `jump_to`, a
    /// target equal to the current value (no run starts) and owner disposal do
    /// not notify; a finite repeat notifies once.
    /// `f` may return `()` or `Result<(), UiError>`; an `Err` goes to the
    /// nearest error handler.
    #[track_caller]
    pub fn on_complete(self, f: impl Into<crate::Callback<()>>) -> Self {
        let f = f.into();
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
                    if matches!(phase, AnimPhase::Completed { .. }) {
                        owner.run_handler(|| {
                            if let Err(err) = f.try_call(()) {
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
