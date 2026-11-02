//! Role: Transitions, keyframes and the motion clock of the engine and client.
//!
//! Transitions: tween, spring and keyframes, with delay, repeat and the
//! decorative flag used by reduced motion, and the engine's motion clock
//! each frame tick carries. Easing curves, springs and iteration counts are
//! `kivact-types` vocabulary, re-exported here. [`Transition::step`] is the
//! one stepper; a spring repeats like a tween (an iteration ends when it
//! settles).

pub use kivact_types::{Easing, Repeat, Spring, StepPosition};

/// The engine's motion clock as of one frame tick: what the client's
/// frame-driven work (frame loops and animations) needs to know about the
/// frame clock standing still, in one value the engine owns. A tick carries
/// a snapshot, so an old tick never reads a newer clock.
///
/// No tick is sent while nothing can be drawn (`HostEnv::surface_active` is
/// false), so a stop shows in the first tick after it as a larger
/// `offset_ns`, not as a flag. Reduced motion is not part of it: the client
/// applies that in the turn that reports it (`HostEnv::reduced_motion`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MotionClock {
    /// The total host time (ns) the frame clock has stood still, summed
    /// over every stop (no surface) that has ended. Animations go on from
    /// where they stood: the time without frames does not count, so a
    /// transition does not finish because the window was hidden.
    pub offset_ns: u64,
    /// The host time (ns) of the last stop's end. A run that began during
    /// that stop begins here. With several stops between two ticks only
    /// the last end is known.
    pub resumed_ns: u64,
}

/// Direction of repeated iterations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayDirection {
    Normal,
    Alternate,
}

/// Interpolation model and parameters of a transition.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Tween {
        ms: f32,
        easing: Easing,
    },
    Spring(Spring),
    /// (offset 0..=1, components)
    Keyframes {
        frames: Vec<(f32, Vec<f32>)>,
        ms: f32,
    },
}

/// How a value moves to a new target.
#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    /// Interpolation model for each iteration.
    pub kind: Kind,
    /// Delay before the first iteration, in milliseconds.
    pub delay_ms: f32,
    /// How many iterations to run.
    pub repeat: Repeat,
    /// Whether successive iterations reverse direction.
    pub direction: PlayDirection,
    /// Allow the host to suppress this transition for reduced motion.
    pub decorative: bool,
}

/// Copy `values` into `out`, component by component.
fn put(out: &mut [f32], values: &[f32]) {
    for (out, value) in out.iter_mut().zip(values) {
        *out = *value;
    }
}

/// Write the components `t` of the way from `a` to `b` into `out`.
pub fn lerp(a: &[f32], b: &[f32], t: f32, out: &mut [f32]) {
    for ((out, x), y) in out.iter_mut().zip(a).zip(b) {
        *out = x + (y - x) * t;
    }
}

/// What a run keeps between its steps besides its endpoints and start time: a
/// spring's velocity, the host time of the previous step and the iteration a
/// spring plays. A timed run (tween, keyframes) reads none of it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpringRun {
    /// Velocity of each component.
    pub velocity: Vec<f32>,
    /// Host time (ns) of the previous step.
    pub last_ns: u64,
    /// The iteration a spring plays, from 0.
    pub iteration: u32,
}

impl SpringRun {
    /// Start a new run at `now_ns`, keeping the velocity (an interrupted
    /// spring goes on with it).
    pub fn restart(&mut self, now_ns: u64) {
        self.last_ns = now_ns;
        self.iteration = 0;
    }
}

