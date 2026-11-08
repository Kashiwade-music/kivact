//! Role: Animation timing, transitions, easing and spring stepping.

use std::sync::Arc;

/// Where a stepped easing jumps (CSS `steps()` jump terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StepPosition {
    /// Jump at the start of each interval; the first output is above zero.
    JumpStart,
    /// Jump at the end of each interval; the output stays below one until the end.
    JumpEnd,
    /// No jump at either end: `count - 1` jumps, holding 0 first and 1 last.
    JumpNone,
    /// Jump at both ends: `count + 1` jumps.
    JumpBoth,
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

/// Which property a style transition animates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransitionProp {
    /// Every paint property without its own entry (CSS `transition: all`).
    All,
    /// Opacity.
    Opacity,
    /// The transform.
    Transform,
    /// The background fill.
    Background,
    /// The border color.
    BorderColor,
    /// The border width.
    BorderWidth,
    /// The corner radius.
    Radius,
    /// The text color.
    TextColor,
    /// The shadow.
    Shadow,
    /// The filter chain.
    Filter,
    /// The backdrop filter chain.
    BackdropFilter,
    /// Presence size (enter_size / exit_size), as before.
    Size,
}

/// Mapping from normalized elapsed time to animation progress.
#[derive(Clone, Debug, PartialEq)]
pub enum Easing {
    /// Progress equals time.
    Linear,
    /// CSS ease = cubic-bezier(0.25, 0.1, 0.25, 1).
    Ease,
    /// cubic-bezier(0.42, 0, 1, 1).
    EaseIn,
    /// cubic-bezier(0, 0, 0.58, 1).
    EaseOut,
    /// cubic-bezier(0.42, 0, 0.58, 1).
    EaseInOut,
    /// CSS-style cubic bezier (x1, y1, x2, y2).
    CubicBezier(f64, f64, f64, f64),
    /// Discrete steps (CSS `steps()`). A zero count is treated as one.
    Steps {
        /// Number of intervals.
        count: u32,
        /// Where the jumps happen.
        position: StepPosition,
    },
    /// CSS linear(): (input, output) points with increasing input.
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; one allocation is shared by all clones.
    LinearPoints(Arc<[(f64, f64)]>),
    /// Pulls back before moving forward.
    BackIn,
    /// Overshoots the end, then settles back.
    BackOut,
    /// Pulls back at the start and overshoots at the end.
    BackInOut,
    /// Oscillates with growing amplitude toward the end.
    ElasticIn,
    /// Oscillates around the end with shrinking amplitude.
    ElasticOut,
    /// Elastic at both ends.
    ElasticInOut,
    /// Bounces away from the start.
    BounceIn,
    /// Bounces against the end.
    BounceOut,
    /// Bounces at both ends.
    BounceInOut,
}

const BACK_C1: f64 = 1.70158;
const BACK_C3: f64 = BACK_C1 + 1.0;
const ELASTIC_C4: f64 = std::f64::consts::TAU / 3.0;
const ELASTIC_C5: f64 = std::f64::consts::TAU / 4.5;
const BOUNCE_N1: f64 = 7.5625;
const BOUNCE_D1: f64 = 2.75;

impl Easing {
    /// `count` discrete steps that jump at `position`.
    ///
    /// ```
    /// use kivact_types::{Easing, StepPosition};
    /// assert_eq!(Easing::steps(4, StepPosition::JumpEnd).apply(0.5), 0.5);
    /// assert_eq!(Easing::steps(4, StepPosition::JumpStart).apply(0.5), 0.75);
    /// ```
    pub fn steps(count: u32, position: StepPosition) -> Easing {
        Easing::Steps { count, position }
    }

