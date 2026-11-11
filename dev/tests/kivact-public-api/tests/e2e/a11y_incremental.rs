//! Role: Tests that accessibility publishes in proportion to what changed
//! and that the pruned hit test finds what the complete walk finds.
//!
//! Hover, press, focus, a text update and a scroll publish only the
//! payloads they change: the identity enumeration (`Stats::a11y_enumerated`)
//! does not run for them. What is published is checked against a complete
//! build at every settle by the debug self-check, which also compares the
//! hit under the mouse (where the last click was) and at the window's center
//! with the complete walk, drawables included.

use std::cell::RefCell;
use std::rc::Rc;

use crate::paint::{RED, at, canvas_of, expect};
use crate::support::mount;
use kivact::draw::Transform;
use kivact::geometry::Point;
use kivact::prelude::*;
use kivact::style::{Display, Position};
use kivact::test::Screen;
use kivact::view::View;

type Log = Rc<RefCell<Vec<String>>>;

/// The `kv_latency` screen at 200 rows: a text `Other <n>`, the buttons
/// `Other` and `Bump`, and a 300 px scroll box `list` of rows (a label and a
/// button with the test id `x<i>`); with `pinned`, the list's content also
/// holds a 20 px fixed box `pin`.
fn latency_screen(pinned: bool) -> Screen {
    let (screen, ()) = mount(move || {
        let other = signal(0u32);
        let first = signal(String::from("Row 0"));
        let mut rows: Vec<View> = (0..200u32)
            .map(|i| {
                let label = if i == 0 {
                    text(move || first.get()).into_view()
                } else {
                    text(format!("Row {i}")).into_view()
                };
                row((label, button("x").test_id(format!("x{i}"))))
                    .style(style().gap(4))
                    .into_view()
            })
            .collect();
        if pinned {
            rows.push(
                node()
                    .test_id("pin")
                    .style(
                        style()
                            .position(Position::Fixed)
                            .left(10)
                            .top(10)
                            .size(20, 20),
                    )
                    .into_view(),
            );
        }
        let view = column((
            text(move || format!("Other {}", other.get())),
            button("Other").on_press(move || other.update(|n| *n += 1)),
            button("Bump").on_press(move || first.update(|l| l.push('!'))),
            scroll(column(rows))
                .test_id("list")
                .style(style().height(300).shrink(0.0)),
        ));
        ((), view)
    });
    screen
}

#[test]
fn hover_and_press_publish_without_enumerating() {
    let screen = latency_screen(false);
    screen.get_by_test_id("x3").hover();
    let s0 = screen.stats();
    screen.get_by_test_id("x4").hover();
    let s1 = screen.stats();
    assert_eq!(
        s1.a11y_enumerated, s0.a11y_enumerated,
        "a hover enumerates nothing"
    );
    assert!(
        s1.a11y_constructed - s0.a11y_constructed <= 4,
        "a hover builds the two buttons: {}",
        s1.a11y_constructed - s0.a11y_constructed
    );
    screen.get_by_test_id("x4").click();
    let s2 = screen.stats();
    assert_eq!(
        s2.a11y_enumerated, s1.a11y_enumerated,
        "a press and the first focus enumerate nothing"
    );
}

#[test]
fn a_text_update_publishes_only_the_text() {
    let screen = latency_screen(false);
    for name in ["Other", "Bump"] {
        let button = screen.get_by_role(Role::Button, name);
        button.click();
        let s0 = screen.stats();
        button.click();
        let s1 = screen.stats();
        assert_eq!(
            s1.a11y_enumerated, s0.a11y_enumerated,
            "{name}: a text update enumerates nothing"
        );
        assert!(
            s1.a11y_constructed - s0.a11y_constructed <= 6,
            "{name}: the button and the text: {}",
            s1.a11y_constructed - s0.a11y_constructed
        );
    }
}

#[test]
fn scrolling_publishes_the_container_and_its_layers() {
    let screen = latency_screen(true);
    let list = screen.get_by_test_id("list");
    list.hover();
    let s0 = screen.stats();
    list.scroll(0.0, 120.0);
    let s1 = screen.stats();
    assert_eq!(
        s1.a11y_enumerated, s0.a11y_enumerated,
        "a scroll enumerates nothing"
    );
    assert!(
        s1.a11y_constructed - s0.a11y_constructed <= 6,
        "the container and the fixed box: {}",
        s1.a11y_constructed - s0.a11y_constructed
    );
}

