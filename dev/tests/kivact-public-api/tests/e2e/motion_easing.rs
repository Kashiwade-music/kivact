//! Role: Easing curves, springs and repeat rules of client animations.

use std::time::Duration;

use crate::support::mount_with;
use kivact::motion::{
    Animated, Easing, Keyframe, Repeat, Spring, StepPosition, Transition, animated,
};
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::test::{RenderOptions, Screen};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A display refreshing every millisecond, so values are read at exact times.
fn exact_frames() -> RenderOptions {
    RenderOptions::from_env().frame_period_ns(1_000_000)
}

/// One animated value per transition, every one heading for the same target.
fn animated_values(transitions: Vec<Transition>) -> (Screen, (Signal<f64>, Vec<Animated<f64>>)) {
    mount_with(exact_frames(), move || {
        let target = signal(0.0f64);
        let values: Vec<Animated<f64>> = transitions
            .into_iter()
            .map(|tr| animated(target, tr))
            .collect();
        ((target, values), text("easing"))
    })
}

/// Set the target, then read every value at each of `times` (ms after the
/// start of the run). Row `j` holds the values at `times[j]`.
fn read_at(
    screen: &Screen,
    target: Signal<f64>,
    values: &[Animated<f64>],
    to: f64,
    times: &[u64],
) -> Vec<Vec<f64>> {
    target.set(to);
    screen.run_until_idle();
    let mut now = 0;
    let mut rows = Vec::new();
    for &t in times {
        screen.advance(ms(t - now));
        now = t;
        rows.push(values.iter().map(|v| v.get_untracked()).collect());
    }
    rows
}

/// CSS cubic-bezier(x1, y1, x2, y2) at input `t`, by bisection on the curve
/// parameter (CSS Easing 1). Written here from the curve equation, not from
/// the implementation under test.
fn bezier_reference(x1: f64, y1: f64, x2: f64, y2: f64, t: f64) -> f64 {
    let bx =
        |s: f64| 3.0 * (1.0 - s) * (1.0 - s) * s * x1 + 3.0 * (1.0 - s) * s * s * x2 + s * s * s;
    let by =
        |s: f64| 3.0 * (1.0 - s) * (1.0 - s) * s * y1 + 3.0 * (1.0 - s) * s * s * y2 + s * s * s;
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if bx(mid) < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    by((lo + hi) / 2.0)
}

/// Values are math on a 0..1000 span, read at 1 ms frames: the tolerance is
/// absolute and far below one frame of change.
const VALUE_TOLERANCE: f64 = 0.01;

fn assert_value(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= VALUE_TOLERANCE,
        "{what}: expected {expected}, got {actual}"
    );
}

#[test]
fn css_named_easings() {
    let named = [
        (Easing::Ease, (0.25, 0.1, 0.25, 1.0)),
        (Easing::EaseIn, (0.42, 0.0, 1.0, 1.0)),
        (Easing::EaseOut, (0.0, 0.0, 0.58, 1.0)),
        (Easing::EaseInOut, (0.42, 0.0, 0.58, 1.0)),
    ];
    let transitions = named
        .iter()
        .map(|(easing, _)| Transition::tween(1000, easing.clone()))
        .collect();
    let (screen, (target, values)) = animated_values(transitions);
    let rows = read_at(&screen, target, &values, 1000.0, &[250, 500, 750]);
    for (column, (_, (x1, y1, x2, y2))) in named.iter().enumerate() {
        for (row, t) in [0.25, 0.5, 0.75].iter().enumerate() {
            let expected = 1000.0 * bezier_reference(*x1, *y1, *x2, *y2, *t);
            assert_value(
                rows[row][column],
                expected,
                &format!("easing {column} at {t}"),
            );
        }
    }
}

