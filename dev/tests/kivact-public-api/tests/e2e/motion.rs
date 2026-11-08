//! Role: Motion timing: clock, frame loops, animated values and transitions.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use super::paint::{BLACK, BLUE, RED, WHITE, distance};
use crate::support::{mount, mount_with};
use kivact::draw::{Filter, Shadow};
use kivact::motion::FrameRate;
use kivact::motion::{
    Animated, Easing, FrameInfo, FrameLoop, Repeat, Spring, Transition, animated, frame_loop,
};
use kivact::prelude::*;
use kivact::style::{TransitionProp, rgba};
use kivact::test::{RenderOptions, Screen, Screenshot};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn popped_exit_releases_layout_and_finishes_after_the_last_descendant() {
    use kivact::view::ExitLayout;

    for exit_layout in [ExitLayout::Keep, ExitLayout::Pop] {
        let completions = Rc::new(Cell::new(0));
        let count = completions.clone();
        let (screen, shown) = mount(move || {
            let shown = signal(true);
            let finished = kivact::state::callback(move |()| count.set(count.get() + 1));
            let exit = |duration| {
                style().exit(style().opacity(0.0)).transition(
                    TransitionProp::All,
                    Transition::tween(duration, Easing::Linear),
                )
            };
            let panel = row((
                show(shown, move || {
                    node()
                        .role(Role::Group)
                        .test_id("leaving")
                        .exit_layout(exit_layout)
                        .on_exit_complete(finished)
                        .style(
                            exit(80)
                                .size(50, 30)
                                .margin_left(6)
                                .background(rgb(0xff0000)),
                        )
                        .child(node().style(exit(180).size(2, 2).background(rgb(0x0000ff))))
                }),
                node().test_id("sibling").style(style().size(18, 30)),
            ))
            .style(
                style()
                    .gap(4)
                    .padding(10)
                    .border(3, rgb(0x000000))
                    .background(rgb(0xffffff)),
            );
            (shown, panel)
        });
        let sibling_x = || screen.get_by_test_id("sibling").rect().unwrap().x;
        let before = sibling_x();
        let leaving = screen.get_by_test_id("leaving").rect().unwrap();
        let (sample_x, sample_y) = (
            (leaving.x + leaving.width - 3.0) as u32,
            (leaving.y + 12.0) as u32,
        );
        assert_eq!(
            screen.screenshot().pixel(sample_x, sample_y),
            Some(0xff0000)
        );

        shown.set(false);
        screen.run_until_idle();
        assert!(screen.query_by_test_id("leaving").is_none());
        assert_eq!(completions.get(), 0);
        let after = sibling_x();
        if exit_layout == ExitLayout::Pop {
            assert!(after < before - 40.0);
        } else {
            assert_eq!(after, before);
        }
        screen.advance(ms(40));
        // Red continues at its old position while the sibling already moved.
        let still_red = screen.screenshot().pixel(sample_x, sample_y).unwrap();
        assert_eq!(still_red >> 16, 0xff);
        assert!(still_red & 0xffff < 0xffff);

        screen.advance(ms(70));
        assert_eq!(completions.get(), 0, "longer child still leaving");
        screen.advance(ms(110));
        assert_eq!(sibling_x(), leaving.x - 6.0);
        assert_eq!(completions.get(), 1);
        screen.advance(ms(200));
        assert_eq!(completions.get(), 1);
    }
}