    /// Points with input `None` are spread evenly between their neighbours (CSS
    /// `linear()`). A missing first input is 0 and a missing last input is 1;
    /// an input below an earlier one is raised to it.
    ///
    /// ```
    /// use kivact_types::Easing;
    /// let e = Easing::linear_points([(None, 0.0), (None, 0.5), (Some(1.0), 1.0)]);
    /// assert_eq!(e.apply(0.5), 0.5);
    /// assert_eq!(e.apply(0.25), 0.25);
    /// ```
    pub fn linear_points(points: impl IntoIterator<Item = (Option<f64>, f64)>) -> Easing {
        let points: Vec<(Option<f64>, f64)> = points.into_iter().collect();
        let last = points.len().saturating_sub(1);
        let known = |i: usize| match points[i].0 {
            Some(x) => Some(x),
            None if i == 0 => Some(0.0),
            None if i == last => Some(1.0),
            None => None,
        };
        let mut inputs = vec![0.0; points.len()];
        let mut anchor: Option<(usize, f64)> = None;
        for i in 0..points.len() {
            let Some(x) = known(i) else { continue };
            let (from, a) = anchor.unwrap_or((i, x));
            for (k, input) in inputs.iter_mut().enumerate().take(i + 1).skip(from) {
                *input = if i == from {
                    x
                } else {
                    a + (x - a) * (k - from) as f64 / (i - from) as f64
                };
            }
            anchor = Some((i, x));
        }
        let mut floor = f64::NEG_INFINITY;
        let resolved: Vec<(f64, f64)> = inputs
            .iter()
            .zip(&points)
            .map(|(&x, &(_, y))| {
                floor = floor.max(x);
                (floor, y)
            })
            .collect();
        Easing::LinearPoints(resolved.into())
    }

    /// Output for progress `t`. `t` is clamped to `0..=1`; the output may leave
    /// `0..=1` where the curve overshoots (back, elastic, cubic-bezier).
    ///
    /// ```
    /// use kivact_types::Easing;
    /// assert_eq!(Easing::Linear.apply(2.0), 1.0);
    /// assert_eq!(Easing::BounceOut.apply(1.0), 1.0);
    /// assert!(Easing::BackOut.apply(0.8) > 1.0);
    /// ```
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::Ease => bezier(t, 0.25, 0.1, 0.25, 1.0),
            Easing::EaseIn => bezier(t, 0.42, 0.0, 1.0, 1.0),
            Easing::EaseOut => bezier(t, 0.0, 0.0, 0.58, 1.0),
            Easing::EaseInOut => bezier(t, 0.42, 0.0, 0.58, 1.0),
            Easing::CubicBezier(x1, y1, x2, y2) => bezier(t, *x1, *y1, *x2, *y2),
            Easing::Steps { count, position } => step_output(*count, *position, t),
            Easing::LinearPoints(points) => points_output(points, t),
            Easing::BackIn => BACK_C3 * t * t * t - BACK_C1 * t * t,
            Easing::BackOut => back_out(t),
            Easing::BackInOut => back_in_out(t),
            Easing::ElasticIn => elastic(t, elastic_in),
            Easing::ElasticOut => elastic(t, elastic_out),
            Easing::ElasticInOut => elastic(t, elastic_in_out),
            Easing::BounceIn => 1.0 - bounce_out(1.0 - t),
            Easing::BounceOut => bounce_out(t),
            Easing::BounceInOut if t < 0.5 => (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0,
            Easing::BounceInOut => (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0,
        }
    }
}