#[test]
fn linear_points_and_steps() {
    let transitions = vec![
        // linear(): 0 at 0, 0.5 at 0.5 (output 1.0), 1 at 1.
        Transition::tween(
            1000,
            Easing::linear_points([(None, 0.0), (Some(0.5), 1.0), (None, 1.0)]),
        ),
        Transition::tween(1000, Easing::steps(4, StepPosition::JumpNone)),
        Transition::tween(1000, Easing::steps(4, StepPosition::JumpBoth)),
        Transition::tween(1000, Easing::steps(4, StepPosition::JumpStart)),
        Transition::tween(1000, Easing::steps(4, StepPosition::JumpEnd)),
    ];
    let (screen, (target, values)) = animated_values(transitions);
    let rows = read_at(&screen, target, &values, 1000.0, &[250, 300, 500, 600]);
    // linear(): the output at 0.25 is 0.5 and at 0.5 it is 1.0.
    assert_value(rows[0][0], 500.0, "linear() at 25%");
    assert_value(rows[2][0], 1000.0, "linear() at 50%");
    // JumpNone (3 jumps): step floor(1.2) = 1 at 30%, floor(2.4) = 2 at 60%.
    assert_value(rows[1][1], 1000.0 / 3.0, "jump-none at 30%");
    assert_value(rows[3][1], 2000.0 / 3.0, "jump-none at 60%");
    // JumpBoth (5 jumps): step floor(1.2) + 1 = 2 at 30%.
    assert_value(rows[1][2], 400.0, "jump-both at 30%");
    // JumpStart (4 jumps): step 2 at 30%. JumpEnd (4 jumps): step 1 at 30%.
    assert_value(rows[1][3], 500.0, "jump-start at 30%");
    assert_value(rows[1][4], 250.0, "jump-end at 30%");
}

/// Each named curve of §12.1 at 50%: the formulas give these known values.
#[test]
fn back_elastic_bounce_shapes() {
    let shapes = [
        (Easing::BackIn, -0.0876975),
        (Easing::BackOut, 1.0876975),
        (Easing::BackInOut, 0.5),
        (Easing::ElasticIn, -0.015625),
        (Easing::ElasticOut, 1.015625),
        (Easing::ElasticInOut, 0.5),
        (Easing::BounceIn, 0.234375),
        (Easing::BounceOut, 0.765625),
        (Easing::BounceInOut, 0.5),
    ];
    let transitions = shapes
        .iter()
        .map(|(easing, _)| Transition::tween(1000, easing.clone()))
        .collect();
    let (screen, (target, values)) = animated_values(transitions);
    let rows = read_at(&screen, target, &values, 1000.0, &[500]);
    for (column, (easing, expected)) in shapes.iter().enumerate() {
        assert_value(
            rows[0][column],
            1000.0 * expected,
            &format!("{easing:?} at 50%"),
        );
    }
}

/// Each keyframe's easing shapes the segment that starts at its offset.
#[test]
fn keyframe_segment_easing() {
    let frames = [
        Keyframe::new(0.0, 0.0).easing(Easing::Linear),
        Keyframe::new(0.5, 100.0).easing(Easing::steps(2, StepPosition::JumpEnd)),
        Keyframe::new(1.0, 50.0).easing(Easing::EaseIn),
    ];
    let transition = Transition::keyframes(frames, 1000);
    let (screen, (target, values)) = animated_values(vec![transition]);
    let rows = read_at(&screen, target, &values, 50.0, &[250, 700, 750, 800, 1000]);
    // Segment 0 to 0.5 is linear: 0 to 100 over the first half.
    assert_value(rows[0][0], 50.0, "first segment, linear at 25%");
    // Segment 0.5 to 1 jumps at its midpoint (progress 0.5, t = 750 ms):
    // 100 until 750 ms, 75 after.
    assert_value(rows[1][0], 100.0, "second segment, before its jump");
    assert_value(rows[2][0], 75.0, "second segment, step 1 at 75%");
    assert_value(rows[3][0], 75.0, "second segment, step 1 at 80%");
    assert_value(rows[4][0], 50.0, "the last keyframe holds at the end");
}

