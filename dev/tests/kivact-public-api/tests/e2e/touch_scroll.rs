//! Role: Touch press/scroll threshold, chaining, cancellation and multi-touch.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::prelude::*;
use kivact::test::Screen;
use kivact::view::{Handled, PointerEvent, PointerKind, PointerPhase};

fn y(screen: &Screen, id: &str) -> f64 {
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
                        events.borrow_mut().push(event.phase());
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
fn fling_decelerates_and_stops() {
    use std::time::Duration;
    let (screen, offset) = mount(|| {
        let reference = node_ref();
        (
            reference.scroll_offset(),
            scroll(node().style(style().height(5000).shrink(0.0)))
                .node_ref(reference)
                .test_id("fling")
                .style(style().size(200, 160)),
        )
    });
    screen.set_pointer_settings(kivact::platform::PointerSettings {
        fling_deceleration: 0.98,
        ..Default::default()
    });
    let bounds = screen.get_by_test_id("fling").rect().unwrap();
    let finger = screen.contact_down(PointerKind::Touch, bounds.x + 40.0, bounds.y + 130.0);
    screen.contact_move(finger, bounds.x + 40.0, bounds.y + 110.0);
    screen.advance(Duration::from_millis(20));
    screen.contact_move(finger, bounds.x + 40.0, bounds.y + 70.0);
    screen.contact_up(finger);
    let released = offset.get().y;
    screen.advance(Duration::from_millis(80));
    let first = offset.get().y - released;
    screen.advance(Duration::from_millis(80));
    let second = offset.get().y - released - first;
    assert!(first > 0.0 && second > 0.0 && second < first);
    screen.advance(Duration::from_secs(3));
    let stopped = offset.get();
    screen.advance(Duration::from_secs(1));
    assert_eq!(offset.get(), stopped);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn scroll_snap_settles_on_a_stop() {
    use kivact::style::{Display, ScrollSnap};
    use std::time::Duration;
    let (screen, offset) = mount(|| {
        let reference = node_ref();
        (
            reference.scroll_offset(),
            scroll(
                column((
                    node().style(style().height(90).shrink(0.0)),
                    node()
                        .test_id("snap-child")
                        .style(style().height(150).shrink(0.0)),
                    node().style(style().height(300).shrink(0.0)),
                ))
                .style(style().display(Display::Contents)),
            )
            .node_ref(reference)
            .test_id("snap")
            .style(style().size(200, 100).scroll_snap(ScrollSnap::Start)),
        )
    });
    let bounds = screen.get_by_test_id("snap").rect().unwrap();
    let stop = screen.get_by_test_id("snap-child").rect().unwrap().y - bounds.y;
    let finger = screen.contact_down(PointerKind::Touch, bounds.x + 40.0, bounds.y + 90.0);
    screen.contact_move(finger, bounds.x + 40.0, bounds.y + 20.0);
    screen.contact_up(finger);
    screen.advance(Duration::from_secs(3));
    assert!((offset.get().y - stop).abs() < 0.5);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn smooth_scroll_uses_the_easing() {
    use kivact::geometry::Point;
    use kivact::motion::Easing;
    use kivact::view::ScrollBehavior;
    use std::time::Duration;
    let (screen, (slow, fast)) = mount(|| {
        let (slow, fast) = (node_ref(), node_ref());
        (
            (slow.scroll_offset(), fast.scroll_offset()),
            column((
                button("Animate").on_press(move || {
                    slow.scroll_to(
                        Point::new(0.0, 300.0),
                        ScrollBehavior::Smooth(Easing::EaseIn),
                    );
                    fast.scroll_to(
                        Point::new(0.0, 300.0),
                        ScrollBehavior::Smooth(Easing::EaseOut),
                    );
                }),
                scroll(node().style(style().height(1000).shrink(0.0)))
                    .node_ref(slow)
                    .style(style().size(200, 100)),
                scroll(node().style(style().height(1000).shrink(0.0)))
                    .node_ref(fast)
                    .style(style().size(200, 100)),
            )),
        )
    });
    screen.get_by_text("Animate").click();
    screen.advance(Duration::from_millis(120));
    assert!(slow.get().y > 0.0 && fast.get().y > slow.get().y);
    screen.advance(Duration::from_secs(1));
    assert_eq!(slow.get(), fast.get());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
