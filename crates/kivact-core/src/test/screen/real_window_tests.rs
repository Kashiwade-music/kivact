//! Role: Real window and native integration tests.
//!
//! Smoke tests of `Screen` on real windows (the Arvane test driver): real
//! layout, shaping, rendering and accessibility, frames shown in real
//! windows (headless where none can open), a virtual clock, failure
//! artifacts, pooled windows and tests on parallel threads.
//!
//! Set `KIVACT_TEST_HOLD_MS` to keep each window open that long at the end
//! of a test (to look at it, or to read it with UI Automation), and
//! Every settle draws requested frames.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::motion::{FrameInfo, FrameLoop, frame_loop};
use crate::platform;
use kivact_engine::testing::display_stats;
use kivact_runtime::signal;

use super::artifacts::failure_dir;
use super::model::Screen;
use super::support::button_box;
use crate::motion::FrameRate;
use crate::prelude::{IntoView, NodeBuilder, Role, column, node, node_ref, row, show, style, text};
use crate::test::{MirrorMode, RenderOptions, render, render_with};

fn counter() -> impl IntoView {
    let count = signal(1);
    let draft = signal(String::new());
    column((
        text(move || format!("count: {}", count.get())),
        row((
            button_box("-").on_press(move || count.update(|c| *c -= 1)),
            button_box("+").on_press(move || count.update(|c| *c += 1)),
        ))
        .style(style().gap(8.0)),
        super::support::editor("draft", draft),
        text(move || format!("draft: {}", draft.get())),
    ))
    .style(style().gap(8.0).padding(16.0))
}

fn small(mirror: MirrorMode) -> RenderOptions {
    RenderOptions {
        width: 400.0,
        height: 300.0,
        mirror,
        ..RenderOptions::default()
    }
}

#[test]
fn a_counter_is_clicked_typed_into_and_composed_in_a_real_window() {
    let t = Instant::now();
    let screen = render_with(small(MirrorMode::Auto), counter);
    let mount_time = t.elapsed();
    let turns = screen.stats().turns;
    let t = Instant::now();
    screen.get_by_role(Role::Button, "+").click();
    let click_time = t.elapsed();
    screen.get_by_text("count: 2");
    assert_eq!(
        screen.stats().turns - turns,
        1,
        "a click costs only the turn of its input events"
    );

    let field = screen.get_by_role(Role::TextBox, "draft");
    field.click();
    field.type_text("hi there");
    field.compose("にほ");
    field.type_text(" 日本");
    screen.get_by_text("draft: hi there 日本");
    assert_eq!(field.value().as_deref(), Some("hi there 日本"));
    assert!(
        screen.with_host(|h| h.ime_area()).is_some(),
        "the IME follows the caret"
    );

    let shot = screen.screenshot();
    assert_eq!((shot.width, shot.height), (400, 300));
    assert!(
        shot.pixels.iter().any(|p| *p != 0x00ff_ffff),
        "something but the background was drawn"
    );
    let plus = screen.get_by_role(Role::Button, "+").screenshot();
    let r = screen
        .get_by_role(Role::Button, "+")
        .rect()
        .expect("laid out");
    assert_eq!((plus.width, plus.height), (r.width as u32, r.height as u32));
    let _ = (mount_time, click_time, screen.has_window());
}

#[test]
fn frames_are_deterministic_and_headless_frames_match_real_ones() {
    let label = || {
        column(text("Hello, Kivact! 日本").style(style().font_size(20.0)))
            .style(style().padding(8.0))
    };
    let a = render_with(small(MirrorMode::Auto), label);
    let b = render_with(small(MirrorMode::Auto), label);
    let c = render_with(small(MirrorMode::Off), label);
    assert!(!c.has_window());
    assert_eq!(a.screenshot().hash(), b.screenshot().hash());
    assert_eq!(a.screenshot().hash(), c.screenshot().hash());
    // Text is drawn: a frame without it differs.
    let empty = render_with(small(MirrorMode::Off), || {
        column(()).style(style().padding(8.0))
    });
    assert_ne!(a.screenshot().hash(), empty.screenshot().hash());
}