/// Springs with a duration settle at that duration, whatever the bounce, the
/// distance and the frame interval.
#[test]
fn spring_with_duration_settles_near_duration() {
    let bounces = [-0.99, -0.5, 0.0, 0.094, 0.5, 0.99];
    let targets = [1.0, 100.0, 0.0005];
    let frame_periods_ms = [1u64, 16, 100];
    for frame_ms in frame_periods_ms {
        let options = RenderOptions::from_env().frame_period_ns(frame_ms * 1_000_000);
        let (screen, (signals, values)) = mount_with(options, move || {
            let mut signals = Vec::new();
            let mut values = Vec::new();
            for target in targets {
                let to = signal(0.0f64);
                signals.push((to, target));
                for bounce in bounces {
                    let spring = Transition::spring(Spring::with_duration(500.0, bounce));
                    values.push(animated(to, spring));
                }
            }
            ((signals, values), text("spring"))
        });
        for (to, target) in &signals {
            to.set(*target);
        }
        screen.run_until_idle();
        let mut settled: Vec<Option<u64>> = vec![None; values.len()];
        for t in 1..=600u64 {
            screen.advance(ms(1));
            for (i, value) in values.iter().enumerate() {
                if settled[i].is_none() && !value.is_running() {
                    settled[i] = Some(t);
                }
            }
        }
        for (i, value) in values.iter().enumerate() {
            let at = settled[i].unwrap_or_else(|| panic!("spring {i} never settled"));
            let (_, target) = signals[i / bounces.len()];
            assert!(
                (500..=500 + frame_ms).contains(&at),
                "frame {frame_ms} ms, spring {i}: settled at {at} ms"
            );
            let v = value.get_untracked();
            assert!(
                (v - target).abs() <= 1e-3 * target,
                "frame {frame_ms} ms, spring {i}: settled at {v}, target {target}"
            );
        }
    }
}

/// The analytic step keeps the value finite and the same at a shared time,
/// whatever the frame interval.
#[test]
fn spring_extremes_stay_finite() {
    let mut at_400: Vec<Vec<f64>> = Vec::new();
    for frame_ms in [16u64, 100] {
        let options = RenderOptions::from_env().frame_period_ns(frame_ms * 1_000_000);
        let (screen, (target, values)) = mount_with(options, move || {
            let target = signal(0.0f64);
            let values = vec![
                animated(
                    target,
                    Transition::spring(Spring::with_duration(500.0, -0.99)),
                ),
                animated(
                    target,
                    Transition::spring(Spring::with_duration(500.0, 0.99)),
                ),
            ];
            ((target, values), text("spring"))
        });
        target.set(1.0);
        screen.run_until_idle();
        let mut row = Vec::new();
        for _ in 0..(400 / frame_ms) {
            screen.advance(ms(frame_ms));
            for value in &values {
                assert!(value.get_untracked().is_finite(), "frame {frame_ms} ms");
            }
        }
        for value in &values {
            row.push(value.get_untracked());
        }
        at_400.push(row);
    }
    for (a, b) in at_400[0].iter().zip(&at_400[1]) {
        assert!((a - b).abs() <= 1e-6, "400 ms: {a} vs {b}");
    }
}

/// A tween repeated an even number of times with `alternate` ends at its
/// from value: an animated value, a timeline track, a style transition. A
/// layout transition with the same timing reports once and moves once.
#[test]
fn alternate_even_count_ends_at_from() {
    let alternate = || {
        Transition::tween(100, Easing::Linear)
            .repeat(Repeat::Times(2))
            .alternate()
    };
    let (screen, handles) = mount_with(exact_frames(), move || {
        let target = signal(0.0f64);
        let shown = animated(target, alternate());
        let tl = timeline();
        let track = tl.track(0.0f64);
        tl.to(track, 100.0, alternate());
        let styled_target = signal(0.0f64);
        let expanded = signal(false);
        let content = column((
            button("Play").test_id("play").on_press(move || tl.play()),
            node().test_id("state").style(move || {
                style()
                    .size(20, 20)
                    .translate_x(styled_target.get())
                    .transition(TransitionProp::All, alternate())
            }),
            node().test_id("layout").style(move || {
                style()
                    .size(if expanded.get() { 200 } else { 40 }, 20)
                    .layout_transition(alternate())
            }),
        ));
        ((target, shown, track, styled_target, expanded), content)
    });
    let (target, shown, track, styled_target, expanded) = handles;
    target.set(100.0);
    styled_target.set(100.0);
    screen.run_until_idle();
    let state_before = screen.get_by_test_id("state").rect().unwrap().x;
    screen.get_by_test_id("play").click();
    // The first pass runs from 0 to 100 in 100 ms. A motion that never starts
    // stays at 0 here, so the peak is sampled before the end.
    screen.advance(ms(100));
    assert!(
        (shown.get_untracked() - 100.0).abs() <= 1.0,
        "animated value peaks at to: {}",
        shown.get_untracked()
    );
    assert!(
        (track.get_untracked() - 100.0).abs() <= 1.0,
        "timeline track peaks at to: {}",
        track.get_untracked()
    );
    assert!(
        (screen.get_by_test_id("state").rect().unwrap().x - state_before - 100.0).abs() <= 1.0,
        "style transition peaks at to"
    );
    screen.advance(ms(300));
    assert_eq!(shown.get_untracked(), 0.0, "animated value ends at from");
    assert!(!shown.is_running());
    assert_eq!(track.get_untracked(), 0.0, "timeline track ends at from");
    assert_eq!(
        screen.get_by_test_id("state").rect().unwrap().x,
        state_before,
        "style transition ends at from"
    );
    expanded.set(true);
    screen.run_until_idle();
    screen.advance(ms(400));
    assert_eq!(screen.get_by_test_id("layout").rect().unwrap().width, 200.0);
    let reports = screen
        .errors()
        .iter()
        .filter(|e| e.message().contains("layout_transition ignores repeat"))
        .count();
    assert_eq!(reports, 1, "reported once: {:?}", screen.errors());
}

