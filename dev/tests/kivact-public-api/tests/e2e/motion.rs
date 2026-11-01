//! Role: Motion timing: clock, frame loops, animated values and transitions.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::support::{mount, mount_with};
use kivact::motion::FrameRate;
use kivact::motion::{Animated, Easing, FrameInfo, FrameLoop, Transition, animated, frame_loop};
use kivact::prelude::*;
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