#[test]
fn virtual_time_drives_sleep_and_frame_loops_exactly() {
    let screen = render(|| {
        let msg = signal("waiting".to_string());
        column((
            text(move || msg.get()),
            button_box("go").on_press(move || {
                crate::task::spawn(async move {
                    crate::task::sleep(Duration::from_millis(500)).await;
                    msg.set("done".into());
                });
            }),
        ))
    });
    screen.get_by_role(Role::Button, "go").click();
    screen.advance(Duration::from_millis(499));
    screen.get_by_text("waiting");
    screen.advance(Duration::from_millis(1));
    screen.get_by_text("done");

    // Ticks of a frame loop over one virtual second, on a host whose
    // display refreshes every `period` ns.
    let ticks = |hz: f64, period: u64| {
        let count = Rc::new(Cell::new(0u64));
        let slot: Rc<Cell<Option<FrameLoop>>> = Rc::new(Cell::new(None));
        let (c, s) = (count.clone(), slot.clone());
        let screen = render_with(
            RenderOptions {
                frame_period_ns: period,
                ..small(MirrorMode::Off)
            },
            move || {
                s.set(Some(frame_loop(move |_: FrameInfo| c.set(c.get() + 1))));
                text("frames")
            },
        );
        slot.take().expect("the loop").start(FrameRate::Hz(hz));
        screen.advance(Duration::from_secs(1));
        assert_eq!(screen.frame_ticks(), count.get());
        count.get()
    };
    let hz60 = 16_666_667;
    assert_eq!(ticks(30.0, hz60), 30);
    assert_eq!(ticks(60.0, hz60), 60, "a loop at the display rate keeps up");
    assert_eq!(
        ticks(120.0, hz60),
        60,
        "the display rate caps a faster loop"
    );
    assert_eq!(ticks(120.0, 8_333_333), 120);

    // A display-rate loop keeps the display's rate when the host wakes
    // late: a tick is stamped with its due time, so the wake latency does
    // not add to every period (the loop asks for its next tick a period
    // after the last one's stamp).
    let times: Rc<std::cell::RefCell<Vec<Duration>>> = Rc::default();
    let slot: Rc<Cell<Option<FrameLoop>>> = Rc::new(Cell::new(None));
    let (t, s) = (times.clone(), slot.clone());
    let screen = render_with(small(MirrorMode::Off), move || {
        s.set(Some(frame_loop(move |info: FrameInfo| {
            t.borrow_mut().push(info.now);
        })));
        text("frames")
    });
    slot.take().expect("the loop").start(FrameRate::Display);
    screen.run_until_idle();
    for _ in 0..30 {
        screen.with_host(|host| {
            let due = host.next_deadline().expect("the loop asks for a frame");
            host.set_now(due + 2_000_000);
            host.wake();
        });
        screen.run_until_idle();
    }
    let times = times.borrow();
    assert!(times.len() >= 29, "{} ticks", times.len());
    for pair in times.windows(2) {
        assert_eq!(
            pair[1] - pair[0],
            Duration::from_nanos(16_666_667),
            "late wakes do not stretch the period: {times:?}"
        );
    }

    // Waiting in virtual time.
    let screen = render(|| {
        let shown = signal(false);
        crate::task::spawn(async move {
            crate::task::sleep(Duration::from_millis(300)).await;
            shown.set(true);
        });
        show(shown, || text("late"))
    });
    let waited = screen.wait_for(Duration::from_secs(1), |s| {
        s.query_by_text("late").is_some()
    });
    assert!(
        (Duration::from_millis(300)..Duration::from_millis(320)).contains(&waited),
        "{waited:?}"
    );
}

