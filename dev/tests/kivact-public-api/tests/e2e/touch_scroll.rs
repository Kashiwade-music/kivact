//! Role: Touch press/scroll threshold, chaining, cancellation and multi-touch.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::{Handled, PointerEvent, PointerKind, PointerPhase};

fn y(screen: &Screen, id: &str) -> f32 {
    screen.get_by_test_id(id).rect().expect("laid out").y
}

#[test]
fn taps_and_edges_keep_presses_while_scrolling_cancels_capture_once() {
    let clicks = Rc::new(Cell::new(0));
    let phases = Rc::new(RefCell::new(Vec::new()));
    let (count, events) = (clicks.clone(), phases.clone());
    let (screen, ()) = mount(move || {
        (
            (),
            scroll(column((
                button("Target")
                    .test_id("target")
                    .on_press(move || count.set(count.get() + 1))
                    .on_pointer(move |event: PointerEvent| {
                        events.borrow_mut().push(event.phase);
                        Handled::No
                    })
                    .style(style().size(160, 60).shrink(0.0)),
                node().style(style().height(400).shrink(0.0)),
            )))
            .test_id("scroll")
            .style(style().size(200, 100)),
        )
    });
    let bounds = screen.get_by_test_id("target").rect().unwrap();
    let (x, at) = (bounds.x + 50.0, bounds.y + 30.0);
    let finger = screen.contact_down(PointerKind::Touch, x, at);
    screen.contact_move(finger, x, at - 5.0);
    screen.contact_up(finger);
    assert_eq!(clicks.get(), 1, "movement below threshold is a tap");
    assert_eq!(y(&screen, "target"), bounds.y);
    // At the upper boundary no ancestor can move upward, so retain the press.
    let finger = screen.contact_down(PointerKind::Touch, x, at);
    screen.contact_move(finger, x, at + 20.0);
    screen.contact_up(finger);
    assert_eq!(clicks.get(), 2);
    assert_eq!(y(&screen, "target"), bounds.y);
    phases.borrow_mut().clear();
    let finger = screen.contact_down(PointerKind::Touch, x, at);
    screen.contact_move(finger, x, at - 20.0);
    assert!((y(&screen, "target") - (bounds.y - 20.0)).abs() < 0.1);
    screen.contact_move(finger, x, at - 30.0);
    screen.contact_up(finger);
    assert_eq!(clicks.get(), 2, "a scroll never activates on release");
    assert_eq!(
        phases
            .borrow()
            .iter()
            .filter(|&&phase| phase == PointerPhase::Cancel)
            .count(),
        1
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn touch_scroll_chains_to_the_outer_container_at_the_inner_boundary() {
    let (screen, ()) = mount(|| {
        (
            (),
            scroll(column((
                node()
                    .test_id("outer-top")
                    .style(style().height(20).shrink(0.0)),
                scroll(column((
                    button("Inner")
                        .test_id("inner-target")
                        .style(style().height(60).shrink(0.0)),
                    node().style(style().height(200).shrink(0.0)),
                )))
                .test_id("inner")
                .style(style().height(80).shrink(0.0)),
                node().style(style().height(400).shrink(0.0)),
            )))
            .style(style().size(200, 120)),
        )
    });
    let target = screen.get_by_test_id("inner-target").rect().unwrap();
    let top = y(&screen, "outer-top");
    let (x, at) = (target.x + 30.0, target.y + 30.0);
    let finger = screen.contact_down(PointerKind::Touch, x, at);
    screen.contact_move(finger, x, at - 300.0);
    assert_eq!(
        y(&screen, "outer-top"),
        top,
        "first movement goes to the inner scroller"
    );
    screen.contact_move(finger, x, at - 330.0);
    assert!(
        y(&screen, "outer-top") < top,
        "subsequent delta chains outward"
    );
    screen.contact_up(finger);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_second_contact_disables_pending_scroll_and_suspension_cancels_the_press() {
    let clicks = Rc::new(Cell::new(0));
    let count = clicks.clone();
    let (screen, ()) = mount(move || {
        (
            (),
            scroll(column((
                button("Target")
                    .test_id("target")
                    .on_press(move || count.set(count.get() + 1))
                    .style(style().height(60).shrink(0.0)),
                node().style(style().height(400).shrink(0.0)),
            )))
            .style(style().size(200, 100)),
        )
    });
    let target = screen.get_by_test_id("target").rect().unwrap();
    let (x, at) = (target.x + 30.0, target.y + 30.0);
    let one = screen.contact_down(PointerKind::Touch, x, at);
    let two = screen.contact_down(PointerKind::Touch, x + 30.0, at);
    screen.contact_move(one, x, at - 20.0);
    assert_eq!(
        y(&screen, "target"),
        target.y,
        "multi-touch remains available to custom handlers"
    );
    screen.contact_cancel(one);
    screen.contact_cancel(two);
    let one = screen.contact_down(PointerKind::Touch, x, at);
    screen.contact_move(one, x, at - 20.0);
    let scrolled = y(&screen, "target");
    assert!(scrolled < target.y);
    let two = screen.contact_down(PointerKind::Touch, x + 30.0, at - 20.0);
    screen.contact_move(one, x, at - 40.0);
    assert_eq!(
        y(&screen, "target"),
        scrolled,
        "established scrolling ends before multi-touch"
    );
    screen.contact_cancel(one);
    screen.contact_cancel(two);
    let finger = screen.contact_down(PointerKind::Touch, x, at);
    screen.suspend(false);
    screen.resume();
    screen.contact_up(finger);
    assert_eq!(clicks.get(), 0);
    let finger = screen.contact_down(PointerKind::Touch, x, at);
    screen.set_window_focused(false);
    screen.set_window_focused(true);
    screen.contact_up(finger);
    assert_eq!(
        clicks.get(),
        0,
        "focus loss never leaves an activatable press"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn removing_the_hit_child_cancels_its_surviving_ancestor_capture() {
    let phases = Rc::new(RefCell::new(Vec::new()));
    let events = phases.clone();
    let (screen, shown) = mount(move || {
        let shown = signal(true);
        (
            shown,
            scroll(
                column((
                    kivact::view::show(shown, || {
                        node()
                            .test_id("origin")
                            .style(style().height(60).shrink(0.0))
                    }),
                    node()
                        .test_id("marker")
                        .style(style().height(400).shrink(0.0)),
                ))
                .on_pointer(move |event: PointerEvent| {
                    events.borrow_mut().push(event.phase);
                    Handled::No
                }),
            )
            .style(style().size(200, 100)),
        )
    });
    let origin = screen.get_by_test_id("origin").rect().unwrap();
    let finger = screen.contact_down(PointerKind::Touch, origin.x + 30.0, origin.y + 30.0);
    shown.set(false);
    screen.run_until_idle();
    let before = y(&screen, "marker");
    screen.contact_move(finger, origin.x + 30.0, origin.y + 10.0);
    screen.contact_up(finger);
    assert_eq!(
        y(&screen, "marker"),
        before,
        "removed origin cannot begin scrolling"
    );
    let phases = phases.borrow().clone();
    assert_eq!(
        phases
            .iter()
            .filter(|&&phase| phase == PointerPhase::Cancel)
            .count(),
        1
    );
    assert!(
        !phases.contains(&PointerPhase::Up),
        "capture was canceled before release: {phases:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_modal_change_cancels_a_pending_touch_capture_before_it_can_commit() {
    use kivact::view::{Placement, overlay, show};
    let phases = Rc::new(RefCell::new(Vec::new()));
    let events = phases.clone();
    let (screen, open) = mount(move || {
        let open = signal(false);
        let anchor = node_ref();
        (
            open,
            column((
                scroll(
                    column((
                        node()
                            .test_id("origin")
                            .style(style().height(60).shrink(0.0)),
                        node().style(style().height(400).shrink(0.0)),
                    ))
                    .on_pointer(move |event: PointerEvent| {
                        events.borrow_mut().push(event.phase);
                        Handled::No
                    }),
                )
                .style(style().size(200, 100)),
                node().node_ref(anchor).style(style().size(20, 20)),
                show(open, move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        button("Modal").style(style().size(120, 40)),
                    )
                    .modal(true)
                }),
            )),
        )
    });
    let origin = screen.get_by_test_id("origin").rect().unwrap();
    let finger = screen.contact_down(PointerKind::Touch, origin.x + 30.0, origin.y + 30.0);
    open.set(true);
    screen.run_until_idle();
    open.set(false);
    screen.run_until_idle();
    screen.contact_up(finger);
    let phases = phases.borrow().clone();
    assert_eq!(
        phases
            .iter()
            .filter(|&&phase| phase == PointerPhase::Cancel)
            .count(),
        1
    );
    assert!(
        !phases.contains(&PointerPhase::Up),
        "old capture cannot commit after modal changes: {phases:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_second_contact_keeps_custom_pinch_available_inside_a_scroll_container() {
    use kivact::interaction::{ViewTransform, use_zoom_pan};
    let (screen, transform) = mount(|| {
        let transform = signal(ViewTransform::default());
        (
            transform,
            scroll(
                node()
                    .test_id("pinch-surface")
                    .part(use_zoom_pan(transform).viewport())
                    .style(style().size(200, 400).shrink(0.0)),
            )
            .style(style().size(200, 150)),
        )
    });
    let surface = screen.get_by_test_id("pinch-surface").rect().unwrap();
    let one = screen.contact_down(PointerKind::Touch, surface.x + 30.0, surface.y + 50.0);
    let two = screen.contact_down(PointerKind::Touch, surface.x + 80.0, surface.y + 50.0);
    screen.contact_move(two, surface.x + 80.0, surface.y + 100.0);
    screen.contact_move(one, surface.x + 30.0, surface.y + 25.0);
    assert!(
        transform.get_untracked().scale > 1.5,
        "both contacts reach the existing pinch controller"
    );
    assert_eq!(
        y(&screen, "pinch-surface"),
        surface.y,
        "pending native scroll was abandoned"
    );
    screen.contact_cancel(one);
    screen.contact_cancel(two);
    assert_eq!(transform.get_untracked(), ViewTransform::default());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
