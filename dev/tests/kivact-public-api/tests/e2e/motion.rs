//! Role: Motion timing: clock, frame loops, animated values and transitions.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::support::{mount, mount_with};
use kivact::motion::FrameRate;
use kivact::motion::{
    Animated, Easing, FrameInfo, FrameLoop, Spring, Transition, animated, frame_loop,
};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::Color;
use kivact::style::TransitionProp;
use kivact::test::{RenderOptions, Screen};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn counter_app() -> (Screen, (FrameLoop, Rc<Cell<u64>>)) {
    counter_app_with(RenderOptions::from_env())
}

/// A display refreshing every millisecond: frame loops up to 240 Hz tick
/// at their own rate (a 60 Hz display caps them at 60 Hz, as the host
/// paces ticks at most once per refresh).
fn fast_display() -> RenderOptions {
    RenderOptions {
        frame_period_ns: 1_000_000,
        ..RenderOptions::from_env()
    }
}

fn counter_app_with(opts: RenderOptions) -> (Screen, (FrameLoop, Rc<Cell<u64>>)) {
    let ticks = Rc::new(Cell::new(0u64));
    let t2 = ticks.clone();
    mount_with(opts, move || {
        let t3 = t2.clone();
        let fl = frame_loop(move |_f: FrameInfo| t3.set(t3.get() + 1));
        ((fl, t2), text("frames"))
    })
}

#[test]
fn fixed_rates_tick_the_theoretical_number_of_times() {
    for hz in [1u64, 30, 60, 120, 144, 240] {
        let (screen, (fl, ticks)) = counter_app_with(fast_display());
        fl.start(FrameRate::Hz(hz as f64));
        screen.advance(Duration::from_secs(1));
        assert_eq!(ticks.get(), hz, "{hz} Hz over one second");
        assert_eq!(fl.rate().get_untracked(), Some(FrameRate::Hz(hz as f64)));
    }
}

#[test]
fn changing_the_rate_midway_and_stopping() {
    let (screen, (fl, ticks)) = counter_app_with(fast_display());
    fl.start(FrameRate::Hz(60.0));
    screen.advance(ms(500));
    assert_eq!(ticks.get(), 30);
    fl.start(FrameRate::Hz(120.0));
    screen.advance(ms(500));
    assert_eq!(ticks.get(), 90);
    fl.stop();
    screen.advance(ms(500));
    assert_eq!(ticks.get(), 90);
    assert_eq!(fl.rate().get_untracked(), None);
    fl.start(FrameRate::Hz(f64::NAN)); // rejected with a diagnostic
    screen.run_until_idle();
    assert!(!screen.errors().is_empty());
}

#[test]
fn display_rate_follows_the_host_refresh() {
    let opts = RenderOptions {
        frame_period_ns: 10_000_000,
        ..RenderOptions::from_env()
    };
    let ticks = Rc::new(Cell::new(0u64));
    let t2 = ticks.clone();
    let (screen, fl) = mount_with(opts, move || {
        let fl = frame_loop(move |_| t2.set(t2.get() + 1));
        (fl, text("x"))
    });
    fl.start(FrameRate::Display);
    screen.advance(Duration::from_secs(1));
    assert_eq!(ticks.get(), 100, "100 Hz display");
}

#[test]
fn a_faster_loop_than_the_display_ticks_at_the_display_rate() {
    for hz in [120.0, 240.0] {
        let (screen, (fl, ticks)) = counter_app();
        fl.start(FrameRate::Hz(hz));
        screen.advance(Duration::from_secs(1));
        assert_eq!(ticks.get(), 60, "{hz} Hz on a 60 Hz display");
    }
}