fn bezier(x: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    // Preserve exact endpoints instead of the bisection's residual progress.
    if x == 0.0 || x == 1.0 {
        return x;
    }
    let curve = |t: f64, a: f64, b: f64| {
        let u = 1.0 - t;
        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    };
    // Solve x(t) = x by bisection (monotonic for valid curves).
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
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

fn back_out(t: f64) -> f64 {
    let u = t - 1.0;
    1.0 + BACK_C3 * u * u * u + BACK_C1 * u * u
}

fn back_in_out(t: f64) -> f64 {
    let c2 = BACK_C1 * 1.525;
    if t < 0.5 {
        let u = 2.0 * t;
        u * u * ((c2 + 1.0) * u - c2) / 2.0
    } else {
        let u = 2.0 * t - 2.0;
        (u * u * ((c2 + 1.0) * u + c2) + 2.0) / 2.0
    }
}

/// `curve` strictly between the exact endpoints 0 and 1 of an elastic easing.
fn elastic(t: f64, curve: fn(f64) -> f64) -> f64 {
    if t == 0.0 || t == 1.0 { t } else { curve(t) }
}

fn elastic_in(t: f64) -> f64 {
    -(10.0 * t - 10.0).exp2() * ((10.0 * t - 10.75) * ELASTIC_C4).sin()
}

fn elastic_out(t: f64) -> f64 {
    (-10.0 * t).exp2() * ((10.0 * t - 0.75) * ELASTIC_C4).sin() + 1.0
}

fn elastic_in_out(t: f64) -> f64 {
    let wave = ((20.0 * t - 11.125) * ELASTIC_C5).sin();
    if t < 0.5 {
        -(20.0 * t - 10.0).exp2() * wave / 2.0
    } else {
        (-20.0 * t + 10.0).exp2() * wave / 2.0 + 1.0
    }
}

fn bounce_out(t: f64) -> f64 {
    if t < 1.0 / BOUNCE_D1 {
        BOUNCE_N1 * t * t
    } else if t < 2.0 / BOUNCE_D1 {
        let u = t - 1.5 / BOUNCE_D1;
        BOUNCE_N1 * u * u + 0.75
    } else if t < 2.5 / BOUNCE_D1 {
        let u = t - 2.25 / BOUNCE_D1;
        BOUNCE_N1 * u * u + 0.9375
    } else {
        let u = t - 2.625 / BOUNCE_D1;
        BOUNCE_N1 * u * u + 0.984375
    }
}

/// CSS `steps()` output for `t` in `0..=1`.
fn step_output(count: u32, position: StepPosition, t: f64) -> f64 {
    let n = match position {
        StepPosition::JumpNone => count.max(2),
        _ => count.max(1),
    } as f64;
    let (bump, jumps) = match position {
        StepPosition::JumpStart => (1.0, n),
        StepPosition::JumpEnd => (0.0, n),
        StepPosition::JumpNone => (0.0, n - 1.0),
        StepPosition::JumpBoth => (1.0, n + 1.0),
    };
    (((t * n).floor() + bump) / jumps).clamp(0.0, 1.0)
}

/// CSS `linear()` output for `t` over resolved, non-decreasing points.
fn points_output(points: &[(f64, f64)], t: f64) -> f64 {
    let after = points.partition_point(|p| p.0 <= t);
    let Some(before) = after.checked_sub(1) else {
        return points.first().map_or(t, |p| p.1);
    };
    let prev = points[before];
    match points.get(after) {
        Some(next) => prev.1 + (next.1 - prev.1) * (t - prev.0) / (next.0 - prev.0),
        None => prev.1,
    }
}

/// Default relative threshold below which a spring counts as at rest.
const DEFAULT_REST: f64 = 1e-3;
/// Band around critical damping (zeta = 1) that uses the critical solution.
const CRITICAL_BAND: f64 = 1e-6;
/// Smallest run amplitude; a component that does not move rests at once.
const AMPLITUDE_MIN: f64 = 1e-9;
/// Largest run amplitude, which bounds the final jump when a run settles.
const AMPLITUDE_MAX: f64 = 500.0;

/// Spring parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Spring force per unit displacement.
    pub stiffness: f64,
    /// Resistance proportional to velocity.
    pub damping: f64,
    /// Mass used when integrating spring acceleration.
    pub mass: f64,
    /// Settled when, for every component, the distance to the target and the
    /// velocity over the natural frequency are within `rest` times the run's
    /// amplitude.
    pub rest: f64,
}

impl Spring {
    /// A soft spring with little overshoot.
    pub const GENTLE: Spring = Spring {
        stiffness: 120.0,
        damping: 14.0,
        mass: 1.0,
        rest: DEFAULT_REST,
    };
    /// A loose spring that overshoots and wobbles.
    pub const WOBBLY: Spring = Spring {
        stiffness: 180.0,
        damping: 12.0,
        mass: 1.0,
        rest: DEFAULT_REST,
    };
    /// A firm spring that settles quickly.
    pub const STIFF: Spring = Spring {
        stiffness: 210.0,
        damping: 20.0,
        mass: 1.0,
        rest: DEFAULT_REST,
    };
    /// The spring used when none is chosen.
    pub const DEFAULT: Spring = Spring {
        stiffness: 170.0,
        damping: 26.0,
        mass: 1.0,
        rest: DEFAULT_REST,
    };