#[test]
fn exit_completion_fires_for_instant_removal_and_skips_a_disposed_callback() {
    let completions = Rc::new(Cell::new(0));
    let count = completions.clone();
    let (screen, shown) = mount(move || {
        let shown = signal(true);
        let finished = kivact::state::callback(move |()| count.set(count.get() + 1));
        (
            shown,
            show(shown, move || {
                let disposed =
                    kivact::state::callback::<(), ()>(|()| panic!("removed callback owner ran"));
                (
                    node().on_exit_complete(finished).style(
                        style().exit(style().opacity(0.0)).transition(
                            TransitionProp::All,
                            Transition::tween(400, Easing::Linear),
                        ),
                    ),
                    node().on_exit_complete(disposed),
                )
            }),
        )
    });
    screen.set_reduced_motion(true);
    shown.set(false);
    screen.run_until_idle();
    assert_eq!(completions.get(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn counter_app() -> (Screen, (FrameLoop, Rc<Cell<u64>>)) {
    counter_app_with(RenderOptions::from_env())
}

/// A display refreshing every millisecond: frame loops up to 240 Hz tick
/// at their own rate (a 60 Hz display caps them at 60 Hz, as the host
/// paces ticks at most once per refresh).
fn fast_display() -> RenderOptions {
    RenderOptions::from_env().frame_period_ns(1_000_000)
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

fn animated_app(tr: Transition) -> (Screen, (Signal<f64>, Animated<f64>)) {
    mount(move || {
        let target = signal(0.0f64);
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
        animated_app(Transition::tween(100, Easing::Linear).essential());
    screen2.set_reduced_motion(true);
    target2.set(10.0);
    screen2.run_until_idle();
    screen2.advance(ms(50));
    let v = info.get_untracked();
    assert!(
        v > 0.0 && v < 10.0,
        "informational animation still runs: {v}"
    );
    // A decorative frame loop is held under reduced motion (§12.5, §17.2).
    let (screen3, (fl, ticks)) = counter_app();
    screen3.set_reduced_motion(true);
    fl.start(FrameRate::Hz(30.0));
    screen3.advance(Duration::from_secs(1));
    assert_eq!(ticks.get(), 0);
}

#[test]
fn keyframes_delay_and_repeat() {
    let tr = Transition::keyframes([(0.0, 0.0f64), (0.5, 100.0), (1.0, 50.0)], 100).delay(20);
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
        .repeat(Repeat::Times(2))
        .alternate();
    let (screen, (target, shown)) = animated_app(tr);
    target.set(100.0);
    screen.run_until_idle();
    screen.advance(ms(150)); // second iteration runs backwards
    let v = shown.get_untracked();
    assert!(v > 30.0 && v < 70.0, "{v}");
    screen.advance(ms(200));
    // Two alternating iterations end where they began (§12.3).
    assert_eq!(shown.get_untracked(), 0.0);
}

/// `on_complete` runs when a run reaches its target, never for a run that a new target or a
/// `jump_to` ends.
#[test]
fn on_complete_runs_only_when_a_run_reaches_its_target() {
    let completions = Rc::new(Cell::new(0u32));
    let c2 = completions.clone();
    let (screen, (target, shown)) = mount(move || {
        let target = signal(0.0f64);
        let c3 = c2.clone();
        let shown = animated(target, Transition::tween(100, Easing::Linear))
            .on_complete(move || c3.set(c3.get() + 1));
        ((target, shown), text(move || format!("{:.1}", shown.get())))
    });
    target.set(100.0);
    screen.run_until_idle();
    screen.advance(ms(200));
    assert_eq!(completions.get(), 1, "the run reached 100");
    target.set(0.0);
    screen.run_until_idle();
    screen.advance(ms(50));
    target.set(90.0);
    screen.run_until_idle();
    screen.advance(ms(300));
    assert_eq!(completions.get(), 2, "only the run toward 90 completed");
    target.set(20.0);
    screen.run_until_idle();
    screen.advance(ms(50));
    shown.jump_to(10.0);
    screen.run_until_idle();
    screen.advance(ms(200));
    assert_eq!(completions.get(), 2, "a jump cancels the run");
    assert!(!shown.is_running());
    assert_eq!(shown.get_untracked(), 10.0);
}

/// Zero iterations end at once on the target; a spring repeats like a tween.
#[test]
fn zero_iterations_end_at_once_and_springs_repeat() {
    let (screen, (target, shown)) =
        animated_app(Transition::tween(100, Easing::Linear).repeat(Repeat::Times(0)));
    target.set(100.0);
    screen.run_until_idle();
    screen.advance(ms(16));
    assert_eq!(
        shown.get_untracked(),
        100.0,
        "no iteration: at the target at once"
    );

    // One spring run: count the 50 ms steps until it settles.
    let (screen, (target, shown)) = animated_app(Transition::spring(Spring::STIFF));
    target.set(100.0);
    screen.run_until_idle();
    let mut once = 0;
    while shown.get_untracked() != 100.0 {
        screen.advance(ms(50));
        once += 1;
        assert!(once < 200, "the spring settles");
    }
    // Two iterations: after one run's time it has restarted and is not at the target.
    let (screen, (target, shown)) =
        animated_app(Transition::spring(Spring::STIFF).repeat(Repeat::Times(2)));
    target.set(100.0);
    screen.run_until_idle();
    for _ in 0..once {
        screen.advance(ms(50));
    }
    assert!(
        shown.get_untracked() < 100.0,
        "the second iteration started from 0"
    );
    for _ in 0..(once + 2) {
        screen.advance(ms(50));
    }
    assert_eq!(shown.get_untracked(), 100.0, "both iterations ended");
}

#[test]
fn reactive_text_color_transitions_on_inheriting_children() {
    let (screen, switched) = mount_with(fast_display(), || {
        let switched = signal(false);
        let ink = move || {
            if switched.get() {
                rgb(0x0000ff)
            } else {
                rgb(0xff0000)
            }
        };
        let child = text("MMMM").test_id("inherited-ink");
        let explicit = text("MMMM")
            .test_id("explicit-ink")
            .style(style().color(rgb(0x008000)));
        (
            switched,
            row((child, explicit)).style(move || {
                style()
                    .font_size(48)
                    .weight(700)
                    .color(ink())
                    .transition(TransitionProp::All, Transition::tween(200, Easing::Linear))
            }),
        )
    });

    let inherited = screen.get_by_test_id("inherited-ink");
    let explicit = screen.get_by_test_id("explicit-ink");
    let first = inherited.screenshot();
    let at = first
        .pixels
        .iter()
        .position(|&pixel| {
            let (r, g, b) = ((pixel >> 16) & 255, (pixel >> 8) & 255, pixel & 255);
            r > 220 && g < 40 && b < 40
        })
        .expect("a fully covered red glyph pixel");
    let explicit_before = explicit.screenshot();

    switched.set(true);
    screen.run_until_idle();
    screen.advance(ms(95));
    let middle = inherited.screenshot().pixels[at];
    assert!((middle >> 16) & 255 > 15 && middle & 255 > 15);
    assert_eq!(explicit.screenshot().pixels, explicit_before.pixels);
    screen.advance(ms(160));
    let settled = inherited.screenshot().pixels[at];
    assert!(settled & 255 > 220 && (settled >> 16) & 255 < 40);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn layout_transition_moves_from_the_previous_position() {
    let (screen, (expanded, presses)) = mount(move || {
        let expanded = signal(false);
        let presses = signal(0);
        let content = row((
            node().style(move || style().size(if expanded.get() { 240 } else { 40 }, 40)),
            button("Moving")
                .test_id("moving")
                .on_press(move || presses.update(|count| *count += 1))
                .style(move || {
                    style()
                        .size(if expanded.get() { 160 } else { 80 }, 40)
                        .background(rgb(0xff0000))
                        .layout_transition(Transition::tween(400, Easing::Linear))
                }),
        ));
        ((expanded, presses), content)
    });
    let moving = screen.get_by_test_id("moving");
    let before = moving.rect().unwrap();
    expanded.set(true);
    screen.run_until_idle();
    let start = moving.rect().unwrap();
    screen.advance(ms(160));
    let midway = moving.rect().unwrap();
    assert!(midway.x > start.x && midway.x < before.x + 200.0);
    assert!(midway.width > start.width && midway.width < 160.0);
    let painted = screen
        .screenshot()
        .pixel((midway.x + 8.0) as u32, (midway.y + 20.0) as u32)
        .unwrap();
    assert!(
        (painted >> 16) & 255 > 200 && painted & 255 < 80,
        "paint follows the displayed rectangle"
    );
    screen.click_at(
        midway.x + midway.width / 2.0,
        midway.y + midway.height / 2.0,
    );
    assert_eq!(
        presses.get_untracked(),
        1,
        "hit testing follows the displayed rectangle"
    );
    screen.advance(ms(500));
    let end = moving.rect().unwrap();
    assert!(start.x < midway.x && midway.x < end.x);
    assert!(end.x > before.x + 150.0);
    assert!(end.width > midway.width);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn proportional_retarget_shortens_the_remaining_time() {
    use kivact::motion::Retarget;

    let (screen, (target, proportional, restart)) = mount(move || {
        let target = signal(0.0f64);
        let timing = Transition::tween(800, Easing::Linear);
        let proportional = animated(target, timing.clone().retarget(Retarget::Proportional));
        let restart = animated(target, timing.clone());
        let proportional_timing = timing.clone().retarget(Retarget::Proportional);
        let paint_proportional = proportional_timing.clone();
        let paint_restart = timing.clone();
        let content = column((
            node().test_id("proportional-layout").style(move || {
                style()
                    .size(target.get() + 20.0, 20)
                    .layout_transition(proportional_timing.clone())
            }),
            node().test_id("restart-layout").style(move || {
                style()
                    .size(target.get() + 20.0, 20)
                    .layout_transition(timing.clone())
            }),
            node().test_id("proportional-paint").style(move || {
                style()
                    .size(20, 20)
                    .translate_x(target.get())
                    .transition(TransitionProp::All, paint_proportional.clone())
            }),
            node().test_id("restart-paint").style(move || {
                style()
                    .size(20, 20)
                    .translate_x(target.get())
                    .transition(TransitionProp::All, paint_restart.clone())
            }),
        ));
        ((target, proportional, restart), content)
    });
    screen.advance(ms(1000));
    target.set(100.0);
    screen.run_until_idle();
    screen.advance(ms(320));
    let current = proportional.get_untracked();
    assert!(current > 0.0 && current < 100.0);
    target.set(current + 10.0);
    screen.run_until_idle();
    screen.advance(ms(220));
    let proportional_width = screen
        .get_by_test_id("proportional-layout")
        .rect()
        .unwrap()
        .width;
    let restart_width = screen
        .get_by_test_id("restart-layout")
        .rect()
        .unwrap()
        .width;
    assert!(
        proportional_width > restart_width + 1.0,
        "layout retarget shares the timing policy"
    );
    let proportional_x = screen
        .get_by_test_id("proportional-paint")
        .rect()
        .unwrap()
        .x;
    let restart_x = screen.get_by_test_id("restart-paint").rect().unwrap().x;
    assert!(
        proportional_x > restart_x + 1.0,
        "paint retarget shares the timing policy"
    );
    assert!(
        !proportional.is_running(),
        "the short remaining distance finishes sooner"
    );
    assert!(
        restart.is_running(),
        "restart keeps the configured duration"
    );
    screen.advance(ms(900));
    assert!(!restart.is_running());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

const YELLOW: u32 = 0xffff00;

/// A white square panel of `size` px with `pad` px around `subject`.
fn white_panel(size: f64, pad: f64, subject: Node) -> Node {
    node()
        .style(style().size(size, size).padding(pad).background(rgb(WHITE)))
        .child(subject)
}

/// A 100 × 100 white panel holding `subject` at (20, 20).
fn panel(subject: Node) -> Node {
    white_panel(100.0, 20.0, subject)
}

/// The panel's pixels, at the window origin so crops line up across screens.
fn panel_shot(screen: &Screen) -> Screenshot {
    screen.screenshot().crop(0, 0, 100, 100)
}

/// Tier 2 (design §3.3): a compositor result matches within ±2/255 per channel.
fn assert_compositor_close(got: &Screenshot, want: &Screenshot) {
    assert_eq!((got.width, got.height), (want.width, want.height));
    let worst = got
        .pixels
        .iter()
        .zip(&want.pixels)
        .map(|(a, b)| distance(*a, *b))
        .max()
        .unwrap_or(0);
    assert!(
        worst <= 2,
        "largest channel difference {worst}/255 is over 2"
    );
}

/// Tier 3 (design §3.3): a raster (box shadow) within the raster tolerance.
fn assert_raster_close(got: &Screenshot, want: &Screenshot) {
    let diff = got.raster_difference(want);
    assert!(diff.within_raster_tolerance(), "{diff:?}");
}

/// The centre pixel of the element `test_id` (0xRRGGBB).
fn center_pixel(screen: &Screen, test_id: &str) -> u32 {
    let rect = screen
        .get_by_test_id(test_id)
        .rect()
        .expect("the element is laid out");
    screen
        .screenshot()
        .pixel(
            (rect.x + rect.width / 2.0) as u32,
            (rect.y + rect.height / 2.0) as u32,
        )
        .expect("in the frame")
}

/// A panel whose subject is `subject`'s filter `from` until the signal turns true, then `to`.
/// The change is a 100 ms linear transition of the filter.
fn filter_scene(fill: u32, from: Filter, to: Filter) -> (Screen, Signal<bool>) {
    mount_with(fast_display(), move || {
        let on = signal(false);
        let subject = node().test_id("subject").style(move || {
            let filter = if on.get() { to.clone() } else { from.clone() };
            style()
                .size(40, 40)
                .background(rgb(fill))
                .filter(filter)
                .transition(
                    TransitionProp::Filter,
                    Transition::tween(100, Easing::Linear),
                )
        });
        (on, panel(subject))
    })
}

/// The reference: a panel whose subject has `filter` as its static style.
fn still_panel(fill: u32, filter: Filter) -> Screenshot {
    let (screen, ()) = mount(move || {
        let subject = node().style(style().size(40, 40).background(rgb(fill)).filter(filter));
        ((), panel(subject))
    });
    panel_shot(&screen)
}

/// The per-property transition: the colour changes at once, the opacity takes 100 ms.
/// Tier 2 (design §3.3): the opacity is a compositor op, checked against its blend with white.
#[test]
fn per_property_transition() {
    let (screen, on) = mount_with(fast_display(), || {
        let on = signal(false);
        let subject = node().test_id("subject").style(move || {
            let on = on.get();
            style()
                .size(60, 60)
                .background(rgb(if on { BLUE } else { RED }))
                .opacity(if on { 0.0 } else { 1.0 })
                .transition(
                    TransitionProp::Opacity,
                    Transition::tween(100, Easing::Linear),
                )
        });
        (on, panel(subject))
    });
    assert!(distance(center_pixel(&screen, "subject"), RED) <= 2);
    on.set(true);
    screen.run_until_idle();
    let at_once = center_pixel(&screen, "subject");
    assert!(
        distance(at_once, BLUE) <= 2,
        "the colour changed at once: {at_once:06x}"
    );
    screen.advance(ms(50));
    // Blue at half opacity over white (0.5 × 0x0000ff + 0.5 × 0xffffff).
    let half = center_pixel(&screen, "subject");
    assert!(distance(half, 0x8080ff) <= 2, "half opacity: {half:06x}");
    screen.advance(ms(100));
    let gone = center_pixel(&screen, "subject");
    assert!(distance(gone, WHITE) <= 2, "no opacity left: {gone:06x}");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A blur to none, and a brightness to none, at 50 % with linear easing (CSS initial
/// values: blur 0, brightness 1). Tier 2 (design §3.3): filters are compositor ops.
#[test]
fn filter_to_none_interpolates() {
    const FILL: u32 = 0x404040;
    let (screen, on) = filter_scene(FILL, Filter::new().blur(4), Filter::new());
    on.set(true);
    screen.run_until_idle();
    screen.advance(ms(50));
    assert_compositor_close(
        &panel_shot(&screen),
        &still_panel(FILL, Filter::new().blur(2)),
    );
    screen.advance(ms(100));
    assert_compositor_close(&panel_shot(&screen), &still_panel(FILL, Filter::new()));

    let (screen, on) = filter_scene(FILL, Filter::new().brightness(2), Filter::new());
    on.set(true);
    screen.run_until_idle();
    screen.advance(ms(50));
    assert_compositor_close(
        &panel_shot(&screen),
        &still_panel(FILL, Filter::new().brightness(1.5)),
    );
    screen.advance(ms(100));
    assert_compositor_close(&panel_shot(&screen), &still_panel(FILL, Filter::new()));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// None to grayscale(1) at 50 % is grayscale(0.5) (the initial value 0 of §12.9.2).
/// Tier 2 (design §3.3): a filter is a compositor op.
#[test]
fn filter_identity_values() {
    const FILL: u32 = 0x2060c0;
    let (screen, on) = filter_scene(FILL, Filter::new(), Filter::new().grayscale(1));
    on.set(true);
    screen.run_until_idle();
    screen.advance(ms(50));
    assert_compositor_close(
        &panel_shot(&screen),
        &still_panel(FILL, Filter::new().grayscale(0.5)),
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// An outer shadow with an inset one, to the outer shadow alone: at 50 % the inset is
/// black at half alpha, still inset. Tier 3 (design §3.3): box shadows are Vello rasters.
#[test]
fn shadow_list_shortening_interpolates() {
    fn outer() -> Shadow {
        Shadow::new(0, 0, 0, rgb(BLUE)).spread(8)
    }
    fn inset(alpha: f64) -> Shadow {
        Shadow::new(0, 0, 0, rgba(BLACK, alpha)).spread(6).inset()
    }
    // Halfway to the padding (spread 0, transparent): spread 3 at alpha 0.5 (§12.9.2).
    fn half_inset() -> Shadow {
        Shadow::new(0, 0, 0, rgba(BLACK, 0.5)).spread(3).inset()
    }
    fn subject_with(shadows: Vec<Shadow>) -> Node {
        let mut s = style().size(60, 60).background(rgb(WHITE));
        for shadow in shadows {
            s = s.shadow(shadow);
        }
        node().test_id("subject").style(s)
    }
    let (screen, on) = mount_with(fast_display(), move || {
        let on = signal(false);
        let subject = node().test_id("subject").style(move || {
            let s = style().size(60, 60).background(rgb(WHITE)).shadow(outer());
            let s = if on.get() { s } else { s.shadow(inset(1.0)) };
            s.transition(
                TransitionProp::Shadow,
                Transition::tween(100, Easing::Linear),
            )
        });
        (on, panel(subject))
    });
    on.set(true);
    screen.run_until_idle();
    screen.advance(ms(50));
    let still = mount(move || ((), panel(subject_with(vec![outer(), half_inset()]))));
    assert_raster_close(&panel_shot(&screen), &panel_shot(&still.0));
    screen.advance(ms(100));
    let end = mount(move || ((), panel(subject_with(vec![outer()]))));
    assert_raster_close(&panel_shot(&screen), &panel_shot(&end.0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The blur edge probe (5 px outside the left edge of a black box): a larger value is darker.
fn blur_probe(reverse: bool) -> Vec<u32> {
    let (screen, on) = mount_with(fast_display(), || {
        let on = signal(false);
        let subject = node().test_id("subject").style(move || {
            let blur = if on.get() { 10.0 } else { 0.0 };
            style()
                .size(60, 60)
                .background(rgb(BLACK))
                .filter(Filter::new().blur(blur))
                .transition(TransitionProp::Filter, Transition::spring(Spring::STIFF))
        });
        (on, white_panel(200.0, 70.0, subject))
    });
    let darkness = |screen: &Screen| {
        let rect = screen
            .get_by_test_id("subject")
            .rect()
            .expect("the subject is laid out");
        let shot = screen.screenshot();
        let x = (rect.x - 5.0) as u32;
        let y = (rect.y + rect.height / 2.0) as u32;
        255 - ((shot.pixel(x, y).expect("the pixel is inside the shot") >> 16) & 255)
    };
    on.set(true);
    screen.run_until_idle();
    screen.advance(ms(100));
    if reverse {
        on.set(false);
        screen.run_until_idle();
    }
    let mut samples = vec![darkness(&screen)];
    for _ in 0..4 {
        screen.advance(ms(1));
        samples.push(darkness(&screen));
    }
    samples
}

/// A spring blur reversed at about 5 px: the first frames after the reversal keep the
/// velocity of the uninterrupted spring (the same spring, unreversed, is the control).
/// Tier 2 (design §3.3): a blur is a compositor op, read as the dark level of one pixel.
#[test]
fn reversed_blur_keeps_its_velocity() {
    let control = blur_probe(false);
    let reversed = blur_probe(true);
    assert_eq!(control[0], reversed[0], "identical until the reversal");
    assert!(
        control[4] >= control[0] + 2,
        "the probe moves with the spring: {control:?}"
    );
    for k in 1..=4 {
        let gap = (control[k] as i32 - reversed[k] as i32).abs();
        assert!(
            gap <= 2,
            "frame {k} after the reversal departs from the velocity: {control:?} vs {reversed:?}"
        );
    }
    assert!(
        reversed[4] >= reversed[0] + 2,
        "the blur keeps growing after the reversal: {reversed:?}"
    );
}

/// The black band left of the subject, in px: black is R 0, and red, yellow and white are R 255,
/// so the band's width is the sum of the black coverage of the pixels 1 to 45 px out.
fn black_band(screen: &Screen) -> f64 {
    let rect = screen
        .get_by_test_id("subject")
        .rect()
        .expect("the subject is laid out");
    let shot = screen.screenshot();
    let y = (rect.y + rect.height / 2.0) as u32;
    let x0 = rect.x as u32;
    (1..=45)
        .map(|k| {
            f64::from(
                255 - ((shot.pixel(x0 - k, y).expect("the pixel is inside the shot") >> 16) & 255),
            ) / 255.0
        })
        .sum()
}

/// The pixel `px` outside the subject's left edge.
fn outside_pixel(screen: &Screen, px: u32) -> u32 {
    let rect = screen
        .get_by_test_id("subject")
        .rect()
        .expect("the subject is laid out");
    let y = (rect.y + rect.height / 2.0) as u32;
    screen
        .screenshot()
        .pixel(rect.x as u32 - px, y)
        .expect("in the frame")
}

/// One black shadow of 0 → 20 px, then two more shadows (red 30, yellow 40) to the list:
/// the black band goes on from its shown width and no shadow jumps to its target.
/// Tier: band widths are shape measures (±1 px); the frame stays exact in its colours.
#[test]
fn retarget_across_shadow_counts_starts_from_shown() {
    fn band(color: u32, spread: f64) -> Shadow {
        Shadow::new(0, 0, 0, rgb(color)).spread(spread)
    }
    let (screen, stage) = mount_with(fast_display(), || {
        let stage = signal(0u8);
        let subject = node().test_id("subject").style(move || {
            let s = style().size(60, 60).background(rgb(WHITE)).transition(
                TransitionProp::Shadow,
                Transition::tween(200, Easing::Linear),
            );
            match stage.get() {
                0 => s.shadow(band(BLACK, 0.0)),
                1 => s.shadow(band(BLACK, 20.0)).shadow(band(RED, 30.0)),
                _ => s
                    .shadow(band(BLACK, 20.0))
                    .shadow(band(RED, 30.0))
                    .shadow(band(YELLOW, 40.0)),
            }
        });
        (stage, white_panel(200.0, 70.0, subject))
    });
    stage.set(1);
    screen.run_until_idle();
    screen.advance(ms(100));
    let before = black_band(&screen);
    assert!((before - 10.0).abs() <= 1.0, "halfway to 20 px: {before}");

    stage.set(2);
    screen.run_until_idle();
    screen.advance(ms(1));
    let after = black_band(&screen);
    assert!(
        (after - before).abs() <= 1.0,
        "the band continues from {before} px: {after}"
    );
    let far = outside_pixel(&screen, 25);
    assert!(
        distance(far, WHITE) <= 2,
        "no new shadow jumped to its target 25 px out: {far:06x}"
    );

    screen.advance(ms(300));
    assert!((black_band(&screen) - 20.0).abs() <= 1.0);
    let yellow = outside_pixel(&screen, 35);
    assert!(
        distance(yellow, YELLOW) <= 2,
        "the new shadow ends at 40 px: {yellow:06x}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A transition interrupted by leaving the hover keeps the shown value: the next frame moves
/// by one frame, and a spring keeps moving forward with its velocity.
/// Tier 1 (design §3.3): positions are layout rectangles.
#[test]
fn interrupted_transition_continues_from_shown() {
    let x_of = |screen: &Screen| {
        screen
            .get_by_test_id("subject")
            .rect()
            .expect("the subject is laid out")
            .x
    };

    // Tween: 200 ms linear. Halfway through, the value is about 25; after leaving, one frame
    // later it has moved about 0.5 px, not back to a restart at the origin.
    let (screen, ()) = mount_with(fast_display(), || {
        let subject = node().test_id("subject").style(
            style()
                .size(300, 40)
                .background(rgb(RED))
                .hover(style().translate_x(100.0))
                .transition(
                    TransitionProp::Transform,
                    Transition::tween(200, Easing::Linear),
                ),
        );
        ((), subject)
    });
    screen.get_by_test_id("subject").hover();
    screen.advance(ms(50));
    let shown = x_of(&screen);
    screen.advance(ms(1));
    let before_leave = x_of(&screen);
    screen.mouse_leave();
    screen.advance(ms(1));
    let after_leave = x_of(&screen);
    assert!(
        (after_leave - before_leave).abs() <= 1.0 && after_leave > shown - 2.0,
        "tween: {shown} then {before_leave} then {after_leave}"
    );

    // Spring: the velocity carries the next frame in the same direction.
    let (screen, ()) = mount_with(fast_display(), || {
        let subject = node().test_id("subject").style(
            style()
                .size(300, 40)
                .background(rgb(RED))
                .hover(style().translate_x(100.0))
                .transition(TransitionProp::Transform, Transition::spring(Spring::STIFF)),
        );
        ((), subject)
    });
    screen.get_by_test_id("subject").hover();
    screen.advance(ms(100));
    let a = x_of(&screen);
    screen.advance(ms(1));
    let b = x_of(&screen);
    screen.mouse_leave();
    screen.advance(ms(1));
    let c = x_of(&screen);
    assert!(b > a, "the spring is moving toward the hover: {a} {b}");
    assert!(
        c > b && ((c - b) - (b - a)).abs() <= 0.5,
        "the spring keeps its velocity after leaving: {a} {b} {c}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Reduced motion holds a decorative frame loop and keeps an essential one running.
#[test]
fn frame_loop_pauses_under_reduced_motion() {
    let decorative_ticks = Rc::new(Cell::new(0u64));
    let essential_ticks = Rc::new(Cell::new(0u64));
    let (d, e) = (decorative_ticks.clone(), essential_ticks.clone());
    let (screen, (decorative, essential)) = mount_with(fast_display(), move || {
        let decorative = frame_loop(move |_f: FrameInfo| d.set(d.get() + 1));
        let essential = frame_loop(move |_f: FrameInfo| e.set(e.get() + 1)).essential();
        ((decorative, essential), text("frames"))
    });
    screen.set_reduced_motion(true);
    decorative.start(FrameRate::Hz(30.0));
    essential.start(FrameRate::Hz(30.0));
    screen.advance(Duration::from_secs(1));
    assert_eq!(decorative_ticks.get(), 0, "the decorative loop is held");
    assert_eq!(
        essential_ticks.get(),
        30,
        "the essential loop runs at 30 Hz"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