#[test]
fn a_list_clips_rows_by_scroll_and_focus() {
    let (screen, ()) = mount(|| {
        let options: Vec<Node> = (0..100)
            .map(|i| {
                node()
                    .role(Role::Option)
                    .name(format!("Option {i}"))
                    .focusable(true)
                    .style(style().height(20).shrink(0.0))
            })
            .collect();
        let view = scroll(column(options))
            .with_node(|n| n.role(Role::ListBox))
            .test_id("options")
            .style(style().height(200).shrink(0.0));
        ((), view)
    });
    let list = screen.get_by_test_id("options");
    list.hover();
    let enumerated = || screen.stats().a11y_enumerated;
    let start = enumerated();
    list.scroll(0.0, 400.0);
    assert_eq!(enumerated(), start, "scrolling a list enumerates nothing");
    screen.get_by_role(Role::Option, "Option 25").focus();
    assert_eq!(enumerated(), start, "focusing a row enumerates nothing");
    list.scroll(0.0, 1200.0);
    assert_eq!(
        enumerated(),
        start,
        "the focused row stays exposed out of view"
    );
    list.scroll(0.0, -1600.0);
    assert_eq!(enumerated(), start, "scrolling back enumerates nothing");
}

/// A button-like box named `name` that logs its activations.
fn pressable(name: impl Into<String>, log: &Log) -> Node {
    let name = name.into();
    let (log, said) = (log.clone(), name.clone());
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .on_press(move || log.borrow_mut().push(said.clone()))
}

/// What a click at the window point `(x, y)` activated.
fn clicked(screen: &Screen, log: &Log, x: f64, y: f64) -> Vec<String> {
    screen.click_at(x, y);
    std::mem::take(&mut *log.borrow_mut())
}

/// 200 pressable rows of 20 px in a 200 px scroll box at (450, 40) whose
/// first child is a sticky 20 px heading.
fn rows_with_heading(log: &Log) -> impl IntoView {
    let mut children: Vec<Node> = vec![
        pressable("heading", log).style(
            style()
                .position(Position::Sticky)
                .top(0)
                .height(20)
                .shrink(0.0)
                .background(rgb(0xdddddd)),
        ),
    ];
    children.extend(
        (0..200).map(|i| pressable(format!("row {i}"), log).style(style().height(20).shrink(0.0))),
    );
    scroll(column(children))
        .test_id("rows")
        .style(at(450, 40, 200, 200))
}

#[test]
fn hit_reaches_boxes_outside_their_parents() {
    let log: Log = Rc::default();
    let (screen, ()) = mount({
        let log = log.clone();
        move || {
            let view = canvas_of((
                node().style(at(20, 40, 100, 100)).child(
                    pressable("badge", &log).style(
                        style()
                            .position(Position::Absolute)
                            .left(90)
                            .top(-20)
                            .size(30, 30),
                    ),
                ),
                node()
                    .style(at(200, 40, 100, 100))
                    .child(pressable("moved", &log).style(style().size(40, 40).translate(150, 0))),
                pressable("under", &log).style(at(20, 200, 100, 100)),
                node()
                    .hit_testable(false)
                    .style(at(20, 200, 100, 100))
                    .child(pressable("ignored", &log).style(style().size(100, 100))),
                node()
                    .style(style().display(Display::Contents))
                    .child(pressable("contents", &log).style(at(200, 200, 60, 60))),
                rows_with_heading(&log),
            ));
            ((), view)
        }
    });
    assert_eq!(
        clicked(&screen, &log, 125.0, 30.0),
        ["badge"],
        "outside its parent, above it"
    );
    assert_eq!(
        clicked(&screen, &log, 370.0, 60.0),
        ["moved"],
        "translated out of its parent"
    );
    assert_eq!(
        clicked(&screen, &log, 70.0, 250.0),
        ["under"],
        "through a box that is not hit testable"
    );
    assert_eq!(
        clicked(&screen, &log, 230.0, 230.0),
        ["contents"],
        "in a display: contents wrapper"
    );
    screen.get_by_test_id("rows").scroll(0.0, 300.0);
    assert_eq!(
        clicked(&screen, &log, 550.0, 50.0),
        ["heading"],
        "the stuck heading over the rows"
    );
    assert_eq!(
        clicked(&screen, &log, 550.0, 130.0),
        ["row 18"],
        "a row scrolled into view"
    );
}