    /// A spring that settles in `duration_ms` milliseconds with the given
    /// bounce (`-1..1`, clamped to `-0.99..=0.99`): 0 is critically damped,
    /// positive bounces, negative is over-damped. The duration holds for runs
    /// of amplitude up to 500. A duration that is not positive and finite, or a
    /// bounce that is not finite, gives a spring whose [`Spring::problem`] is
    /// set.
    ///
    /// ```
    /// use kivact_types::Spring;
    /// let s = Spring::with_duration(500.0, 0.2);
    /// assert!(s.problem().is_none());
    /// assert_eq!(s.mass, 1.0);
    /// assert!(Spring::with_duration(0.0, 0.0).problem().is_some());
    /// ```
    pub fn with_duration(duration_ms: f64, bounce: f64) -> Spring {
        let valid = duration_ms.is_finite() && duration_ms > 0.0 && bounce.is_finite();
        if !valid {
            return Spring {
                stiffness: f64::NAN,
                damping: f64::NAN,
                mass: 1.0,
                rest: DEFAULT_REST,
            };
        }
        let bounce = bounce.clamp(-0.99, 0.99);
        let zeta = if bounce >= 0.0 {
            1.0 - bounce
        } else {
            1.0 / (1.0 + bounce)
        };
        let omega = settle_time(zeta, DEFAULT_REST) / (duration_ms / 1000.0);
        Spring {
            stiffness: omega * omega,
            damping: 2.0 * zeta * omega,
            mass: 1.0,
            rest: DEFAULT_REST,
        }
    }

    /// This spring with another rest threshold.
    pub fn rest(self, rest: f64) -> Spring {
        Spring { rest, ..self }
    }

    /// Why this spring cannot run (stiffness <= 0, damping < 0, mass <= 0,
    /// rest <= 0, or a value not finite), if so. Callers report it.
    ///
    /// ```
    /// use kivact_types::Spring;
    /// assert!(Spring::DEFAULT.problem().is_none());
    /// assert!(Spring::DEFAULT.rest(0.0).problem().is_some());
    /// ```
    pub fn problem(&self) -> Option<&'static str> {
        let all = [self.stiffness, self.damping, self.mass, self.rest];
        if all.iter().any(|v| !v.is_finite()) {
            return Some("spring values must be finite");
        }
        if self.stiffness <= 0.0 {
            return Some("spring stiffness must be positive");
        }
        if self.damping < 0.0 {
            return Some("spring damping must not be negative");
        }
        if self.mass <= 0.0 {
            return Some("spring mass must be positive");
        }
        if self.rest <= 0.0 {
            return Some("spring rest must be positive");
        }
        None
    }
}

/// Position and velocity after `t` seconds of a damped oscillator with natural
/// frequency `w0` and damping ratio `zeta`, from offset `x0` and velocity `v0`.
fn oscillate(w0: f64, zeta: f64, x0: f64, v0: f64, t: f64) -> (f64, f64) {
    if zeta < 1.0 - CRITICAL_BAND {
        let wd = w0 * (1.0 - zeta * zeta).sqrt();
        let decay = (-zeta * w0 * t).exp();
        let b = (v0 + zeta * w0 * x0) / wd;
        let (sin, cos) = (wd * t).sin_cos();
        let x = decay * (x0 * cos + b * sin);
        let v = decay * (v0 * cos - (zeta * w0 * b + wd * x0) * sin);
        (x, v)
    } else if zeta <= 1.0 + CRITICAL_BAND {
        let decay = (-w0 * t).exp();
        let c = v0 + w0 * x0;
        (decay * (x0 + c * t), decay * (v0 - w0 * c * t))
    } else {
        let s = (zeta * zeta - 1.0).sqrt();
        let r1 = -w0 / (zeta + s);
        let r2 = -w0 * (zeta + s);
        let a = (v0 - r2 * x0) / (r1 - r2);
        let b = x0 - a;
        let (e1, e2) = ((r1 * t).exp(), (r2 * t).exp());
        (a * e1 + b * e2, a * r1 * e1 + b * r2 * e2)
    }
}

