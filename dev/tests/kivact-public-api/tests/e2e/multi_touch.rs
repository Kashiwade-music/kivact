//! Role: Tests for several pointers at once: fingers, a pen and the mouse.
//!
//! Every touch or pen contact is a pointer of its own: its events go to the
//! listener its `Down` captured, a drag follows the pointer that began it,
//! and one finger's cancel ends only that finger's drag. Only the primary
//! pointers (the mouse, the first finger while no other is down) hover,
//! focus and click; a later finger only reaches `on_pointer`. Two fingers
//! on a `zoom_pan` surface pinch, and a mouse hovering and clicking while a
//! finger drags keeps both working.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount;
use kivact::draw::Point;
use kivact::interaction::{
    DragCoordinates, DragPhase, DragUpdate, ViewTransform, use_drag, use_splitter, use_zoom_pan,
};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::Screen;
use kivact::ui::{EditPhase, ScalarRange};
use kivact::view::{Handled, MouseButton, PointerEvent, PointerKind, PointerPhase};

type Phases = Rc<RefCell<Vec<DragPhase>>>;

fn center(screen: &Screen, id: &str) -> Point {
    screen.get_by_test_id(id).rect().expect("laid out").center()
}

/// A box a drag moves sideways: `value` is its offset, `phases` the drag's
/// phases, `downs` whether each `Down` it got was a primary pointer's.
fn draggable(id: &str, value: Signal<f32>, phases: Phases, downs: Rc<RefCell<Vec<bool>>>) -> Node {
    let started = signal(0.0_f32);
    let (start, end) = (phases.clone(), phases.clone());
    let session = use_drag::<()>(DragCoordinates::Window)
        .on_start(move |_| {
            start.borrow_mut().push(DragPhase::Start);
            started.set(value.get_untracked());
        })
        .on_update(move |update: DragUpdate<()>| {
            phases.borrow_mut().push(DragPhase::Move);
            value.set(started.get_untracked() + update.delta.x);
        })
        .on_end(move |update: DragUpdate<()>| end.borrow_mut().push(update.phase));
    node()
        .test_id(id)
        .style(style().size(200, 40))
        .on_pointer(move |event: PointerEvent| {
            if event.phase == PointerPhase::Down {
                downs.borrow_mut().push(event.is_primary);
            }
            Handled::No
        })
        .part(session.source(()))
}