impl Transition {
    /// Step a run from `from` to `to` that began at `start_ns` to `now_ns`:
    /// write its value into `current` (the value of the previous step) and
    /// return whether it is done (then `current` is `to`). The one stepper of
    /// client animations and host state transitions.
    pub fn step(
        &self,
        from: &[f32],
        to: &[f32],
        start_ns: u64,
        now_ns: u64,
        current: &mut Vec<f32>,
        run: &mut SpringRun,
    ) -> bool {
        if current.len() != to.len() {
            current.clear();
            current.extend_from_slice(to);
        }
        let elapsed_ms = now_ns.saturating_sub(start_ns) as f32 / 1e6;
        let Kind::Spring(spring) = &self.kind else {
            return self.sample_timed(from, to, elapsed_ms, current);
        };
        if elapsed_ms < self.delay_ms {
            run.last_ns = now_ns;
            return false;
        }
        if self.repeat == Repeat::Times(0) {
            current.copy_from_slice(to);
            return true;
        }
        let dt = now_ns.saturating_sub(run.last_ns) as f32 / 1e9;
        run.last_ns = now_ns;
        run.velocity.resize(to.len(), 0.0);
        let paired = from.len() == to.len();
        let backwards = self.direction == PlayDirection::Alternate && run.iteration % 2 == 1;
        let target = if backwards && paired { from } else { to };
        if !spring_step(*spring, current, &mut run.velocity, target, dt) {
            return false;
        }
        run.velocity.fill(0.0);
        let next = run.iteration.saturating_add(1);
        let more = match self.repeat {
            Repeat::Times(n) => next < n,
            Repeat::Forever => true,
        };
        if !more {
            current.copy_from_slice(to);
            return true;
        }
        run.iteration = next;
        if self.direction == PlayDirection::Normal && paired {
            current.copy_from_slice(from);
        }
        false
    }

    /// Sample a time-based (tween/keyframes) transition at `elapsed_ms`
    /// into `out` (as long as `to`); returns whether it is done. Nothing is
    /// allocated.
    fn sample_timed(&self, from: &[f32], to: &[f32], elapsed_ms: f32, out: &mut [f32]) -> bool {
        let t_ms = elapsed_ms - self.delay_ms;
        if t_ms < 0.0 {
            put(out, from);
            return false;
        }
        let dur = match &self.kind {
            Kind::Tween { ms, .. } | Kind::Keyframes { ms, .. } => *ms,
            Kind::Spring(_) => 0.0,
        };
        if dur <= 0.0 {
            put(out, to);
            return true;
        }
        let iter = (t_ms / dur).floor();
        let total = match self.repeat {
            Repeat::Times(n) => n as f32,
            Repeat::Forever => f32::INFINITY,
        };
        if iter >= total {
            put(out, to);
            return true;
        }
        let mut p = (t_ms - iter * dur) / dur;
        if self.direction == PlayDirection::Alternate && (iter as u64) % 2 == 1 {
            p = 1.0 - p;
        }
        match &self.kind {
            Kind::Tween { easing, .. } => lerp(from, to, easing.apply(p), out),
            Kind::Keyframes { frames, .. } => keyframe_at(frames, p, to, out),
            Kind::Spring(_) => put(out, to),
        }
        false
    }
}

/// The keyframes' value at `p` (those as long as `fallback`; `fallback`
/// without any) into `out`.
fn keyframe_at(frames: &[(f32, Vec<f32>)], p: f32, fallback: &[f32], out: &mut [f32]) {
    let mut valid = frames.iter().filter(|f| f.1.len() == fallback.len());
    let Some(first) = valid.next() else {
        put(out, fallback);
        return;
    };
    if p <= first.0 {
        put(out, &first.1);
        return;
    }
    let mut a = first;
    for b in valid {
        if p <= b.0 {
            let span = (b.0 - a.0).max(1e-6);
            lerp(&a.1, &b.1, (p - a.0) / span, out);
            return;
        }
        a = b;
    }
    put(out, &a.1);
}

/// Shared semi-implicit spring integration; returns whether it settled.
fn spring_step(s: Spring, x: &mut [f32], v: &mut [f32], to: &[f32], mut dt: f32) -> bool {
    let m = s.mass.max(1e-3);
    while dt > 0.0 {
        let h = dt.min(0.001);
        for ((x, v), to) in x.iter_mut().zip(v.iter_mut()).zip(to) {
            let acceleration = (-s.stiffness * (*x - to) - s.damping * *v) / m;
            *v += acceleration * h;
            *x += *v * h;
        }
        dt -= h;
    }
    x.iter().zip(to).all(|(a, b)| (a - b).abs() < 1e-3) && v.iter().all(|x| x.abs() < 1e-2)
}