/// Seconds (for a natural frequency of 1) after which a 0 to 1 run stays within
/// `rest` in both position and velocity.
fn settle_time(zeta: f64, rest: f64) -> f64 {
    let energy = |t: f64| {
        let (x, v) = oscillate(1.0, zeta, -1.0, 0.0, t);
        x.hypot(v)
    };
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..64 {
        if energy(hi) <= rest {
            break;
        }
        lo = hi;
        hi *= 2.0;
    }
    for _ in 0..200 {
        if hi - lo <= 1e-7 {
            break;
        }
        let mid = (lo + hi) / 2.0;
        if energy(mid) <= rest {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

/// Advance a spring by `dt` seconds with the analytic solution, component by
/// component, from the previous position `x` and velocity `v`; returns whether
/// every component is at rest. A spring with a problem finishes at once.
fn spring_advance(
    s: Spring,
    x: &mut [f64],
    v: &mut [f64],
    to: &[f64],
    amplitude: &[f64],
    dt: f64,
) -> bool {
    if s.problem().is_some() {
        put(x, to);
        v.fill(0.0);
        return true;
    }
    let m = s.mass.max(1e-3);
    let w0 = (s.stiffness / m).sqrt();
    let zeta = s.damping / (2.0 * (s.stiffness * m).sqrt());
    let dt = dt.max(0.0);
    let mut settled = true;
    for (i, ((x, v), to)) in x.iter_mut().zip(v.iter_mut()).zip(to).enumerate() {
        let (offset, velocity) = oscillate(w0, zeta, *x - to, *v, dt);
        *x = to + offset;
        *v = velocity;
        let a = amplitude.get(i).copied().unwrap_or(1.0);
        settled &= offset.hypot(velocity / w0) <= s.rest * a;
    }
    settled
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
    /// Every iteration plays from start to end.
    Normal,
    /// Every other iteration plays backwards.
    Alternate,
}

/// Interpolation model and parameters of a transition.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// A timed interpolation along one easing.
    Tween {
        /// Duration of one iteration, in milliseconds.
        ms: f64,
        /// Mapping from elapsed time to progress.
        easing: Easing,
    },
    /// A physical spring that settles by itself.
    Spring(Spring),
    /// Timed interpolation through frames.
    Keyframes {
        /// (offset 0..=1, components, easing of the segment that starts here)
        frames: Vec<(f64, Vec<f64>, Easing)>,
        /// Duration of one iteration, in milliseconds.
        ms: f64,
    },
}

/// One keyframe of [`Transition::keyframes`].
#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe<T> {
    /// Position in the iteration, `0..=1`.
    pub offset: f64,
    /// The value at `offset`.
    pub value: T,
    /// Easing of the segment that starts at this keyframe.
    pub easing: Easing,
}

impl<T> Keyframe<T> {
    /// A keyframe at `offset` with a linear segment easing.
    pub fn new(offset: f64, value: T) -> Keyframe<T> {
        Keyframe {
            offset,
            value,
            easing: Easing::Linear,
        }
    }

    /// This keyframe with `easing` for the segment that starts here.
    pub fn easing(self, easing: Easing) -> Keyframe<T> {
        Keyframe { easing, ..self }
    }
}

impl<T> From<(f64, T)> for Keyframe<T> {
    fn from((offset, value): (f64, T)) -> Keyframe<T> {
        Keyframe::new(offset, value)
    }
}

/// Timing when an active animation receives a new target.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Retarget {
    /// Start again with the configured duration.
    #[default]
    Restart,
    /// Scale duration by the distance to the new target.
    Proportional,
}

/// How a value moves to a new target.
#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    /// Interpolation model for each iteration.
    pub kind: Kind,
    /// Delay before the first iteration, in milliseconds.
    pub delay_ms: f64,
    /// How many iterations to run.
    pub repeat: Repeat,
    /// Whether successive iterations reverse direction.
    pub direction: PlayDirection,
    /// Allow the host to suppress this transition for reduced motion.
    pub decorative: bool,
    /// Timing policy when interrupted.
    pub retarget: Retarget,
}

/// Copy `values` into `out`, component by component.
fn put(out: &mut [f64], values: &[f64]) {
    for (out, value) in out.iter_mut().zip(values) {
        *out = *value;
    }
}

/// Write the components `t` of the way from `a` to `b` into `out`.
pub fn lerp(a: &[f64], b: &[f64], t: f64, out: &mut [f64]) {
    for ((out, x), y) in out.iter_mut().zip(a).zip(b) {
        *out = x + (y - x) * t;
    }
}

