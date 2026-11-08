//! Role: Timeline playback: segments, phases, calls, reduced motion, stagger.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount_with;
use kivact::motion::{
    Animated, At, Easing, Repeat, Stagger, StaggerFrom, StepPosition, Timeline, Transition,
    animated, timeline,
};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A display refreshing every millisecond, so values are read at exact times.
fn exact_frames() -> RenderOptions {
    RenderOptions::from_env().frame_period_ns(1_000_000)
}

/// Values are math on a 0..100 span, read at 1 ms frames.
const VALUE_TOLERANCE: f64 = 0.01;

fn assert_value(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= VALUE_TOLERANCE,
        "{what}: expected {expected}, got {actual}"
    );
}

/// What a test asks of a timeline, from a handler or an effect.
#[derive(Clone, Copy, Debug)]
enum Op {
    Play,
    Pause,
    Reverse,
    Finish,
    Seek(f64),
    Rate(f64),
}

/// Playback controls of `tl`: the test calls them through the callback, so
/// they run as effects (the timeline's controls need an effect).
fn control(tl: Timeline) -> kivact::state::Callback<Op> {
    kivact::state::callback(move |op: Op| match op {
        Op::Play => tl.play(),
        Op::Pause => tl.pause(),
        Op::Reverse => tl.reverse(),
        Op::Finish => tl.finish(),
        Op::Seek(at) => tl.seek(at),
        Op::Rate(rate) => tl.set_rate(rate),
    })
}

/// Advance the clock to `to` ms (absolute, from the screen's start).
fn advance_to(screen: &Screen, to: u64) {
    let now = screen.now().as_millis() as u64;
    screen.advance(ms(to - now));
}