#[test]
fn surface_loss_pauses_frames_without_a_burst_on_restore() {
    let (screen, (fl, ticks)) = counter_app();
    fl.start(FrameRate::Hz(60.0));
    screen.advance(ms(500));
    assert_eq!(ticks.get(), 30);
    screen.lose_surface();
    screen.advance(Duration::from_secs(1));
    assert_eq!(ticks.get(), 30, "no ticks while the surface is lost");
    screen.restore_surface();
    screen.advance(ms(500));
    assert_eq!(ticks.get(), 60, "resumes at the rate, no catch-up burst");
}

#[test]
fn a_suspended_app_stops_its_frame_clock_and_redraws_when_resumed() {
    // Android destroys the surface with the suspension; iOS keeps it.
    for destroys_surface in [true, false] {
        let (screen, (target, shown, label)) = mount(move || {
            let target = signal(0.0f32);
            let shown = animated(target, Transition::tween(200, Easing::Linear));
            let label = signal("before".to_string());
            let view = column((
                text(move || format!("{:.0}", shown.get())),
                text(move || label.get()),
                text(move || format!("drawable {}", platform::surface_active())),
            ));
            ((target, shown, label), view)
        });
        target.set(100.0);
        screen.run_until_idle();
        screen.advance(ms(100));
        let halfway = shown.get_untracked();
        assert!((40.0..=60.0).contains(&halfway), "{halfway}");
        let ticks = screen.frame_ticks();
        let shown_frame = screen.screenshot().hash();

        screen.suspend(destroys_surface);
        screen.get_by_text("drawable false");
        // Work in the background changes the state, but nothing is drawn.
        label.set("changed while suspended".into());
        screen.advance(Duration::from_secs(1));
        assert_eq!(
            screen.frame_ticks(),
            ticks,
            "no frame ticks while suspended"
        );
        assert_eq!(
            shown.get_untracked(),
            halfway,
            "the frame clock stood still"
        );
        assert_eq!(
            screen.screenshot().hash(),
            shown_frame,
            "nothing is drawn while suspended"
        );

        screen.resume();
        screen.get_by_text("drawable true");
        screen.get_by_text("changed while suspended");
        assert_ne!(screen.screenshot().hash(), shown_frame, "redrawn on resume");
        // The transition goes on from where it stood instead of jumping to
        // its end: the second it was suspended does not count.
        screen.advance(ms(50));
        let later = shown.get_untracked();
        assert!(
            later > halfway && later < 100.0,
            "{halfway} → {later} (destroys surface: {destroys_surface})"
        );
        screen.advance(ms(200));
        assert_eq!(shown.get_untracked(), 100.0);
        assert!(screen.frame_ticks() > ticks);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn a_memory_warning_is_noted_and_the_app_keeps_drawing() {
    let (screen, count) = mount(|| {
        let count = signal(0);
        (count, text(move || format!("count {}", count.get())))
    });
    screen.memory_warning();
    count.set(1);
    screen.run_until_idle();
    screen.get_by_text("count 1");
    let noted = screen
        .trace()
        .iter()
        .any(|turn| turn.notes.iter().any(|n| n.contains("MemoryWarning")));
    assert!(noted, "the warning is in the diagnostics");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn animated_app(tr: Transition) -> (Screen, (Signal<f32>, Animated<f32>)) {
    mount(move || {
        let target = signal(0.0f32);
        let shown = animated(target, tr);
        ((target, shown), text(move || format!("{:.1}", shown.get())))
    })
}

#[test]
fn interrupted_tween_continues_from_the_current_value() {
    let (screen, (target, shown)) = animated_app(Transition::tween(100, Easing::Linear));
    target.set(100.0);
    screen.run_until_idle();
    assert!(shown.is_running());
    screen.advance(ms(50));
    let mid = shown.get_untracked();
    assert!((30.0..=55.0).contains(&mid), "{mid}");
    target.set(0.0);
    screen.run_until_idle();
    screen.advance(ms(25));
    let back = shown.get_untracked();
    assert!(
        back < mid && back > 25.0,
        "continued from {mid}, now {back}"
    );
    screen.advance(ms(200));
    assert_eq!(shown.get_untracked(), 0.0);
    assert!(!shown.is_running());
    shown.jump_to(7.0);
    screen.run_until_idle();
    screen.get_by_text("7.0");
}

#[test]
fn reduced_motion_completes_decorative_transitions_only() {
    let (screen, (target, decorative)) = animated_app(Transition::tween(100, Easing::EaseOut));
    screen.set_reduced_motion(true);
    target.set(10.0);
    screen.run_until_idle();
    assert_eq!(decorative.get_untracked(), 10.0, "decorative: instant");
    let (screen2, (target2, info)) =
        animated_app(Transition::tween(100, Easing::Linear).decorative(false));
    screen2.set_reduced_motion(true);
    target2.set(10.0);
    screen2.run_until_idle();
    screen2.advance(ms(50));
    let v = info.get_untracked();
    assert!(
        v > 0.0 && v < 10.0,
        "informational animation still runs: {v}"
    );
    // frame loops are never stopped by reduced motion
    let (screen3, (fl, ticks)) = counter_app();
    screen3.set_reduced_motion(true);
    fl.start(FrameRate::Hz(30.0));
    screen3.advance(Duration::from_secs(1));
    assert_eq!(ticks.get(), 30);
}

#[test]
fn enabling_reduced_motion_finishes_running_decorative_values_once() {
    for transition in [
        Transition::tween(500, Easing::Linear).delay(100),
        Transition::spring(Spring::GENTLE),
        Transition::keyframes([(0.0, 0.0f32), (0.5, 5.0), (1.0, 10.0)], 500)
            .repeat(None)
            .alternate(),
    ] {
        let completions = Rc::new(Cell::new(0));
        let out = completions.clone();
        let (screen, (target, shown)) = mount(move || {
            let target = signal(0.0f32);
            let shown = animated(target, transition).on_complete(move || out.set(out.get() + 1));
            ((target, shown), text(move || format!("{}", shown.get())))
        });
        target.set(10.0);
        screen.run_until_idle();
        screen.advance(ms(50));
        assert!(shown.is_running());
        assert_ne!(shown.get_untracked(), 10.0);
        screen.set_reduced_motion(true);
        assert_eq!(
            shown.get_untracked(),
            10.0,
            "existing run completes without another frame"
        );
        assert!(!shown.is_running());
        assert_eq!(completions.get(), 1);
        screen.advance(ms(100));
        screen.set_reduced_motion(false);
        screen.advance(ms(100));
        assert_eq!(
            shown.get_untracked(),
            10.0,
            "turning reduction off does not restart an old run"
        );
        assert_eq!(completions.get(), 1);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn reduced_motion_change_preserves_essential_and_explicitly_paused_values() {
    let (screen, (target, essential, paused)) = mount(|| {
        let target = signal(0.0f32);
        let essential = animated(
            target,
            Transition::tween(500, Easing::Linear).decorative(false),
        );
        let paused = animated(target, Transition::tween(500, Easing::Linear));
        (
            (target, essential, paused),
            column((
                text(move || format!("{}", essential.get())),
                text(move || format!("{}", paused.get())),
            )),
        )
    });
    target.set(10.0);
    screen.run_until_idle();
    screen.advance(ms(50));
    paused.pause();
    screen.run_until_idle();
    let before = (essential.get_untracked(), paused.get_untracked());
    screen.set_reduced_motion(true);
    assert_eq!((essential.get_untracked(), paused.get_untracked()), before);
    screen.advance(ms(50));
    assert!(essential.get_untracked() > before.0 && essential.get_untracked() < 10.0);
    assert_eq!(paused.get_untracked(), before.1);
    paused.resume();
    screen.run_until_idle();
    assert_eq!(
        paused.get_untracked(),
        10.0,
        "resume observes the current reduced-motion preference"
    );
    screen.advance(ms(500));
    assert_eq!(essential.get_untracked(), 10.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn spring_settles_on_target() {
    let (screen, (target, shown)) = animated_app(Transition::spring(Spring::GENTLE));
    target.set(1.0);
    screen.advance(ms(100));
    let early = shown.get_untracked();
    assert!(early > 0.0 && early < 1.0, "{early}");
    screen.advance(Duration::from_secs(3));
    assert_eq!(shown.get_untracked(), 1.0);
    assert!(!shown.is_running());
}

#[test]
fn keyframes_delay_and_repeat() {
    let tr = Transition::keyframes([(0.0, 0.0f32), (0.5, 100.0), (1.0, 50.0)], 100).delay(20);
    let (screen, (target, shown)) = animated_app(tr);
    target.set(50.0);
    screen.run_until_idle();
    screen.advance(ms(10));
    assert_eq!(shown.get_untracked(), 0.0, "still in the delay");
    screen.advance(ms(60)); // 50 ms into the keyframes
    let peak = shown.get_untracked();
    assert!(peak > 80.0, "{peak}");
    screen.advance(ms(200));
    assert_eq!(shown.get_untracked(), 50.0);
    let tr = Transition::tween(100, Easing::Linear)
        .repeat(Some(2))
        .alternate();
    let (screen, (target, shown)) = animated_app(tr);
    target.set(100.0);
    screen.run_until_idle();
    screen.advance(ms(150)); // second iteration runs backwards
    let v = shown.get_untracked();
    assert!(v > 30.0 && v < 70.0, "{v}");
    screen.advance(ms(200));
    assert_eq!(shown.get_untracked(), 100.0);
}

#[test]
fn unmounting_during_an_animation_leaves_nothing_running() {
    let (screen, (open, target)) = mount(|| {
        let open = signal(true);
        let target = signal(0.0f32);
        (
            (open, target),
            show(open, move || {
                let a = animated(target, Transition::tween(1000, Easing::Linear));
                let fl = frame_loop(|_| {});
                fl.start(FrameRate::Display);
                text(move || format!("{:.0}", a.get()))
            }),
        )
    });
    target.set(100.0);
    screen.advance(ms(100));
    assert!(screen.stats().animations == 1 && screen.stats().frame_loops == 1);
    open.set(false);
    screen.run_until_idle();
    let s = screen.stats();
    assert_eq!((s.animations, s.frame_loops), (0, 0));
    let ticks = screen.frame_ticks();
    screen.advance(Duration::from_secs(1));
    assert!(
        screen.frame_ticks() <= ticks + 1,
        "frames stop once nothing needs them"
    );
}

#[test]
fn style_paint_transitions_use_the_same_animation() {
    let (screen, dim) = mount(|| {
        let dim = signal(false);
        (
            dim,
            node()
                .style(move || {
                    style()
                        .size(10, 10)
                        .background(Color::BLACK)
                        .opacity(if dim.get() { 0.0 } else { 1.0 })
                        .transition(
                            TransitionProp::Paint,
                            Transition::tween(100, Easing::Linear),
                        )
                })
                .test_id("fader"),
        )
    });
    let before = screen.stats();
    let resting = screen.screenshot().pixel(5, 5);
    dim.set(true);
    screen.run_until_idle();
    screen.advance(ms(50));
    let mid = screen.stats();
    // The host interpolates paint; the core writes only the new target.
    assert_eq!(mid.set_props, before.set_props + 1);
    assert_eq!(screen.stats().animations, 0);
    let halfway = screen.screenshot().pixel(5, 5);
    assert_ne!(halfway, resting);
    screen.advance(ms(200));
    let done = screen.stats();
    assert_ne!(screen.screenshot().pixel(5, 5), halfway);
    let ticks = screen.frame_ticks();
    screen.advance(ms(200));
    assert_eq!(
        screen.stats().set_props,
        done.set_props,
        "settled: no more paint updates"
    );
    assert_eq!(
        screen.frame_ticks(),
        ticks,
        "settled host motion stops requesting frames"
    );
}