/// What a run keeps between its steps besides its endpoints and start time: a
/// spring's velocity, the host time of the previous step, the iteration a
/// spring plays and the spring's amplitude. A timed run (tween, keyframes)
/// reads none of it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpringRun {
    /// Velocity of each component.
    pub velocity: Vec<f64>,
    /// Host time (ns) of the previous step.
    pub last_ns: u64,
    /// The iteration a spring plays, from 0.
    pub iteration: u32,
    /// Amplitude of each component, computed by [`SpringRun::start`] (a pure
    /// cache; empty when the transition is not a spring).
    pub amplitude: Vec<f64>,
}

impl SpringRun {
    /// Start a run at `now_ns` from `from` toward `to`, keeping the velocity
    /// (an interrupted spring goes on with it), and compute the amplitude from
    /// `from`, `to`, the velocity and `transition`'s spring (empty when the
    /// transition is not a spring).
    pub fn start(&mut self, now_ns: u64, transition: &Transition, from: &[f64], to: &[f64]) {
        self.last_ns = now_ns;
        self.iteration = 0;
        self.velocity.resize(to.len(), 0.0);
        self.amplitude.clear();
        let Kind::Spring(spring) = &transition.kind else {
            return;
        };
        let w0 = (spring.stiffness / spring.mass.max(1e-3)).sqrt();
        let velocity = &self.velocity;
        self.amplitude.extend(to.iter().enumerate().map(|(i, to)| {
            let distance = from.get(i).map_or(0.0, |from| (to - from).abs());
            let speed = velocity[i].abs() / w0;
            distance.max(speed).clamp(AMPLITUDE_MIN, AMPLITUDE_MAX)
        }));
    }
}

impl Transition {
    /// Choose timing when the target changes during a run.
    pub fn retarget(mut self, retarget: Retarget) -> Self {
        self.retarget = retarget;
        self
    }

    /// Derive timing for an interrupted component animation.
    // xtask-allow(internal-contract): Shared component timing bridges runtime and host animation drivers.
    #[doc(hidden)]
    pub fn retargeted(&self, from: &[f64], to: &[f64], current: &[f64], target: &[f64]) -> Self {
        let mut timing = self.clone();
        if self.retarget == Retarget::Proportional {
            let distance = |a: &[f64], b: &[f64]| {
                a.iter()
                    .zip(b)
                    .map(|(a, b)| (b - a).powi(2))
                    .sum::<f64>()
                    .sqrt()
            };
            let original = distance(from, to);
            if original > f64::EPSILON {
                let ratio = distance(current, target) / original;
                match &mut timing.kind {
                    Kind::Tween { ms, .. } | Kind::Keyframes { ms, .. } => *ms *= ratio,
                    Kind::Spring(_) => {}
                }
                timing.delay_ms = 0.0;
            }
        }
        timing
    }

    /// Play absolute component keyframes in offset order.
    pub fn keyframes<T: crate::Animatable>(
        frames: impl IntoIterator<Item = impl Into<Keyframe<T>>>,
        ms: impl Into<f64>,
    ) -> Self {
        let mut frames: Vec<(f64, Vec<f64>, Easing)> = frames
            .into_iter()
            .map(|frame| {
                let frame: Keyframe<T> = frame.into();
                let mut components = Vec::with_capacity(T::COMPONENTS);
                frame.value.to_components(&mut components);
                (frame.offset.clamp(0.0, 1.0), components, frame.easing)
            })
            .collect();
        frames.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self::new(Kind::Keyframes {
            frames,
            ms: ms.into().max(0.0),
        })
    }

    fn new(kind: Kind) -> Self {
        Self {
            kind,
            delay_ms: 0.0,
            repeat: Repeat::Times(1),
            direction: PlayDirection::Normal,
            decorative: true,
            retarget: Retarget::Restart,
        }
    }

    /// A tween over `ms` milliseconds along `easing`.
    pub fn tween(ms: impl Into<f64>, easing: Easing) -> Self {
        Self::new(Kind::Tween {
            ms: (ms.into()).max(0.0),
            easing,
        })
    }

