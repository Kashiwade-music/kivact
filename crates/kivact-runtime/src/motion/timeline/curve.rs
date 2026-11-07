//! Role: The value of a timeline's track segment at a time: a timed
//! transition through the shared stepper, a spring by the closed form of
//! design §12.6.1, and the rest time of a spring.

use kivact_types::motion::{PlayDirection, Spring, SpringRun, Transition, lerp};

use super::super::Repeat;
use super::super::transition::{Kind, settles_at_start};
use super::plan::{SegmentDef, SegmentKind};

/// Largest rest time (ms) a spring segment is measured to; a spring that is
/// still moving at this time ends here.
const MAX_REST_MS: u64 = 60_000;
/// Band around critical damping (zeta = 1) that uses the critical solution.
const CRITICAL_BAND: f64 = 1e-6;

fn to_ns(ms: f64) -> u64 {
    (ms.max(0.0) * 1e6).round() as u64
}

/// The value of a track segment `t_ms` after its start, given its `from`.
pub(super) fn seg_value(seg: &SegmentDef, from: &[f64], t_ms: f64) -> Vec<f64> {
    let SegmentKind::Track {
        to,
        transition,
        one_ms,
        ..
    } = &seg.kind
    else {
        return from.to_vec();
    };
    let mut out = from.to_vec();
    match &transition.kind {
        Kind::Spring(spring) => {
            let p = spring_fraction(spring, transition, *one_ms, t_ms);
            lerp(from, to, p, &mut out);
        }
        _ => {
            // A timed transition is stateless: its value at a time is the one
            // `Transition::step` gives from the start.
            transition.step(
                from,
                to,
                0,
                to_ns(t_ms),
                &mut out,
                &mut SpringRun::default(),
            );
        }
    }
    out
}

/// The progress (0 at the start, 1 at the target) of a spring segment
/// `t_ms` after its start, with iterations of `one_ms` (§12.6.1).
fn spring_fraction(spring: &Spring, timing: &Transition, one_ms: f64, t_ms: f64) -> f64 {
    let t = t_ms - timing.delay_ms;
    if t < 0.0 {
        return 0.0;
    }
    let total = match timing.repeat {
        Repeat::Times(n) => f64::from(n),
        Repeat::Forever => f64::INFINITY,
    };
    let iter = if one_ms > 0.0 {
        (t / one_ms).floor()
    } else {
        total
    };
    if iter >= total {
        return if settles_at_start(timing) { 0.0 } else { 1.0 };
    }
    let p = spring_progress(spring, (t - iter * one_ms) / 1000.0);
    if timing.direction == PlayDirection::Alternate && iter % 2.0 == 1.0 {
        1.0 - p
    } else {
        p
    }
}

/// The step response of `spring` (from rest, 0 to 1) at `t` seconds: the
/// closed form of §12.6.1. A spring with a problem reads as already at its
/// target.
fn spring_progress(spring: &Spring, t: f64) -> f64 {
    if spring.problem().is_some() {
        return 1.0;
    }
    let mass = spring.mass.max(1e-3);
    let w0 = (spring.stiffness / mass).sqrt();
    let zeta = spring.damping / (2.0 * (spring.stiffness * mass).sqrt());
    let t = t.max(0.0);
    if zeta < 1.0 - CRITICAL_BAND {
        let wd = w0 * (1.0 - zeta * zeta).sqrt();
        let (sin, cos) = (wd * t).sin_cos();
        1.0 - (-zeta * w0 * t).exp() * (cos + zeta * w0 / wd * sin)
    } else if zeta <= 1.0 + CRITICAL_BAND {
        1.0 - (-w0 * t).exp() * (1.0 + w0 * t)
    } else {
        let root = (zeta * zeta - 1.0).sqrt();
        let r1 = -w0 * (zeta - root);
        let r2 = -w0 * (zeta + root);
        1.0 - (r2 * (r1 * t).exp() - r1 * (r2 * t).exp()) / (r2 - r1)
    }
}

/// The time (ms) a spring from rest to 1 stays settled from (the rest time of
/// `Transition::step`, measured in 1 ms steps).
pub(super) fn spring_rest_ms(spring: &Spring) -> f64 {
    let transition = Transition::spring(*spring);
    let mut run = SpringRun::default();
    run.start(0, &transition, &[0.0], &[1.0]);
    let mut current = vec![0.0];
    for ms in 1..=MAX_REST_MS {
        if transition.step(&[0.0], &[1.0], 0, ms * 1_000_000, &mut current, &mut run) {
            return ms as f64;
        }
    }
    MAX_REST_MS as f64
}