#[test]
fn segments_follow_positions() {
    let (screen, (ctl, (x, y, z, w, v))) = mount_with(exact_frames(), || {
        let tl = timeline();
        let x = tl.track(0.0f64);
        let y = tl.track(0.0f64);
        let z = tl.track(0.0f64);
        let w = tl.track(0.0f64);
        let v = tl.track(0.0f64);
        let linear = || Transition::tween(100, Easing::Linear);
        // 0..100 (the first segment starts at 0).
        tl.to(x, 100.0, linear()).at(At::AfterPrevious(0.0));
        // 20..120: WithPrevious starts with the segment before it.
        tl.to(y, 100.0, linear()).at(At::WithPrevious(20.0));
        // 300..400: an absolute time.
        tl.to(z, 100.0, linear()).at(At::Time(300.0));
        tl.label("mid", At::Time(500.0));
        // 550..650: 50 ms after the label.
        tl.to(w, 100.0, linear()).at(At::Label("mid", 50.0));
        // 660..760: 10 ms after the segment before it ends (650).
        tl.to(v, 100.0, linear()).at(At::AfterPrevious(10.0));
        ((control(tl), (x, y, z, w, v)), text("timeline"))
    });
    ctl.call(Op::Play);
    screen.run_until_idle();
    let read = |at: u64| {
        advance_to(&screen, at);
        (
            x.get_untracked(),
            y.get_untracked(),
            z.get_untracked(),
            w.get_untracked(),
            v.get_untracked(),
        )
    };
    let (xs, ys, zs, ws, vs) = read(10);
    assert_value(xs, 10.0, "x at 10 ms");
    assert_value(ys, 0.0, "y before its start at 20 ms");
    assert_value(zs + ws + vs, 0.0, "later segments have not started");
    let (xs, ys, _, _, _) = read(70);
    assert_value(xs, 70.0, "x at 70 ms");
    assert_value(ys, 50.0, "y at 70 ms (WithPrevious 20)");
    let (xs, ys, _, _, _) = read(130);
    assert_value(xs, 100.0, "x holds its end");
    assert_value(ys, 100.0, "y at its end");
    let (_, _, zs, _, _) = read(350);
    assert_value(zs, 50.0, "z at 350 ms (Time 300)");
    let (_, _, _, ws, vs) = read(600);
    assert_value(ws, 50.0, "w at 600 ms (label 500 plus 50)");
    assert_value(vs, 0.0, "v not yet started");
    let (_, _, _, ws, vs) = read(700);
    assert_value(ws, 100.0, "w at its end");
    assert_value(vs, 40.0, "v at 700 ms (after the previous end plus 10)");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn segment_delay_and_repeat_match_transition() {
    let transition = || {
        Transition::tween(100, Easing::Linear)
            .delay(50)
            .repeat(Repeat::Times(2))
    };
    let (screen, (target, ctl, a, b, shown)) = mount_with(exact_frames(), move || {
        let tl = timeline();
        let a = tl.track(0.0f64);
        let b = tl.track(0.0f64);
        let target = signal(0.0f64);
        // The same transition as a value: the track must match it.
        let shown: Animated<f64> = animated(target, transition());
        tl.to(a, 100.0, transition()).at(At::AfterPrevious(0.0));
        // The segment after it starts when the 250 ms one ends.
        tl.to(b, 100.0, Transition::tween(100, Easing::Linear))
            .at(At::AfterPrevious(0.0));
        ((target, control(tl), a, b, shown), text("delay"))
    });
    target.set(100.0);
    screen.run_until_idle();
    ctl.call(Op::Play);
    screen.run_until_idle();
    // Delay 0 until 50; the first iteration ends at 100 (50 ms in: 0.5);
    // the second restarts from 0 at 150; the track holds to at 250.
    let expected = [
        (40u64, 0.0, 0.0),
        (100, 50.0, 0.0),
        (160, 10.0, 0.0),
        (240, 90.0, 0.0),
        (255, 100.0, 5.0),
        (300, 100.0, 50.0),
    ];
    for (at, track_a, track_b) in expected {
        advance_to(&screen, at);
        assert_value(a.get_untracked(), track_a, &format!("segment a at {at} ms"));
        assert_value(b.get_untracked(), track_b, &format!("segment b at {at} ms"));
        if at <= 250 {
            assert_value(
                a.get_untracked(),
                shown.get_untracked(),
                &format!("the transition at {at} ms"),
            );
        }
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Playback keeps its rate and direction across play, pause, reverse and
/// finish (§12.6, the transition table).
#[test]
fn playback_settings_persist_across_phases() {
    let (screen, (a, b, c, ctl_a, ctl_b, ctl_c)) = mount_with(exact_frames(), || {
        let tl_a = timeline();
        let a = tl_a.track(0.0f64);
        tl_a.to(a, 100.0, Transition::tween(1000, Easing::Linear));
        let tl_b = timeline();
        let b = tl_b.track(0.0f64);
        tl_b.to(b, 100.0, Transition::tween(1000, Easing::Linear));
        let tl_c = timeline();
        let c = tl_c.track(0.0f64);
        tl_c.to(c, 100.0, Transition::tween(1000, Easing::Linear));
        (
            (a, b, c, control(tl_a), control(tl_b), control(tl_c)),
            text("playback"),
        )
    });
    // a: the rate set before play applies; b: play, pause, play; c: reverse,
    // finish (at the start, going backwards that is 0), then play from the end.
    ctl_a.call(Op::Rate(2.0));
    ctl_a.call(Op::Play);
    ctl_b.call(Op::Play);
    ctl_c.call(Op::Reverse);
    ctl_c.call(Op::Finish);
    ctl_c.call(Op::Play);
    screen.run_until_idle();
    screen.advance(ms(250));
    assert_value(
        a.get_untracked(),
        50.0,
        "rate 2: 250 ms is 500 ms of timeline",
    );
    assert_value(b.get_untracked(), 25.0, "rate 1");
    assert_value(
        c.get_untracked(),
        75.0,
        "backwards from the end: 1000 - 250 ms",
    );

    ctl_a.call(Op::Pause);
    ctl_b.call(Op::Pause);
    screen.run_until_idle();
    screen.advance(ms(300));
    assert_value(a.get_untracked(), 50.0, "a paused");
    assert_value(b.get_untracked(), 25.0, "b paused");
    assert_value(c.get_untracked(), 45.0, "c keeps playing backwards");

    ctl_a.call(Op::Play);
    ctl_b.call(Op::Play);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert_value(a.get_untracked(), 70.0, "a resumes at rate 2");
    assert_value(b.get_untracked(), 35.0, "b resumes at rate 1");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn seek_reverse_and_rate() {
    let (screen, (x, ctl)) = mount_with(exact_frames(), || {
        let tl = timeline();
        let x = tl.track(0.0f64);
        tl.to(x, 100.0, Transition::tween(1000, Easing::Linear));
        ((x, control(tl)), text("seek"))
    });
    ctl.call(Op::Seek(250.0));
    screen.run_until_idle();
    assert_value(x.get_untracked(), 25.0, "seek to 250 ms");
    ctl.call(Op::Play);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert_value(x.get_untracked(), 35.0, "plays forward from the seek");
    ctl.call(Op::Reverse);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert_value(x.get_untracked(), 25.0, "reverse goes back");
    ctl.call(Op::Rate(2.0));
    screen.run_until_idle();
    screen.advance(ms(100));
    assert_value(
        x.get_untracked(),
        5.0,
        "rate 2 covers 200 ms of timeline backwards",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Half the time at rate 2 and a repeat (Normal, then alternate, which runs the
/// second iteration backwards).
#[test]
fn repeat_and_alternate() {
    let (screen, (x, y, ctl_x, ctl_y)) = mount_with(exact_frames(), || {
        let tl_x = timeline();
        let x = tl_x.track(0.0f64);
        tl_x.to(x, 100.0, Transition::tween(100, Easing::Linear));
        let tl_x = tl_x.repeat(Repeat::Times(2));
        let tl_y = timeline();
        let y = tl_y.track(0.0f64);
        tl_y.to(y, 100.0, Transition::tween(100, Easing::Linear));
        let tl_y = tl_y.repeat(Repeat::Times(2)).alternate();
        ((x, y, control(tl_x), control(tl_y)), text("repeat"))
    });
    ctl_x.call(Op::Play);
    ctl_y.call(Op::Play);
    screen.run_until_idle();
    screen.advance(ms(130));
    // The second iteration starts again from 0: 30 ms into it.
    assert_value(x.get_untracked(), 30.0, "normal repeat restarts from 0");
    // Alternate: the second iteration runs backwards, 70 ms from its start.
    assert_value(
        y.get_untracked(),
        70.0,
        "alternate runs the second pass backwards",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A call fires when playback crosses its time, in either direction, and
/// never when a seek crosses it.
#[test]
fn calls_fire_on_playback_not_seek() {
    let calls = Rc::new(Cell::new(0));
    let c2 = calls.clone();
    let (screen, (x, ctl)) = mount_with(exact_frames(), move || {
        let tl = timeline();
        let x = tl.track(0.0f64);
        tl.to(x, 100.0, Transition::tween(1000, Easing::Linear));
        let counter = c2.clone();
        tl.call(
            At::Time(100.0),
            kivact::state::callback(move |()| counter.set(counter.get() + 1)),
        );
        ((x, control(tl)), text("calls"))
    });
    ctl.call(Op::Play);
    screen.run_until_idle();
    screen.advance(ms(150));
    assert_eq!(calls.get(), 1, "playback crossed 100 ms");
    ctl.call(Op::Seek(50.0));
    screen.run_until_idle();
    ctl.call(Op::Seek(200.0));
    screen.run_until_idle();
    assert_eq!(calls.get(), 1, "a seek across the call does not fire it");
    ctl.call(Op::Reverse);
    ctl.call(Op::Play);
    screen.run_until_idle();
    screen.advance(ms(150));
    assert_eq!(calls.get(), 2, "playing backwards across 100 ms fires it");
    assert_value(
        x.get_untracked(),
        5.0,
        "backwards from 200 ms: time 50 ms, 5 of 100",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A non-finite seek is reported and ignored: the playback goes on from where
/// it was, and does not wedge at a NaN time.
#[test]
fn non_finite_seek_is_reported_and_ignored() {
    let (screen, (x, ctl)) = mount_with(exact_frames(), || {
        let tl = timeline();
        let x = tl.track(0.0f64);
        tl.to(x, 100.0, Transition::tween(1000, Easing::Linear));
        ((x, control(tl)), text("nan seek"))
    });
    ctl.call(Op::Seek(250.0));
    screen.run_until_idle();
    ctl.call(Op::Seek(f64::NAN));
    ctl.call(Op::Play);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert_value(x.get_untracked(), 35.0, "play goes on from 250 ms");
    assert_eq!(screen.errors().len(), 1, "{:?}", screen.errors());
}

/// A call fires once in every repetition, and an odd repetition of an
/// alternate playback runs it mirrored, as the tracks run.
#[test]
fn calls_fire_in_every_repetition() {
    let normal = Rc::new(Cell::new(0));
    let mirrored = Rc::new(Cell::new(0));
    let endless = Rc::new(Cell::new(0));
    let (n2, m2, e2) = (normal.clone(), mirrored.clone(), endless.clone());
    let (screen, (ctl_n, ctl_m, ctl_e)) = mount_with(exact_frames(), move || {
        let tl_n = timeline();
        let x = tl_n.track(0.0f64);
        tl_n.to(x, 100.0, Transition::tween(100, Easing::Linear));
        let counter = n2.clone();
        tl_n.call(
            At::Time(50.0),
            kivact::state::callback(move |()| counter.set(counter.get() + 1)),
        );
        let tl_n = tl_n.repeat(Repeat::Times(2));

        let tl_m = timeline();
        let y = tl_m.track(0.0f64);
        tl_m.to(y, 100.0, Transition::tween(100, Easing::Linear));
        let counter = m2.clone();
        tl_m.call(
            At::Time(30.0),
            kivact::state::callback(move |()| counter.set(counter.get() + 1)),
        );
        let tl_m = tl_m.repeat(Repeat::Times(2)).alternate();

        let tl_e = timeline();
        let z = tl_e.track(0.0f64);
        tl_e.to(z, 100.0, Transition::tween(100, Easing::Linear));
        let counter = e2.clone();
        tl_e.call(
            At::Time(50.0),
            kivact::state::callback(move |()| counter.set(counter.get() + 1)),
        );
        let tl_e = tl_e.repeat(Repeat::Forever);
        (
            (control(tl_n), control(tl_m), control(tl_e)),
            text("rep calls"),
        )
    });
    ctl_n.call(Op::Play);
    ctl_m.call(Op::Play);
    ctl_e.call(Op::Play);
    screen.run_until_idle();
    // Normal: 50 ms in the first repetition. Alternate: 30 ms in the first,
    // and the mirrored call of the second at 100 + (100 - 30) = 170 ms.
    screen.advance(ms(40));
    assert_eq!(normal.get(), 0, "the call at 50 ms is not due yet");
    assert_eq!(mirrored.get(), 1, "the call at 30 ms fired");
    screen.advance(ms(60));
    assert_eq!(normal.get(), 1, "the first repetition at 50 ms");
    assert_eq!(mirrored.get(), 1, "the mirrored call is not due at 100 ms");
    // 200 ms: the second repetition's call at 150 ms; the endless playback
    // has also fired at 50 and 150 ms.
    screen.advance(ms(100));
    assert_eq!(normal.get(), 2, "the second repetition at 150 ms");
    assert_eq!(
        mirrored.get(),
        2,
        "the mirrored second repetition at 170 ms"
    );
    assert_eq!(endless.get(), 2, "endless: 50 and 150 ms");
    // 260 ms: the endless playback's third call at 250 ms; the others stop.
    screen.advance(ms(60));
    assert_eq!(normal.get(), 2, "a two-time repeat does not play a third");
    assert_eq!(mirrored.get(), 2, "the alternate playback ends at 200 ms");
    assert_eq!(endless.get(), 3, "endless: 250 ms");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The track follows the progress value of a driven timeline.
#[test]
fn drive_by_progress() {
    let (screen, (progress, x)) = mount_with(exact_frames(), || {
        let progress = signal(0.0f64);
        let tl = timeline().drive(progress);
        let x = tl.track(0.0f64);
        tl.to(x, 100.0, Transition::tween(1000, Easing::Linear));
        ((progress, x), text("driven"))
    });
    progress.set(0.25);
    screen.run_until_idle();
    assert_value(x.get_untracked(), 25.0, "progress 0.25");
    progress.set(0.5);
    screen.run_until_idle();
    assert_value(x.get_untracked(), 50.0, "progress 0.5");
    progress.set(2.0);
    screen.run_until_idle();
    assert_value(x.get_untracked(), 100.0, "progress clamps to 1");
}

/// A non-essential timeline finishes at once under reduced motion and calls
/// `on_complete`; an essential one still plays.
#[test]
fn reduced_motion_finishes_decorative() {
    let completions = Rc::new(Cell::new(0));
    let c2 = completions.clone();
    let (screen, (x, y, ctl_x, ctl_y)) = mount_with(exact_frames(), move || {
        let counter = c2.clone();
        let tl_x = timeline().on_complete(kivact::state::callback(move |()| {
            counter.set(counter.get() + 1)
        }));
        let x = tl_x.track(0.0f64);
        tl_x.to(x, 100.0, Transition::tween(1000, Easing::Linear));
        let tl_y = timeline().essential();
        let y = tl_y.track(0.0f64);
        tl_y.to(y, 100.0, Transition::tween(1000, Easing::Linear));
        ((x, y, control(tl_x), control(tl_y)), text("motion"))
    });
    screen.set_reduced_motion(true);
    ctl_x.call(Op::Play);
    ctl_y.call(Op::Play);
    screen.run_until_idle();
    assert_value(x.get_untracked(), 100.0, "decorative: at its end at once");
    assert_eq!(completions.get(), 1, "on_complete runs");
    screen.advance(ms(100));
    let essential = y.get_untracked();
    assert!(
        essential > 0.0 && essential < 100.0,
        "essential still plays: {essential}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Stagger delays, through an animated value per item: at 25 ms each item's
/// value is 100 * (25 - delay) / 100 for a 100 ms tween.
#[test]
fn stagger_delays() {
    // Each case: its stagger, and the value of its five items at 25 ms.
    let cases: [(Stagger, [f64; 5]); 6] = [
        (
            Stagger::new(10.0).from(StaggerFrom::Start),
            [25.0, 15.0, 5.0, 0.0, 0.0],
        ),
        (
            Stagger::new(10.0).from(StaggerFrom::End),
            [0.0, 0.0, 5.0, 15.0, 25.0],
        ),
        (
            Stagger::new(10.0).from(StaggerFrom::Center),
            [5.0, 15.0, 25.0, 15.0, 5.0],
        ),
        (
            Stagger::new(10.0).from(StaggerFrom::Edges),
            [25.0, 15.0, 5.0, 15.0, 25.0],
        ),
        (
            Stagger::new(10.0).from(StaggerFrom::Index(3)),
            [0.0, 5.0, 15.0, 25.0, 15.0],
        ),
        // steps(3, JumpNone) over d / D = 0, 0.25, 0.5, 0.75, 1 gives 0, 0, 0.5, 1, 1;
        // delays 0, 0, 20, 40, 40.
        (
            Stagger::new(10.0)
                .from(StaggerFrom::Start)
                .easing(Easing::steps(3, StepPosition::JumpNone)),
            [25.0, 25.0, 5.0, 0.0, 0.0],
        ),
    ];
    let expected: Vec<[f64; 5]> = cases.iter().map(|(_, values)| *values).collect();
    let staggers: Vec<Stagger> = cases.into_iter().map(|(stagger, _)| stagger).collect();
    let (screen, rows) = mount_with(exact_frames(), move || {
        let mut rows: Vec<(Signal<f64>, Vec<Animated<f64>>)> = Vec::new();
        for stagger in &staggers {
            let target = signal(0.0f64);
            let items = (0..5)
                .map(|i| {
                    animated(
                        target,
                        Transition::tween(100, Easing::Linear).delay(stagger.delay(i, 5)),
                    )
                })
                .collect();
            rows.push((target, items));
        }
        (rows, text("stagger"))
    });
    for (target, _) in &rows {
        target.set(100.0);
    }
    screen.run_until_idle();
    screen.advance(ms(25));
    for (case, (_, items)) in rows.iter().enumerate() {
        for (i, item) in items.iter().enumerate() {
            assert_value(
                item.get_untracked(),
                expected[case][i],
                &format!("case {case}, item {i}"),
            );
        }
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