    /// A spring with the given stiffness, damping and mass.
    pub fn spring(spring: Spring) -> Self {
        Self::new(Kind::Spring(spring))
    }

    /// How long the first iteration takes to end, its delay included, in
    /// milliseconds: `None` for a spring, which settles by its physics.
    pub fn duration_ms(&self) -> Option<f64> {
        match &self.kind {
            Kind::Tween { ms, .. } | Kind::Keyframes { ms, .. } => Some(self.delay_ms + ms),
            Kind::Spring(_) => None,
        }
    }

    /// Wait `ms` milliseconds before the first iteration starts.
    pub fn delay(mut self, ms: impl Into<f64>) -> Self {
        self.delay_ms = (ms.into()).max(0.0);
        self
    }

    /// Set the number of iterations.
    pub fn repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }

    /// Play every other iteration backwards.
    pub fn alternate(mut self) -> Self {
        self.direction = PlayDirection::Alternate;
        self
    }

    /// Keep motion that communicates information under reduced motion.
    pub fn essential(mut self) -> Self {
        self.decorative = false;
        self
    }

    /// The value a finished run ends on: the end of its last iteration, which
    /// is `from` for an alternating run of an even iteration count and `to`
    /// otherwise.
    fn end_value<'a>(&self, from: &'a [f64], to: &'a [f64]) -> &'a [f64] {
        let reversed = match self.repeat {
            Repeat::Times(n) => n > 0 && n % 2 == 0,
            Repeat::Forever => false,
        };
        if reversed && self.direction == PlayDirection::Alternate && from.len() == to.len() {
            from
        } else {
            to
        }
    }

    /// Step a run from `from` to `to` that began at `start_ns` to `now_ns`:
    /// write its value into `current` (the value of the previous step) and
    /// return whether it is done (then `current` is the end of the last
    /// iteration: `from` for an alternating run of an even iteration count,
    /// `to` otherwise). The one stepper of client animations and host state
    /// transitions; a spring run is started with [`SpringRun::start`].
    pub fn step(
        &self,
        from: &[f64],
        to: &[f64],
        start_ns: u64,
        now_ns: u64,
        current: &mut Vec<f64>,
        run: &mut SpringRun,
    ) -> bool {
        if current.len() != to.len() {
            current.clear();
            current.extend_from_slice(to);
        }
        let elapsed_ms = now_ns.saturating_sub(start_ns) as f64 / 1e6;
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
        let dt = now_ns.saturating_sub(run.last_ns) as f64 / 1e9;
        run.last_ns = now_ns;
        run.velocity.resize(to.len(), 0.0);
        let paired = from.len() == to.len();
        let backwards = self.direction == PlayDirection::Alternate && run.iteration % 2 == 1;
        let target = if backwards && paired { from } else { to };
        let amplitude = &run.amplitude;
        if !spring_advance(*spring, current, &mut run.velocity, target, amplitude, dt) {
            return false;
        }
        run.velocity.fill(0.0);
        let next = run.iteration.saturating_add(1);
        let more = match self.repeat {
            Repeat::Times(n) => next < n,
            Repeat::Forever => true,
        };
        if !more {
            current.copy_from_slice(target);
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
    fn sample_timed(&self, from: &[f64], to: &[f64], elapsed_ms: f64, out: &mut [f64]) -> bool {
        let t_ms = elapsed_ms - self.delay_ms;
        if t_ms < 0.0 {
            put(out, from);
            return false;
        }
        let dur = match &self.kind {
            Kind::Tween { ms, .. } | Kind::Keyframes { ms, .. } => *ms,
            Kind::Spring(_) => 0.0,
        };
        let iter = (t_ms / dur).floor();
        let total = match self.repeat {
            Repeat::Times(n) => n as f64,
            Repeat::Forever => f64::INFINITY,
        };
        if dur <= 0.0 || iter >= total {
            put(out, self.end_value(from, to));
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
/// without any) into `out`, easing each segment with the easing of the frame
/// it starts at.
fn keyframe_at(frames: &[(f64, Vec<f64>, Easing)], p: f64, fallback: &[f64], out: &mut [f64]) {
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
            lerp(&a.1, &b.1, a.2.apply((p - a.0) / span), out);
            return;
        }
        a = b;
    }
    put(out, &a.1);
}