#[test]
fn two_fingers_drag_two_boxes_and_a_splitter_each_moving_only_its_own_value() {
    let (a_phases, b_phases) = (Phases::default(), Phases::default());
    let downs: Rc<RefCell<Vec<bool>>> = Rc::default();
    let (pa, pb, d) = (a_phases.clone(), b_phases.clone(), downs.clone());
    let (screen, (a, b, split)) = mount(move || {
        let (a, b, split) = (signal(0.0_f32), signal(0.0_f32), signal(40.0));
        let range = ScalarRange::new(0.0, 100.0).expect("invariant: finite range");
        (
            (a, b, split),
            column((
                draggable("a", a, pa, d.clone()),
                draggable("b", b, pb, d),
                node()
                    .name("Split")
                    .test_id("split")
                    .style(style().size(8, 100))
                    .part(use_splitter(split, range, Orientation::Horizontal).handle()),
            ))
            .style(style().align_items(Align::Start).gap(10)),
        )
    });
    let (at_a, at_b, at_split) = (
        center(&screen, "a"),
        center(&screen, "b"),
        center(&screen, "split"),
    );
    let one = screen.contact_down(PointerKind::Touch, at_a.x, at_a.y);
    let two = screen.contact_down(PointerKind::Touch, at_b.x, at_b.y);
    let three = screen.contact_down(PointerKind::Touch, at_split.x, at_split.y);
    assert_eq!(
        *downs.borrow(),
        [true, false],
        "only the first finger is primary"
    );
    screen.contact_move(one, at_a.x + 30.0, at_a.y);
    screen.contact_move(two, at_b.x - 20.0, at_b.y);
    screen.contact_move(three, at_split.x + 15.0, at_split.y);
    assert_eq!(
        (a.get_untracked(), b.get_untracked(), split.get_untracked()),
        (30.0, -20.0, 55.0)
    );
    // Moving one finger leaves the others' values where they are, also
    // when the fingers cross over the other boxes.
    screen.contact_move(two, at_b.x + 5.0, at_a.y);
    screen.contact_move(one, at_a.x + 40.0, at_b.y);
    assert_eq!(
        (a.get_untracked(), b.get_untracked(), split.get_untracked()),
        (40.0, 5.0, 55.0)
    );
    screen.contact_up(two);
    screen.contact_up(one);
    screen.contact_up(three);
    assert_eq!(
        *a_phases.borrow(),
        [
            DragPhase::Start,
            DragPhase::Move,
            DragPhase::Move,
            DragPhase::End
        ]
    );
    assert_eq!(*b_phases.borrow(), *a_phases.borrow());
    assert_eq!(split.get_untracked(), 55.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn one_fingers_cancel_does_not_stop_the_other_fingers_drag() {
    let (a_phases, b_phases) = (Phases::default(), Phases::default());
    let downs: Rc<RefCell<Vec<bool>>> = Rc::default();
    let (pa, pb, d) = (a_phases.clone(), b_phases.clone(), downs.clone());
    let (screen, (a, b)) = mount(move || {
        let (a, b) = (signal(0.0_f32), signal(0.0_f32));
        (
            (a, b),
            column((draggable("a", a, pa, d.clone()), draggable("b", b, pb, d)))
                .style(style().align_items(Align::Start).gap(10)),
        )
    });
    let (at_a, at_b) = (center(&screen, "a"), center(&screen, "b"));
    let one = screen.contact_down(PointerKind::Touch, at_a.x, at_a.y);
    let two = screen.contact_down(PointerKind::Touch, at_b.x, at_b.y);
    screen.contact_move(one, at_a.x + 10.0, at_a.y);
    screen.contact_move(two, at_b.x + 10.0, at_b.y);
    screen.contact_cancel(one);
    assert_eq!(
        *a_phases.borrow(),
        [DragPhase::Start, DragPhase::Move, DragPhase::Cancel]
    );
    screen.contact_move(two, at_b.x + 25.0, at_b.y);
    assert_eq!(b.get_untracked(), 25.0, "the other finger still drags");
    assert_eq!(a.get_untracked(), 10.0);
    screen.contact_up(two);
    assert_eq!(
        *b_phases.borrow(),
        [
            DragPhase::Start,
            DragPhase::Move,
            DragPhase::Move,
            DragPhase::End
        ]
    );
    // The canceled finger's box takes a new drag at once.
    let again = screen.contact_down(PointerKind::Touch, at_a.x, at_a.y);
    screen.contact_move(again, at_a.x + 5.0, at_a.y);
    screen.contact_up(again);
    assert_eq!(a.get_untracked(), 15.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn two_fingers_pinch_a_zoom_pan_surface_about_their_midpoint() {
    let log: Rc<RefCell<Vec<EditPhase>>> = Rc::default();
    let out = log.clone();
    let (screen, transform) = mount(move || {
        let transform = signal(ViewTransform::default());
        (
            transform,
            column((node()
                .role(Role::Group)
                .name("Surface")
                .test_id("surface")
                .style(style().size(300, 200))
                .part(
                    use_zoom_pan(transform)
                        .on_edit(move |phase| out.borrow_mut().push(phase))
                        .viewport(),
                ),))
            .style(style().align_items(Align::Start)),
        )
    });
    let r = screen.get_by_test_id("surface").rect().expect("laid out");
    let at = |x: f32, y: f32| Point::new(r.x + x, r.y + y);
    // Fingers 50 px apart around (75, 50), spread to 100 px around (100, 50):
    // twice the scale, the content point (75, 50) under the new midpoint.
    let (p, q) = (at(50.0, 50.0), at(100.0, 50.0));
    let one = screen.contact_down(PointerKind::Touch, p.x, p.y);
    assert_eq!(
        transform.get_untracked(),
        ViewTransform::default(),
        "one finger alone does nothing"
    );
    let two = screen.contact_down(PointerKind::Touch, q.x, q.y);
    let q2 = at(150.0, 50.0);
    screen.contact_move(two, q2.x, q2.y);
    let pinched = transform.get_untracked();
    assert!((pinched.scale - 2.0).abs() < 1e-6, "{pinched:?}");
    assert!(
        (pinched.offset.x + 50.0).abs() < 0.01 && (pinched.offset.y + 50.0).abs() < 0.01,
        "{pinched:?}"
    );
    let content = pinched.to_content(Point::new(100.0, 50.0));
    assert!((content.x - 75.0).abs() < 0.01 && (content.y - 50.0).abs() < 0.01);
    // Both fingers moving together pan at that scale.
    screen.contact_move(one, p.x + 10.0, p.y + 20.0);
    screen.contact_move(two, q2.x + 10.0, q2.y + 20.0);
    let panned = transform.get_untracked();
    assert!((panned.scale - 2.0).abs() < 1e-6, "{panned:?}");
    assert!(
        (panned.offset.x + 40.0).abs() < 0.01 && (panned.offset.y + 30.0).abs() < 0.01,
        "{panned:?}"
    );
    screen.contact_up(two);
    assert_eq!(*log.borrow(), [EditPhase::Begin, EditPhase::Commit]);
    let committed = transform.get_untracked();
    // The finger left down does not pan alone, and lifting it changes
    // nothing.
    screen.contact_move(one, p.x + 60.0, p.y);
    screen.contact_up(one);
    assert_eq!(transform.get_untracked(), committed);

    // A canceled finger restores the transform from the pinch's start.
    log.borrow_mut().clear();
    let one = screen.contact_down(PointerKind::Touch, p.x, p.y);
    let two = screen.contact_down(PointerKind::Touch, q.x, q.y);
    screen.contact_move(two, q2.x, q2.y);
    assert_ne!(transform.get_untracked(), committed);
    screen.contact_cancel(one);
    assert_eq!(transform.get_untracked(), committed);
    assert_eq!(*log.borrow(), [EditPhase::Begin, EditPhase::Cancel]);
    screen.contact_up(two);
    assert_eq!(transform.get_untracked(), committed);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_mouse_hovers_and_clicks_while_a_finger_drags() {
    let presses = Rc::new(RefCell::new(0));
    let count = presses.clone();
    let phases = Phases::default();
    let (screen, dragged) = mount(move || {
        let dragged = signal(0.0_f32);
        (
            dragged,
            column((
                node()
                    .role(Role::Button)
                    .name("A")
                    .test_id("button")
                    .focusable(true)
                    .on_press(move || *count.borrow_mut() += 1)
                    .style(
                        style()
                            .size(60, 40)
                            .background(rgb(0x101010))
                            .hover(style().background(rgb(0x0000c0))),
                    ),
                draggable("drag", dragged, phases, Rc::default()),
            ))
            .style(style().align_items(Align::Start).gap(10)),
        )
    });
    let button = center(&screen, "button");
    let at = center(&screen, "drag");
    let color = || screen.screenshot().pixel(button.x as u32, button.y as u32);
    screen.mouse_move(button.x, button.y);
    assert_eq!(color(), Some(0x0000c0), "the mouse hovers the button");

    let finger = screen.contact_down(PointerKind::Touch, at.x, at.y);
    screen.contact_move(finger, at.x + 30.0, at.y);
    assert_eq!(dragged.get_untracked(), 30.0);
    assert_eq!(
        screen.hovered().map(|e| e.name()).as_deref(),
        Some("A"),
        "the finger's drag leaves the mouse's hover alone"
    );
    assert_eq!(color(), Some(0x0000c0));
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*presses.borrow(), 1, "the mouse clicks during the drag");
    screen.contact_move(finger, at.x + 40.0, at.y);
    assert_eq!(
        dragged.get_untracked(),
        40.0,
        "the click left the drag alone"
    );
    screen.contact_up(finger);
    assert_eq!(screen.hovered().map(|e| e.name()).as_deref(), Some("A"));
    assert_eq!(color(), Some(0x0000c0), "still hovered after the finger");
    // A finger's tap clicks too, the mouse still resting there.
    screen.get_by_test_id("button").tap();
    assert_eq!(*presses.borrow(), 2);
    assert_eq!(color(), Some(0x0000c0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_later_finger_reaches_on_pointer_but_does_not_click_or_focus() {
    let presses = Rc::new(RefCell::new(0));
    let count = presses.clone();
    let downs: Rc<RefCell<Vec<bool>>> = Rc::default();
    let seen = downs.clone();
    let (screen, ()) = mount(move || {
        (
            (),
            column((
                node()
                    .role(Role::Button)
                    .name("B")
                    .test_id("button")
                    .focusable(true)
                    .on_press(move || *count.borrow_mut() += 1)
                    .on_pointer(move |event: PointerEvent| {
                        if event.phase == PointerPhase::Down {
                            seen.borrow_mut().push(event.is_primary);
                        }
                        Handled::No
                    })
                    .style(style().size(60, 40)),
                node().test_id("rest").style(style().size(60, 40)),
            ))
            .style(style().align_items(Align::Start).gap(10)),
        )
    });
    let rest = center(&screen, "rest");
    let target = screen.get_by_test_id("button");
    let first = screen.contact_down(PointerKind::Touch, rest.x, rest.y);
    target.tap();
    assert_eq!(*downs.borrow(), [false], "the second finger is not primary");
    assert_eq!(*presses.borrow(), 0, "a later finger does not click");
    assert!(!target.is_focused(), "nor focus");
    // While a finger is down, a new finger is not primary, even after the
    // primary one lifted (Pointer Events' `isPrimary`).
    let second = screen.contact_down(PointerKind::Touch, rest.x, rest.y);
    screen.contact_up(first);
    target.tap();
    assert_eq!(*presses.borrow(), 0);
    screen.contact_up(second);
    // With no finger down, the next one is primary again; a pen is primary
    // of its own kind meanwhile.
    let finger = screen.contact_down(PointerKind::Touch, rest.x, rest.y);
    target.tap_with(PointerKind::Pen);
    assert_eq!(*presses.borrow(), 1, "the first pen is primary");
    screen.contact_up(finger);
    target.tap();
    assert_eq!(*presses.borrow(), 2);
    assert!(target.is_focused());
    assert_eq!(*downs.borrow(), [false, false, true, true]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
