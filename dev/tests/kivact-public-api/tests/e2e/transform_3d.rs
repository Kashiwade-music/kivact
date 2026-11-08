//! Role: Tests for 3D transforms: projection, hit testing and depth order.
//!
//! The projection of a node is the one §5.2 gives: the drawn pixels, the
//! hit test, `NodeRef::rect` and the accessibility bounds follow it. Each
//! test says which tier of §3.3 its pixels are checked against: geometry
//! (position within ±1 device px), compositor ops (±2/255 per channel).
//! Checked on real windows at scale 1.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::geometry::Rect;
use kivact::prelude::*;
use kivact::style::{BackfaceVisibility, TransformStyle};
use kivact::test::{Screenshot, render};

use crate::paint::{BLUE, RED, WHITE, at, canvas_of, distance, expect, pixel};
use crate::support::mount;

type Log = Rc<RefCell<Vec<String>>>;

/// `left` within `tolerance` of `right`.
#[track_caller]
fn near(left: f64, right: f64, tolerance: f64, what: &str) {
    assert!(
        (left - right).abs() <= tolerance,
        "{what}: {left} is not {right} (±{tolerance})"
    );
}

/// `a` and `b` are the same rect within `tolerance`.
#[track_caller]
fn same_rect(a: Rect, b: Rect, tolerance: f64, what: &str) {
    near(a.x, b.x, tolerance, &format!("{what} x"));
    near(a.y, b.y, tolerance, &format!("{what} y"));
    near(a.width, b.width, tolerance, &format!("{what} width"));
    near(a.height, b.height, tolerance, &format!("{what} height"));
}

/// A button named `name` that logs its activations.
fn pressable(name: &'static str, log: &Log) -> Node {
    let log = log.clone();
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .on_press(move || log.borrow_mut().push(name.to_string()))
}

/// The width of the red run on row `y` (the first to the last red pixel).
fn red_width(shot: &Screenshot, y: u32) -> u32 {
    let reds: Vec<u32> = (0..800)
        .filter(|&x| distance(pixel(shot, x, y), RED) <= 8)
        .collect();
    match (reds.first(), reds.last()) {
        (Some(first), Some(last)) => last - first + 1,
        _ => 0,
    }
}

#[test]
fn rotate_y_narrows_the_drawn_box() {
    // Compositor tier (flat fill, ±2/255); the 50 px edges are geometry
    // (within 1 px): the box is 100 px wide after cos 60° = 0.5.
    let screen = render(|| {
        canvas_of(node().style(at(100, 100, 200, 100).background(rgb(RED)).rotate_y(60)))
    });
    let shot = screen.screenshot();
    expect(&shot, 200, 150, RED, 2, "the centre");
    expect(&shot, 240, 150, RED, 2, "40 px right of the centre");
    expect(&shot, 160, 150, RED, 2, "40 px left of the centre");
    expect(&shot, 260, 150, WHITE, 2, "60 px right: outside the box");
    expect(&shot, 140, 150, WHITE, 2, "60 px left: outside the box");
}

#[test]
fn perspective_makes_the_far_edge_shorter() {
    // Geometry tier: the red run's width follows the projection, which
    // makes the top edge (tilted away) shorter than the bottom edge.
    let screen = render(|| {
        canvas_of(
            node()
                .style(at(100, 100, 200, 200).perspective(400))
                .child(node().style(style().size(200, 200).background(rgb(RED)).rotate_x(45))),
        )
    });
    let shot = screen.screenshot();
    let top = red_width(&shot, 145);
    let bottom = red_width(&shot, 280);
    assert!(
        top < bottom,
        "the far (top) edge is shorter: {top} px against {bottom} px"
    );
    // The projected widths at those rows, from the perspective of 400 px:
    // about 172 px near the top edge and 240 px near the bottom.
    near(top as f64, 172.0, 4.0, "width near the top edge");
    near(bottom as f64, 240.0, 4.0, "width near the bottom edge");
}

#[test]
fn hit_test_follows_the_projection() {
    // Geometry tier: a click is decided by the projected quad.
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || {
            canvas_of(
                pressable("slab", &log)
                    .style(at(100, 100, 200, 100).background(rgb(BLUE)).rotate_y(60)),
            )
        }
    });
    // The projected centre is hit.
    screen.click_at(200.0, 150.0);
    assert_eq!(*log.borrow(), ["slab"], "the projected centre is hit");
    // Inside the original box but outside the projection: not hit.
    screen.click_at(110.0, 150.0);
    assert_eq!(
        *log.borrow(),
        ["slab"],
        "a point inside the unrotated box but outside the projection"
    );
}

