//! Role: Tests for CSS transforms and z-index layering.
//!
//! Element transforms (CSS `transform`, `translate`, `rotate`, `scale`,
//! `transform-origin`) and `z-index`: a transform maps the element's box
//! and its whole subtree, and the element is hit, pointed at, reported
//! (`NodeRef::rect`, `Element::rect`) and exposed to assistive technology
//! where it is drawn (G-07); z-index orders painting and hit testing
//! among siblings, also from a hover state style. Checked on the frame's
//! pixels (scale 1) and through real input.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::draw::Draw;
use kivact::draw::Transform;
use kivact::geometry::{Point, Rect, Size};
use kivact::prelude::*;
use kivact::view::canvas;
use kivact::view::{Handled, PointerEvent};
use kivact::view::{MouseButton, WheelEvent};

use crate::paint::{BLUE, WHITE, at, canvas_of, expect};
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

/// A button-like box named `name` that logs its activations.
fn pressable(name: &'static str, log: &Log) -> Node {
    let log = log.clone();
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .on_press(move || log.borrow_mut().push(name.to_string()))
}

#[test]
fn a_rotated_square_is_hit_on_its_diamond_and_reports_its_bounds() {
    let log: Log = Rc::default();
    let (screen, rect) = mount({
        let log = log.clone();
        move || {
            let r = node_ref();
            let diamond = pressable("diamond", &log)
                .node_ref(r)
                .style(at(100, 100, 40, 40).background(rgb(BLUE)).rotate(45));
            (r.rect(), canvas_of(diamond))
        }
    });
    let s = screen.screenshot();
    // The diamond's tips are 20√2 ≈ 28.3 px from the centre (120, 120).
    expect(&s, 120, 93, BLUE, 8, "the top tip, above the layout box");
    expect(&s, 102, 102, WHITE, 8, "the layout box's corner");
    screen.click_at(102.0, 102.0);
    assert!(
        log.borrow().is_empty(),
        "the corner of the layout box is not hit"
    );
    screen.click_at(120.0, 94.0);
    assert_eq!(*log.borrow(), ["diamond"], "the tip is");

    let half = 20.0 * std::f64::consts::SQRT_2;
    let bounds = Rect::new(120.0 - half, 120.0 - half, 2.0 * half, 2.0 * half);
    let drawn = rect.get_untracked().expect("reported");
    same_rect(drawn, bounds, 0.01, "NodeRef::rect is the drawn bounds");
    let element = screen.get_by_role(Role::Button, "diamond");
    same_rect(
        element.rect().expect("laid out"),
        bounds,
        0.01,
        "Element::rect",
    );
    same_rect(
        element.accessibility_bounds().expect("exposed"),
        bounds,
        0.01,
        "accessibility bounds",
    );
}

/// A node editor: a clipping viewport that pans (drag) and zooms (wheel)
/// a world box scaled and moved by them (its origin top-left), holding
/// live widgets.
struct World {
    zoom: Signal<f64>,
    pan: Signal<(f64, f64)>,
    pad: Rc<RefCell<Vec<Point>>>,
    name: Signal<String>,
    snap: Signal<bool>,
}

fn world() -> (World, impl IntoView) {
    let zoom = signal(2.0f64);
    let pan = signal((30.0f64, 20.0f64));
    let name = signal(String::new());
    let snap = signal(false);
    let pad: Rc<RefCell<Vec<Point>>> = Rc::default();
    let seen = pad.clone();
    // Where the drag last was, in the (untransformed) viewport.
    let grab: Rc<RefCell<Option<Point>>> = Rc::default();
    let world = node().test_id("world").style(move || {
        let (x, y) = pan.get();
        style()
            .size(400, 300)
            .background(rgb(0xeeeeee))
            .transform_origin(0, 0)
            .transform(Transform::new().translate(x, y).scale(zoom.get()))
    });
    let view = canvas_of(
        node()
            .test_id("viewport")
            .on_wheel(move |e: WheelEvent| {
                // Wheel down zooms out, up zooms in.
                zoom.update(|z| *z *= if e.delta().y > 0.0 { 0.5 } else { 2.0 });
                Handled::Yes
            })
            .on_pointer(move |e: PointerEvent| {
                use kivact::view::PointerPhase::{Down, Move, Up};
                let mut at = grab.borrow_mut();
                match (e.phase(), *at) {
                    (Down(_), _) => *at = Some(e.pos()),
                    (Move, Some(last)) => {
                        pan.update(|p| *p = (p.0 + e.pos().x - last.x, p.1 + e.pos().y - last.y));
                        *at = Some(e.pos());
                    }
                    (Up(_), _) => *at = None,
                    _ => {}
                }
                Handled::Yes
            })
            .style(at(50, 50, 400, 300).clip(true))
            .child(
                world.child((
                    kivact::view::node()
                        .name("Name")
                        .part(kivact::interaction::use_text_edit(name).control())
                        .test_id("field")
                        .style(at(10, 10, 120, 28)),
                    canvas(|d: &mut Draw, s: Size| d.fill_rect(s.to_rect(), rgb(BLUE)))
                        .name("pad")
                        .test_id("pad")
                        .on_pointer(move |e: PointerEvent| {
                            if matches!(e.phase(), kivact::view::PointerPhase::Down(_)) {
                                seen.borrow_mut().push(e.pos());
                            }
                            Handled::Yes
                        })
                        .style(at(10, 50, 80, 40)),
                    node()
                        .child(
                            node()
                                .role(Role::CheckBox)
                                .focusable(true)
                                .name("Snap")
                                .child(text("Snap"))
                                .checked(snap)
                                .on_press(move || snap.update(|value| *value = !*value)),
                        )
                        .style(at(10, 100, 120, 30)),
                )),
            ),
    );
    (
        World {
            zoom,
            pan,
            pad,
            name,
            snap,
        },
        view,
    )
}