/// Where a 200 px card turned 45° about x, seen through a perspective of
/// 400 about its centre, is hit outside its untransformed box: its box is
/// at `(x, y)` of the window and the projection spans x − 21.5 to x + 221.5
/// and y + 39.9 to y + 185.9 (`transform_3d::rect_is_the_projected_bounds`),
/// so `(x + 210, y + 180)` lies in its near edge, 10 px right of the box.
fn beyond_the_box(x: f64, y: f64) -> (f64, f64) {
    (x + 210.0, y + 180.0)
}

/// A 200 px card named `name`, turned 45° about x.
fn card(name: &'static str, log: &Log) -> Node {
    pressable(name, log).style(style().size(200, 200).rotate_x(45))
}

#[test]
fn hit_reaches_projected_boxes_outside_their_parents() {
    let log: Log = Rc::default();
    let (screen, ()) = mount({
        let log = log.clone();
        move || {
            let view = canvas_of((
                // The parent's perspective projects the card.
                node()
                    .style(at(20, 20, 200, 200).perspective(400))
                    .child(card("child", &log)),
                // The same, one plain box further down.
                node().style(at(280, 20, 200, 200)).child(
                    node()
                        .style(style().size(200, 200).perspective(400))
                        .child(card("grandchild", &log)),
                ),
                // The card's own transform holds the perspective.
                node().style(at(540, 20, 200, 200)).child(
                    pressable("own", &log).style(
                        style()
                            .size(200, 200)
                            .transform(Transform::new().perspective(400.0).rotate_x(45.0)),
                    ),
                ),
            ));
            ((), view)
        }
    });
    for (name, x) in [("child", 20.0), ("grandchild", 280.0), ("own", 540.0)] {
        let (cx, cy) = beyond_the_box(x, 20.0);
        assert_eq!(
            clicked(&screen, &log, cx, cy),
            [name],
            "{name}: projected out of its parent's box"
        );
    }
}

#[test]
fn hit_reaches_a_projected_box_inside_a_drawable() {
    let log: Log = Rc::default();
    let (screen, ()) = mount({
        let log = log.clone();
        move || {
            let deep = drawable();
            let view = canvas(move |d, _| d.drawable(deep, Point::new(100.0, 100.0)))
                .test_id("stage")
                .style(style().size(800, 600))
                .drawable(deep, move || {
                    node().style(style().size(200, 200)).child(
                        node()
                            .style(style().size(200, 200).perspective(400))
                            .child(card("deep", &log)),
                    )
                });
            ((), view)
        }
    });
    let stage = screen.get_by_test_id("stage").rect().expect("laid out");
    let (x, y) = beyond_the_box(stage.x + 100.0, stage.y + 100.0);
    assert_eq!(
        clicked(&screen, &log, x, y),
        ["deep"],
        "projected out of its parent's box inside a drawable"
    );
}

/// A red 200 px card turned 45° about x under its parent's perspective of
/// 400, the parent's box at `(x, y)`.
fn projected_card(x: i32, y: i32) -> Node {
    node()
        .style(at(x, y, 200, 200).perspective(400))
        .child(node().style(style().size(200, 200).background(rgb(RED)).rotate_x(45)))
}

#[test]
fn a_projection_into_the_window_is_drawn_from_a_box_outside_it() {
    // Geometry tier (±1 px): the parents' boxes end at x = -10, left of the
    // window; the cards' near edges reach x = 11.5 (`beyond_the_box`), so
    // the pixel (4, y + 182) is in each projection.
    let (screen, ()) = mount(|| {
        let mut many: Vec<Node> = (0..39)
            .map(|i| node().style(at(300 + 10 * i, 0, 8, 8)))
            .collect();
        many.push(projected_card(-210, 0));
        let view = canvas_of((
            projected_card(-210, 20),
            // An indexed container: its children are found by their reach.
            node().style(at(0, 300, 800, 300)).child(many),
        ));
        ((), view)
    });
    let shot = screen.screenshot();
    expect(&shot, 4, 202, RED, 2, "a few children");
    expect(&shot, 4, 482, RED, 2, "an indexed container");
}
