//! Role: Verify shared node observations and node-gated frame scheduling.

use crate::support::mount;
use kivact::geometry::Point;
use kivact::motion::{FrameRate, frame_loop};
use kivact::prelude::*;
use kivact::view::{Asset, ImageStatus, ScrollBehavior, ScrollState};
use std::time::Duration;

#[test]
fn scroll_state_follows_the_scroll_position() {
    let (screen, (target, state, height, shown)) = mount(|| {
        let target = node_ref();
        let state = target.scroll_state();
        let height = signal(500.0);
        let shown = signal(true);
        (
            (target, state, height, shown),
            show(shown, move || {
                scroll(node().style(move || style().height(height.get()).shrink(0)))
                    .node_ref(target)
                    .style(style().size(200, 100))
            }),
        )
    });
    assert_eq!(state.get(), ScrollState::AtStart);
    target.scroll_to(Point::new(0.0, 120.0), ScrollBehavior::Instant);
    screen.run_until_idle();
    assert_eq!(state.get(), ScrollState::Middle);
    target.scroll_to(Point::new(0.0, 10000.0), ScrollBehavior::Instant);
    screen.run_until_idle();
    assert_eq!(state.get(), ScrollState::AtEnd);
    height.set(50.0);
    screen.run_until_idle();
    assert_eq!(state.get(), ScrollState::NotScrollable);
    shown.set(false);
    screen.run_until_idle();
    assert_eq!(state.get(), ScrollState::NotScrollable);
    height.set(500.0);
    shown.set(true);
    screen.run_until_idle();
    assert_eq!(state.get(), ScrollState::AtStart);
}

#[test]
fn image_status_reports_a_failed_image() {
    let (screen, (source, status)) = mount(|| {
        let source = signal(Asset::from("missing-node-ref-status.png"));
        let target = node_ref();
        let status = target.image_status();
        (
            (source, status),
            image(source).node_ref(target).on_image_error(|_| ()),
        )
    });
    assert_eq!(status.get(), ImageStatus::Failed);
    source.set(Asset::from(include_bytes!("../fixtures/rgba.png")));
    screen.run_until_idle();
    assert_eq!(status.get(), ImageStatus::Ready);
    source.set(Asset::from("missing-node-ref-status-again.png"));
    screen.run_until_idle();
    assert_eq!(status.get(), ImageStatus::Failed);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn frame_loop_runs_only_while_present() {
    let (screen, (shown, ticks, loop_handle, rate)) = mount(|| {
        let shown = signal(false);
        let ticks = signal(0);
        let target = node_ref();
        let loop_handle =
            frame_loop(move |_| ticks.update(|count| *count += 1)).while_present(target);
        let rate = loop_handle.rate();
        (
            (shown, ticks, loop_handle, rate),
            show(shown, move || {
                node().node_ref(target).style(style().size(40, 30))
            }),
        )
    });
    loop_handle.start(FrameRate::Hz(30.0));
    screen.advance(Duration::from_millis(100));
    assert_eq!(ticks.get(), 0);
    assert_eq!(rate.get(), Some(FrameRate::Hz(30.0)));
    shown.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    assert!(ticks.get() > 0);
    shown.set(false);
    screen.run_until_idle();
    let paused = ticks.get();
    screen.advance(Duration::from_millis(100));
    assert_eq!(ticks.get(), paused);
    shown.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    assert!(ticks.get() > paused);
    loop_handle.stop();
    shown.set(false);
    screen.run_until_idle();
    shown.set(true);
    screen.run_until_idle();
    let stopped = ticks.get();
    screen.advance(Duration::from_millis(100));
    assert_eq!(ticks.get(), stopped);
    assert_eq!(rate.get(), None);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
