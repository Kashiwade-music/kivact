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