#[test]
fn a_display_rate_loop_follows_the_window_to_a_faster_display() {
    let count = Rc::new(Cell::new(0u64));
    let slot: Rc<Cell<Option<FrameLoop>>> = Rc::new(Cell::new(None));
    let (c, s) = (count.clone(), slot.clone());
    let screen = render_with(small(MirrorMode::Off), move || {
        s.set(Some(frame_loop(move |_: FrameInfo| c.set(c.get() + 1))));
        text("frames")
    });
    slot.take().expect("the loop").start(FrameRate::Display);
    screen.advance(Duration::from_secs(1));
    let at60 = count.replace(0);
    assert!((59..=61).contains(&at60), "{at60} ticks at 60 Hz");
    // The window moves to a 120 Hz monitor.
    screen.with_host(|host| host.set_display_period(8_333_333));
    screen.run_until_idle();
    count.set(0);
    screen.advance(Duration::from_secs(1));
    let at120 = count.get();
    assert!((119..=121).contains(&at120), "{at120} ticks at 120 Hz");
}

#[test]
fn a_minimized_window_asks_for_no_redraws_until_it_is_restored() {
    let screen = render_with(small(MirrorMode::Off), counter);
    screen.with_host(|host| {
        host.redraw();
    });
    screen.lose_surface();
    screen.get_by_role(Role::Button, "+").click();
    // A redraw without a surface draws nothing and leaves the paint dirty:
    // asking for one would spin the event loop while minimized.
    screen.with_host(|host| {
        host.redraw();
    });
    screen.run_until_idle();
    assert!(!screen.with_host(|host| host.redraw_requested()));
    screen.restore_surface();
    assert!(screen.with_host(|host| host.redraw_requested()));
    screen.get_by_text("count: 2");
}

#[test]
fn the_environment_and_platform_requests_are_the_tests() {
    let screen = render_with(small(MirrorMode::Auto), || {
        let status = signal("idle".to_string());
        column((
            text(move || format!("reduced motion: {}", platform::reduced_motion())),
            text(move || format!("scale: {}", platform::scale_factor())),
            text(move || status.get()),
            button_box("copy").on_press(move || {
                crate::task::spawn(async move {
                    let result = platform::copy_text("copied").await;
                    status.set(match result {
                        Ok(()) => "copy ok".into(),
                        Err(e) => format!("copy failed: {}", e.message()),
                    });
                });
            }),
        ))
    });
    screen.get_by_text("reduced motion: false");
    screen.set_reduced_motion(true);
    screen.get_by_text("reduced motion: true");

    screen.set_scale(2.0);
    screen.get_by_text("scale: 2");
    let shot = screen.screenshot();
    assert_eq!((shot.width, shot.height), (800, 600));
    screen.resize(300.0, 200.0);
    let shot = screen.screenshot();
    assert_eq!((shot.width, shot.height), (600, 400));
    screen.set_dark(true);

    screen.fail_next_platform_request("no clipboard today");
    screen.get_by_role(Role::Button, "copy").click();
    screen.get_by_text("copy failed: no clipboard today");
    screen.get_by_role(Role::Button, "copy").click();
    screen.get_by_text("copy ok");
    assert_eq!(screen.clipboard().as_deref(), Some("copied"));

    assert!(screen.request_close(), "a close request ends the App");
}

fn modal_screen() -> Screen {
    render(|| {
        let open = signal(true);
        column((
            button_box("disabled").disabled(true),
            button_box("behind"),
            node().test_id("empty"),
            show(open, move || {
                // Without an anchor the modal is centred in the window.
                crate::view::overlay(
                    node_ref(),
                    crate::view::Placement::Center,
                    button_box("close").on_press(move || open.set(false)),
                )
                .modal(true)
            }),
        ))
    })
}

#[test]
#[should_panic(expected = "is behind the backdrop of the modal")]
fn clicking_behind_a_modal_fails() {
    let screen = modal_screen();
    screen.get_by_role(Role::Button, "behind").click();
}

