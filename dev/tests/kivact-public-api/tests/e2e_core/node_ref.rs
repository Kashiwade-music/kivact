//! Role: Verify shared node observations and node-gated frame scheduling.

use kivact::motion::FrameRate;
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::view::{Asset, ImageStatus, ScrollBehavior, ScrollState};
use kivact_public_api_tests::support::mount;
use std::time::Duration;

#[test]
fn scroll_state_follows_the_scroll_position() {
    let (screen, (target, state, height, shown, axes)) = mount(|| {
        let target = node_ref();
        let state = target.scroll_state();
        let height = signal(500.0);
        let axes = signal(ScrollAxes::Vertical);
        let shown = signal(true);
        (
            (target, state, height, shown, axes),
            show(shown, move || {
                scroll(node().style(move || style().width(400).height(height.get()).shrink(0)))
                    .node_ref(target)
                    .axes(axes)
                    .style(style().size(200, 100))
            }),
        )
    });
    assert_eq!(state.get(), ScrollState::AtStart);
    height.set(50.0);
    screen.run_until_idle();
    assert_eq!(
        state.get(),
        ScrollState::NotScrollable,
        "extent changed at offset zero"
    );
    axes.set(ScrollAxes::Horizontal);
    screen.run_until_idle();
    assert_eq!(
        state.get(),
        ScrollState::AtStart,
        "axes changed without changing geometry"
    );
    axes.set(ScrollAxes::Vertical);
    height.set(500.0);
    screen.run_until_idle();
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
        let loop_handle = {
            let presence = target.presence();
            frame_loop(move |_| ticks.update(|count| *count += 1))
                .while_present(move || presence.get() == kivact::view::Presence::Present)
        };
        let rate = loop_handle.rate();
        (
            (shown, ticks, loop_handle, rate),
            show(shown, move || {
                node().node_ref(target).style(
                    style()
                        .size(40, 30)
                        .enter(style().opacity(0.0))
                        .exit(style().opacity(0.0))
                        .transition(TransitionProp::All, Transition::tween(120, Easing::Linear)),
                )
            }),
        )
    });
    loop_handle.start(FrameRate::Hz(30.0));
    screen.advance(Duration::from_millis(100));
    assert_eq!(ticks.get(), 0);
    assert_eq!(rate.get(), Some(FrameRate::Hz(30.0)));
    shown.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(50));
    assert_eq!(ticks.get(), 0, "entering node keeps the callback paused");
    screen.advance(Duration::from_millis(200));
    assert!(ticks.get() > 0);
    shown.set(false);
    screen.run_until_idle();
    let paused = ticks.get();
    screen.advance(Duration::from_millis(100));
    assert_eq!(ticks.get(), paused);
    shown.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(250));
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

#[test]
fn rect_called_after_binding_follows_the_node() {
    let (screen, (target, width)) = mount(|| {
        let target = node_ref();
        let width = signal(120.0);
        (
            (target, width),
            node()
                .node_ref(target)
                .style(move || style().width(width.get()).height(40).shrink(0)),
        )
    });
    // Bound at mount, before anything read its geometry.
    let rect = target.rect();
    screen.run_until_idle();
    assert_eq!(rect.get().map(|r| r.width), Some(120.0));
    width.set(160.0);
    screen.run_until_idle();
    assert_eq!(rect.get().map(|r| r.width), Some(160.0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Handlers that activate themselves or each other run in budgeted turns:
/// a turn over its budget reports a cycle and the rest runs in the next one.
#[test]
fn activating_itself_or_each_other_does_not_hang() {
    const TIMES: u32 = 2500;
    let (screen, (own, mutual)) = mount(|| {
        let own = signal(0u32);
        let mutual = signal(0u32);
        let me = node_ref();
        let (x, y) = (node_ref(), node_ref());
        (
            (own, mutual),
            column((
                button("self").node_ref(me).on_press(move || {
                    own.update(|n| *n += 1);
                    if own.get_untracked() < TIMES {
                        me.activate();
                    }
                }),
                button("x").node_ref(x).on_press(move || {
                    mutual.update(|n| *n += 1);
                    if mutual.get_untracked() < TIMES {
                        y.activate();
                    }
                }),
                button("y").node_ref(y).on_press(move || {
                    mutual.update(|n| *n += 1);
                    if mutual.get_untracked() < TIMES {
                        x.activate();
                    }
                }),
            )),
        )
    });
    screen.get_by_role(Role::Button, "self").click();
    screen.run_until_idle();
    assert_eq!(own.get_untracked(), TIMES);
    screen.get_by_role(Role::Button, "x").click();
    screen.run_until_idle();
    assert_eq!(mutual.get_untracked(), TIMES);
    let errors = screen.errors();
    assert!(!errors.is_empty(), "a turn over its budget reports a cycle");
    assert!(
        errors
            .iter()
            .all(|e| matches!(e.kind(), kivact::ErrorKind::Cycle { .. })),
        "{errors:?}"
    );
}