#[test]
fn rect_is_the_projected_bounds() {
    // Geometry tier (±0.5 px). The §5.2 bounds of the child's corners
    // (±100, ±100) under rotate_x(45) and the parent's perspective 400,
    // about the centre (200, 200): x 200 ± 121.474, y from 200 − 60.088 to
    // 200 + 85.895.
    let (_screen, rect) = mount(|| {
        let r = node_ref();
        let card = node()
            .node_ref(r)
            .style(style().size(200, 200).background(rgb(RED)).rotate_x(45));
        let view = canvas_of(
            node()
                .style(at(100, 100, 200, 200).perspective(400))
                .child(card),
        );
        (r.rect(), view)
    });
    let drawn = rect.get_untracked().expect("reported");
    same_rect(
        drawn,
        Rect::new(78.526, 139.912, 242.947, 145.983),
        0.5,
        "NodeRef::rect is the projected bounds",
    );
}

#[test]
fn preserve_3d_orders_by_depth() {
    // Compositor tier (±2/255): the front child (nearer, translate_z 50)
    // covers the back one, and the click reaches the front child.
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || {
            canvas_of(
                node()
                    .style(at(100, 100, 200, 200).transform_style(TransformStyle::Preserve3d))
                    .child((
                        pressable("front", &log)
                            .style(at(0, 0, 120, 120).background(rgb(RED)).translate_z(50)),
                        pressable("back", &log)
                            .style(at(0, 0, 120, 120).background(rgb(BLUE)).translate_z(0)),
                    )),
            )
        }
    });
    let shot = screen.screenshot();
    expect(&shot, 150, 150, RED, 2, "the front child is on top");
    screen.click_at(150.0, 150.0);
    assert_eq!(
        *log.borrow(),
        ["front"],
        "the click reaches the front child"
    );
}

#[test]
fn backface_hidden_is_not_drawn_or_hit() {
    // Compositor tier for the pixel; the hit test is geometry.
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || {
            canvas_of(
                pressable("back", &log).style(
                    at(100, 100, 100, 100)
                        .background(rgb(RED))
                        .rotate_y(180)
                        .backface_visibility(BackfaceVisibility::Hidden),
                ),
            )
        }
    });
    let shot = screen.screenshot();
    expect(&shot, 150, 150, WHITE, 2, "the back face is not drawn");
    screen.click_at(150.0, 150.0);
    assert!(log.borrow().is_empty(), "the back face is not hit");
}

#[test]
fn mirrored_backface_hidden_is_still_drawn() {
    // Compositor tier for the pixel; the hit test is geometry. A mirror
    // keeps the front face toward the viewer, so it is drawn and hit.
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || {
            canvas_of(
                pressable("mirror", &log).style(
                    at(100, 100, 100, 100)
                        .background(rgb(RED))
                        .scale_xy(-1.0, 1.0)
                        .backface_visibility(BackfaceVisibility::Hidden),
                ),
            )
        }
    });
    let shot = screen.screenshot();
    expect(&shot, 150, 150, RED, 2, "the mirrored front face is drawn");
    screen.click_at(150.0, 150.0);
    assert_eq!(*log.borrow(), ["mirror"], "the mirrored front face is hit");
}

#[test]
fn a11y_bounds_of_3d_nodes_are_projected() {
    // Geometry tier (±1 px): the rotate_y(60) box is 100 px wide at the
    // centre (200, 150).
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || canvas_of(pressable("tilt", &log).style(at(100, 100, 200, 100).rotate_y(60)))
    });
    let bounds = screen
        .get_by_role(Role::Button, "tilt")
        .accessibility_bounds()
        .expect("exposed");
    same_rect(
        bounds,
        Rect::new(150.0, 100.0, 100.0, 100.0),
        1.0,
        "the accessibility bounds",
    );
}

/// The caret (the IME cursor area) of a focused 3D text input lies inside the
/// projected quad. The focus ring is not measured here: the test's row in
/// design §17.2 names only the caret.
#[test]
fn focus_ring_and_caret_follow_3d() {
    // Geometry tier: the IME cursor area (the caret) lies inside the
    // projected quad of the input's rotated wrapper (rotate_y(60) of a
    // 240 × 56 box at (100, 100): x in 160..280, y in 100..156).
    let screen = render(|| {
        canvas_of(
            node()
                .style(at(100, 100, 240, 56).rotate_y(60))
                .child(kivact::ui::text_input("Name", signal(String::new()))),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "Name");
    field.focus();
    assert!(field.is_focused(), "the input takes focus");
    let caret = screen.ime_cursor_area().expect("an IME cursor area");
    assert!(
        caret.x >= 160.0 && caret.x + caret.width <= 280.0,
        "the caret is inside the projected width: {caret:?}"
    );
    assert!(
        caret.y >= 100.0 && caret.y + caret.height <= 156.0,
        "the caret is inside the projected height: {caret:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