#[test]
#[should_panic(expected = "is disabled")]
fn clicking_a_disabled_button_fails() {
    let screen = modal_screen();
    screen.get_by_role(Role::Button, "close").click();
    screen.get_by_role(Role::Button, "disabled").click();
}

#[test]
#[should_panic(expected = "has no size")]
fn clicking_an_element_without_size_fails() {
    let screen = modal_screen();
    screen.get_by_test_id("empty").click();
}

#[test]
#[should_panic(expected = "not exposed to assistive technology")]
fn focusing_an_element_hidden_from_assistive_technology_fails() {
    let screen = modal_screen();
    screen.get_by_test_id("empty").focus();
}

#[test]
fn failures_leave_artifacts_windows_are_pooled_and_tests_run_in_parallel() {
    let dir = failure_dir();
    let failed = std::thread::Builder::new()
        .name("kivact-test-failure-probe".into())
        .spawn(|| {
            let screen = render(counter);
            screen.get_by_role(Role::Button, "does not exist");
        })
        .expect("a thread")
        .join();
    let message = failed
        .expect_err("the query fails")
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default();
    let png = dir.join("kivact-test-failure-probe.png");
    let txt = dir.join("kivact-test-failure-probe.txt");
    assert!(message.contains(&png.display().to_string()), "{message}");
    assert!(std::fs::metadata(&png).expect("a screenshot").len() > 0);
    let tree = std::fs::read_to_string(&txt).expect("the trees");
    assert!(tree.contains("Text \"count: 1\""), "{tree}");

    // Closed windows go back to the pool (when there are windows at all).
    let before = display_stats();
    let windowed = (0..3)
        .map(|_| render(counter).has_window())
        .fold(false, |a, b| a | b);
    if let (true, Some(before), Some(after)) = (windowed, before, display_stats()) {
        assert!(after.reused > before.reused, "{before:?} → {after:?}");
    }

    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(move || {
                for _ in 0..4 {
                    let screen = render(counter);
                    screen.get_by_role(Role::Button, "+").click();
                    screen.get_by_text("count: 2");
                }
            });
        }
    });
}

/// Per-test cost, serial and parallel, real windows and headless:
/// `cargo test -p kivact --features test --lib real_window_tests::cost -- --ignored --nocapture`.
#[test]
#[ignore = "a measurement, not a check"]
fn cost() {
    let one = |mirror: MirrorMode| {
        let t = Instant::now();
        let screen: Screen = render_with(
            RenderOptions {
                mirror,
                ..RenderOptions::default()
            },
            counter,
        );
        let mount_time = t.elapsed();
        let t = Instant::now();
        screen.get_by_role(Role::Button, "+").click();
        screen.get_by_text("count: 2");
        (mount_time, t.elapsed())
    };
    let n: usize = std::env::var("KIVACT_COST_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600);
    let mut report = String::new();
    for window in [MirrorMode::Required, MirrorMode::Off] {
        one(window);
        for threads in [1, 8, 24] {
            let t = Instant::now();
            let per: Vec<(Duration, Duration)> = std::thread::scope(|scope| {
                let handles: Vec<_> = (0..threads)
                    .map(|_| {
                        scope.spawn(move || {
                            (0..n / threads).map(|_| one(window)).collect::<Vec<_>>()
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .flat_map(|h| h.join().expect("a thread"))
                    .collect()
            });
            let count = per.len() as u32;
            let mount: Duration = per.iter().map(|p| p.0).sum();
            let click: Duration = per.iter().map(|p| p.1).sum();
            report.push_str(&format!(
                "{window:?}, {threads} threads: {count} tests in {:?}; per test mount {:?}, click+settle {:?}\n",
                t.elapsed(),
                mount / count,
                click / count
            ));
        }
    }
    report.push_str(&format!("{:?}\n", display_stats()));
    std::fs::write(
        failure_dir().with_file_name("kivact-test-cost.txt"),
        &report,
    )
    .expect("the report");
}