#[test]
fn widgets_in_a_zoomed_world_take_input_where_they_are_drawn() {
    let (screen, w) = mount(world);
    // The world's origin is at (50 + 30, 50 + 20) and its scale 2: the
    // field (10, 10, 120 × 28 in the world) is drawn at (100, 90), 240 × 56.
    let field = screen.get_by_test_id("field");
    same_rect(
        field.rect().expect("drawn"),
        Rect::new(100.0, 90.0, 240.0, 56.0),
        0.01,
        "the field as drawn",
    );
    field.click();
    assert!(field.is_focused(), "a click on the drawn field focuses it");
    field.type_text("hi");
    assert_eq!(w.name.get_untracked(), "hi");
    // The IME places its window at the drawn caret: in the scaled field.
    let ime = screen.ime_text().expect("a focused input");
    let area = ime.area;
    assert!(
        area.x >= 100.0 && area.x + area.width <= 340.5 && area.width > 150.0,
        "the text area as drawn: {area:?}"
    );
    let caret = ime.char_rects.last().copied().expect("the caret");
    assert!(
        caret.height > 20.0 && caret.y >= 90.0 && caret.y + caret.height <= 146.5,
        "the caret as drawn (twice its height): {caret:?}"
    );
    // The candidate window goes where the caret is drawn.
    let cursor = screen.ime_cursor_area().expect("an IME cursor area");
    same_rect(cursor, caret, 0.5, "the IME cursor area");
    // A canvas in the world gets its own untransformed coordinates.
    screen.click_at(100.0 + 2.0 * 5.0, 170.0 + 2.0 * 7.0);
    assert_eq!(*w.pad.borrow(), [Point::new(5.0, 7.0)]);
    screen.get_by_role(Role::CheckBox, "Snap").click();
    assert!(
        w.snap.get_untracked(),
        "the checkbox toggles where it is drawn"
    );

    // Zoom out with the wheel: the world is scaled 1 from (80, 70).
    screen.get_by_test_id("viewport").scroll(0.0, 1.0);
    assert_eq!(w.zoom.get_untracked(), 1.0);
    same_rect(
        screen.get_by_test_id("field").rect().expect("drawn"),
        Rect::new(90.0, 80.0, 120.0, 28.0),
        0.01,
        "the field after zooming out",
    );
    screen.click_at(90.0 + 5.0, 120.0 + 7.0);
    assert_eq!(w.pad.borrow()[1], Point::new(5.0, 7.0));
    // Drag in the pad's own coordinates (the test library maps them).
    screen
        .get_by_test_id("pad")
        .drag(Point::new(2.0, 3.0), Point::new(40.0, 30.0));
    assert_eq!(w.pad.borrow()[2], Point::new(2.0, 3.0));
    // Exposed where drawn.
    same_rect(
        screen
            .get_by_test_id("pad")
            .accessibility_bounds()
            .expect("exposed"),
        Rect::new(90.0, 120.0, 80.0, 40.0),
        0.01,
        "the pad's accessibility bounds",
    );
    let s = screen.screenshot();
    expect(&s, 95, 125, BLUE, 8, "the pad is drawn where it is hit");

    // Pan by dragging the empty world: it moves with the pointer.
    screen.mouse_move(400.0, 300.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(420.0, 310.0);
    screen.mouse_move(440.0, 320.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(w.pan.get_untracked(), (70.0, 40.0));
    same_rect(
        screen.get_by_test_id("field").rect().expect("drawn"),
        Rect::new(130.0, 100.0, 120.0, 28.0),
        0.01,
        "the field after panning",
    );
    screen.click_at(130.0 + 10.0, 140.0 + 10.0);
    assert_eq!(w.pad.borrow()[3], Point::new(10.0, 10.0));
}