#[test]
fn set_velocity_after_alternate_resumes_the_target() {
    let (screen, (target, shown)) = mount_with(exact_frames(), move || {
        let target = signal(0.0f64);
        let shown = animated(
            target,
            Transition::spring(Spring::STIFF)
                .repeat(Repeat::Times(2))
                .alternate(),
        );
        ((target, shown), text("velocity"))
    });
    target.set(10.0);
    screen.run_until_idle();
    let mut once = 0;
    while shown.is_running() {
        screen.advance(ms(50));
        once += 1;
        assert!(once < 200, "the alternate spring completes");
    }
    assert_eq!(shown.get_untracked(), 0.0, "alternate, even count: from");
    shown.set_velocity(5.0);
    screen.run_until_idle();
    screen.advance(ms(20));
    let moving = shown.get_untracked();
    assert!(moving > 0.0, "the velocity moves it toward 10: {moving}");
    screen.advance(Duration::from_secs(5));
    assert!(!shown.is_running());
    assert!(
        (shown.get_untracked() - 10.0).abs() <= 1e-3 * 10.0,
        "rests on the target"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A spring with stiffness 0 is reported once where a style and an animated
/// value take it; the values go straight to the target.
#[test]
fn invalid_spring_is_reported() {
    let bad = Spring {
        stiffness: 0.0,
        ..Spring::DEFAULT
    };
    let (screen, (target, shown)) = mount_with(exact_frames(), move || {
        let target = signal(0.0f64);
        let shown = animated(target, Transition::spring(bad));
        let moved = node().test_id("moved").style(move || {
            style()
                .size(20, 20)
                .translate_x(target.get())
                .transition(TransitionProp::All, Transition::spring(bad))
        });
        ((target, shown), moved)
    });
    target.set(10.0);
    screen.run_until_idle();
    // The value jumps on the first frame (1 ms here), with no interpolation.
    screen.advance(ms(1));
    assert_eq!(shown.get_untracked(), 10.0, "the animated value jumps");
    assert_eq!(
        screen.get_by_test_id("moved").rect().unwrap().x,
        10.0,
        "the style transition jumps"
    );
    let errors = screen.errors();
    assert_eq!(
        errors.len(),
        2,
        "one report for the style and one for the animated value: {errors:?}"
    );
    assert!(
        errors.iter().all(|e| e.message().contains("spring")),
        "{errors:?}"
    );
}

#[test]
fn set_velocity_moves_an_idle_spring() {
    let (screen, shown) = mount_with(exact_frames(), move || {
        let target = signal(0.0f64);
        let shown = animated(target, Transition::spring(Spring::STIFF));
        (shown, text("idle"))
    });
    shown.set_velocity(10.0);
    screen.run_until_idle();
    screen.advance(ms(20));
    assert!(shown.get_untracked() > 0.0, "moves away from 0 at once");
    screen.advance(Duration::from_secs(5));
    assert!(shown.get_untracked().abs() < 1e-3, "returns to the target");
    assert!(!shown.is_running());
}
