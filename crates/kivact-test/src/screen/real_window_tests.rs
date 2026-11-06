//! Role: Real window and native integration tests.
//!
//! Smoke tests of `Screen` on real windows (the Arvane test driver): real
//! layout, shaping, rendering and accessibility, frames shown in real
//! windows (headless where none can open), a virtual clock, failure
//! artifacts, pooled windows and tests on parallel threads.
//!
//! Set `KIVACT_TEST_HOLD_MS` to keep each window open that long at the end
//! of a test (to look at it, or to read it with UI Automation).
//! Every settle draws requested frames.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use kivact_core::motion::{FrameInfo, FrameLoop, frame_loop};
use kivact_engine::testing::display_stats;
use kivact_runtime::signal;

use super::artifacts::failure_dir;
use super::support::button_box;
use crate::{MirrorMode, RenderOptions, render, render_with};
use kivact_core::motion::FrameRate;
use kivact_core::style::style;
use kivact_core::view::{IntoView, Role, column, row, show, text};

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
    RenderOptions::default()
        .window_size(400.0, 300.0)
        .mirror(mirror)
}

#[test]
fn virtual_time_drives_sleep_and_frame_loops_exactly() {
    let screen = render(|| {
        let msg = signal("waiting".to_string());
        column((
            text(move || msg.get()),
            button_box("go").on_press(move || {
                kivact_core::task::spawn(async move {
                    kivact_core::task::sleep(Duration::from_millis(500)).await;
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
        kivact_core::task::spawn(async move {
            kivact_core::task::sleep(Duration::from_millis(300)).await;
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
    screen.with_host(|host| host.set_surface(true));
    assert!(screen.with_host(|host| host.redraw_requested()));
    screen.run_until_idle();
    screen.get_by_text("count: 2");
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

fn ticks(hz: f64, period: u64) -> u64 {
    let count = Rc::new(Cell::new(0u64));
    let slot: Rc<Cell<Option<FrameLoop>>> = Rc::new(Cell::new(None));
    let (c, s) = (count.clone(), slot.clone());
    let screen = render_with(small(MirrorMode::Off).frame_period_ns(period), move || {
        s.set(Some(frame_loop(move |_: FrameInfo| c.set(c.get() + 1))));
        text("frames")
    });
    slot.take().expect("the loop").start(FrameRate::Hz(hz));
    screen.advance(Duration::from_secs(1));
    assert_eq!(screen.frame_ticks(), count.get());
    count.get()
}
