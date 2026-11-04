//! Role: Easing curves, spring parameters and iteration counts of animations.

/// Where a discrete easing step jumps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPosition {
    Start,
    End,
}

/// How many times a transition plays (CSS `animation-iteration-count`, Framer
/// Motion's `repeat`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Repeat {
    /// This many iterations; `Times(0)` ends at once on the target.
    Times(u32),
    /// Until the target changes.
    Forever,
}

/// Mapping from normalized elapsed time to animation progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// CSS-style cubic bezier (x1, y1, x2, y2).
    CubicBezier(f32, f32, f32, f32),
    /// Discrete steps. A zero count is treated as one.
    Steps {
        count: u32,
        position: StepPosition,
    },
}

impl Easing {
    /// `count` discrete steps that jump at the start of each step.
    pub fn steps_start(count: u32) -> Self {
        Easing::Steps {
            count,
            position: StepPosition::Start,
        }
    }

    /// `count` discrete steps that jump at the end of each step.
    pub fn steps_end(count: u32) -> Self {
        Easing::Steps {
            count,
            position: StepPosition::End,
        }
    }

    /// Evaluate progress after clamping normalized time to `0..=1`.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::CubicBezier(x1, y1, x2, y2) => bezier(t, x1, y1, x2, y2),
            Easing::Steps { count, position } => {
                let n = count.max(1) as f32;
                match position {
                    StepPosition::Start => ((t * n).floor() + 1.0).min(n) / n,
                    StepPosition::End => (t * n).floor() / n,
                }
            }
        }
    }
}

fn bezier(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let curve = |t: f32, a: f32, b: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    };
    // Solve x(t) = x by bisection (monotonic for valid curves).
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..30 {
        let mid = (lo + hi) / 2.0;
        if curve(mid, x1, x2) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    curve((lo + hi) / 2.0, y1, y2)
}

/// Spring parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Spring force per unit displacement.
    pub stiffness: f32,
    /// Resistance proportional to velocity.
    pub damping: f32,
    /// Mass used when integrating spring acceleration.
    pub mass: f32,
}

impl Spring {
    pub const GENTLE: Spring = Spring {
        stiffness: 120.0,
        damping: 14.0,
        mass: 1.0,
    };
    pub const WOBBLY: Spring = Spring {
        stiffness: 180.0,
        damping: 12.0,
        mass: 1.0,
    };
    pub const STIFF: Spring = Spring {
        stiffness: 210.0,
        damping: 20.0,
        mass: 1.0,
    };
    pub const DEFAULT: Spring = Spring {
        stiffness: 170.0,
        damping: 26.0,
        mass: 1.0,
    };
}

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
