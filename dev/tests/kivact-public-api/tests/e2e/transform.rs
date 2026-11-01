//! Role: Tests for CSS transforms and z-index layering.
//!
//! Element transforms (CSS `transform`, `translate`, `rotate`, `scale`,
//! `transform-origin`) and `z-index`: a transform maps the element's box
//! and its whole subtree, and the element is hit, pointed at, reported
//! (`NodeRef::rect`, `Element::rect`) and exposed to assistive technology
//! where it is drawn (G-07); z-index orders painting and hit testing
//! among siblings, also from a hover state style. Checked on the frame's
//! pixels (scale 1) and through real input.

use kivact::view::View;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use kivact::control::canvas;
use kivact::draw::Transform;
use kivact::draw::{Draw, Point, Rect, Size};
use kivact::motion::{Easing, Transition, animated};
use kivact::prelude::*;
use kivact::style::{Align, Color, Justify, pct};
use kivact::style::{Position, TransitionProp};
use kivact::test::{RenderOptions, Screen, Screenshot, render, render_with};
use kivact::view::{Handled, Placement, PointerEvent, overlay};
use kivact::view::{MouseButton, WheelEvent};

use crate::paint::{BLUE, RED, WHITE, at, canvas_of, expect, pixel};
use crate::support::{mount, mount_with};

type Log = Rc<RefCell<Vec<String>>>;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// `left` within `tolerance` of `right`.
#[track_caller]
fn near(left: f32, right: f32, tolerance: f32, what: &str) {
    assert!(
        (left - right).abs() <= tolerance,
        "{what}: {left} is not {right} (±{tolerance})"
    );
}

/// `a` and `b` are the same rect within `tolerance`.
#[track_caller]
fn same_rect(a: Rect, b: Rect, tolerance: f32, what: &str) {
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

/// The bounds of the dark pixels (the text) in `[x0, x1) × [y0, y1)`:
/// `(left, top, right, bottom)`, exclusive.
fn ink(shot: &Screenshot, (x0, y0, x1, y1): (u32, u32, u32, u32)) -> (u32, u32, u32, u32) {
    let mut out = (u32::MAX, u32::MAX, 0, 0);
    for y in y0..y1 {
        for x in x0..x1 {
            if pixel(shot, x, y) & 0xff < 0x60 {
                out = (
                    out.0.min(x),
                    out.1.min(y),
                    out.2.max(x + 1),
                    out.3.max(y + 1),
                );
            }
        }
    }
    out
}

/// A pale button skewed like a game's (`skewX(-30deg)`), its label
/// counter-skewed upright (or left skewed, or neither skewed).
fn skewed_button(skew: bool, counter: bool, log: &Log) -> Node {
    let label = text("SKEW").style(
        style()
            .font_size(28)
            .color(Color::BLACK)
            .transform(Transform::new().skew_x(if counter { 30 } else { 0 })),
    );
    canvas_of(
        pressable("skewed", log).child(label).style(
            at(100, 100, 200, 60)
                .direction(kivact::style::Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .background(rgb(0xffd0d0))
                .transform(Transform::new().skew_x(if skew { -30 } else { 0 })),
        ),
    )
}

#[test]
fn a_skewed_button_with_a_counter_skewed_label_draws_it_upright_and_is_hit_as_drawn() {
    let log: Log = Rc::default();
    let shot = |skew, counter| {
        let l = log.clone();
        render(move || skewed_button(skew, counter, &l)).screenshot()
    };
    let area = (100, 100, 300, 160);
    let upright = ink(&shot(false, false), area);
    let countered = ink(&shot(true, true), area);
    let slanted = ink(&shot(true, false), area);
    let width = |b: (u32, u32, u32, u32)| b.2 - b.0;
    assert!(upright.0 < upright.2, "the reference label is drawn");
    // The two skews cancel: the label is as wide and as tall as upright
    // text, where a skewed one is wider by about its height × tan 30°.
    assert!(
        width(countered).abs_diff(width(upright)) <= 1
            && countered.3 - countered.1 == upright.3 - upright.1,
        "upright {upright:?}, counter-skewed {countered:?}"
    );
    assert!(
        width(slanted) >= width(upright) + 8,
        "a skewed label slants: upright {upright:?}, slanted {slanted:?}"
    );

    let screen = render({
        let l = log.clone();
        move || skewed_button(true, true, &l)
    });
    let s = screen.screenshot();
    // skewX(-30deg) about the centre (200, 130): the top edge moves right
    // by 30 × tan 30° ≈ 17 px, the bottom edge left as much.
    expect(
        &s,
        310,
        105,
        0xffd0d0,
        8,
        "inside the parallelogram, right of the box",
    );
    expect(
        &s,
        102,
        105,
        WHITE,
        8,
        "inside the box, outside the parallelogram",
    );
    log.borrow_mut().clear();
    screen.click_at(102.0, 105.0);
    assert!(log.borrow().is_empty(), "the untransformed box is not hit");
    screen.click_at(310.0, 105.0);
    assert_eq!(*log.borrow(), ["skewed"], "the parallelogram is");
    // The test library acts on the drawn centre.
    screen.get_by_role(Role::Button, "skewed").click();
    assert_eq!(log.borrow().len(), 2);
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

    let half = 20.0 * std::f32::consts::SQRT_2;
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
    zoom: Signal<f32>,
    pan: Signal<(f32, f32)>,
    pad: Rc<RefCell<Vec<Point>>>,
    name: Signal<String>,
    snap: Signal<bool>,
}

fn world() -> (World, impl IntoView) {
    let zoom = signal(2.0f32);
    let pan = signal((30.0f32, 20.0f32));
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
                zoom.update(|z| *z *= if e.delta.y > 0.0 { 0.5 } else { 2.0 });
                Handled::Yes
            })
            .on_pointer(move |e: PointerEvent| {
                use kivact::view::PointerPhase::{Down, Move, Up};
                let mut at = grab.borrow_mut();
                match (e.phase, *at) {
                    (Down, _) => *at = Some(e.pos),
                    (Move, Some(last)) => {
                        pan.update(|p| *p = (p.0 + e.pos.x - last.x, p.1 + e.pos.y - last.y));
                        *at = Some(e.pos);
                    }
                    (Up, _) => *at = None,
                    _ => {}
                }
                Handled::Yes
            })
            .style(at(50, 50, 400, 300).clip(true))
            .child(
                world.child((
                    kivact::control::node()
                        .name("Name")
                        .part(kivact::interaction::use_text_edit(name).control())
                        .test_id("field")
                        .style(at(10, 10, 120, 28)),
                    canvas(|d: &mut Draw, s: Size| d.fill_rect(s.to_rect(), rgb(BLUE)))
                        .name("pad")
                        .test_id("pad")
                        .on_pointer(move |e: PointerEvent| {
                            if e.phase == kivact::view::PointerPhase::Down {
                                seen.borrow_mut().push(e.pos);
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

#[test]
fn an_overlay_anchors_to_the_transformed_bounds_and_is_exposed_there() {
    let screen = render(|| {
        let anchor = node_ref();
        node()
            .child((
                text("anchor")
                    .node_ref(anchor)
                    .test_id("anchor")
                    .style(style().width(60).height(20)),
                overlay(
                    anchor,
                    Placement::Below,
                    text("menu").test_id("menu").style(style().width(50)),
                ),
            ))
            .style(
                style()
                    .size(200, 100)
                    .transform_origin(0, 0)
                    .transform(Transform::new().translate(100, 40).scale(2)),
            )
    });
    let anchor = screen.get_by_test_id("anchor").rect().expect("drawn");
    same_rect(
        anchor,
        Rect::new(100.0, 40.0, 120.0, 40.0),
        0.01,
        "the anchor",
    );
    let menu = screen.get_by_test_id("menu");
    let r = menu.rect().expect("placed");
    near(r.x, anchor.x, 0.01, "the menu starts at the drawn anchor");
    near(
        r.y,
        anchor.y + anchor.height,
        0.01,
        "below the drawn anchor",
    );
    near(
        r.width,
        50.0,
        0.01,
        "an overlay is not scaled by its ancestors",
    );
    same_rect(
        menu.accessibility_bounds().expect("exposed"),
        r,
        0.01,
        "the menu's accessibility bounds (its ancestors' transform undone)",
    );
}

#[test]
fn z_index_orders_painting_and_hits_and_a_hover_state_raises_a_button() {
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || {
            canvas_of((
                // Tree order puts it under b; its hover state raises it.
                pressable("a", &log).style(
                    at(100, 100, 100, 60)
                        .background(rgb(RED))
                        .hover(style().z_index(1)),
                ),
                pressable("b", &log).style(at(150, 100, 100, 60).background(rgb(BLUE))),
                // Last in tree order, but below its siblings.
                pressable("under", &log)
                    .style(at(120, 120, 40, 40).background(rgb(0x00ff00)).z_index(-1)),
                // First in tree order, but above b.
                pressable("over", &log)
                    .style(at(230, 100, 50, 60).background(rgb(0xffff00)).z_index(2)),
            ))
        }
    });
    let s = screen.screenshot();
    expect(&s, 130, 130, RED, 8, "a negative z-index paints below a");
    expect(&s, 175, 130, BLUE, 8, "b over a in tree order");
    expect(&s, 240, 130, 0xffff00, 8, "z-index 2 over b");
    let hit = |x: f32, y: f32| {
        screen.mouse_move(x, y);
        screen.hovered().map(|e| e.name()).unwrap_or_default()
    };
    assert_eq!(hit(130.0, 130.0), "a", "the hit follows the paint order");
    assert_eq!(hit(240.0, 130.0), "over");
    assert_eq!(hit(175.0, 130.0), "b");
    // Hovering a raises it above b (CSS `.btn-group .btn:hover{z-index:1}`).
    screen.mouse_move(110.0, 110.0);
    let s = screen.screenshot();
    expect(&s, 175, 130, RED, 8, "the hovered a is raised over b");
    screen.mouse_move(175.0, 130.0);
    assert_eq!(
        screen.hovered().map(|e| e.name()).unwrap_or_default(),
        "a",
        "and hit over b while hovered"
    );
    // Tab order stays the tree order.
    let order: Vec<String> = (0..4)
        .map(|_| {
            screen.press_tab(false);
            screen.focused().map(|e| e.name()).unwrap_or_default()
        })
        .collect();
    assert_eq!(order, ["a", "b", "under", "over"]);
}

#[test]
fn a_rotation_animates_with_animated_and_with_a_paint_transition() {
    let bounds_at =
        |screen: &Screen, id: &str| screen.get_by_test_id(id).rect().expect("drawn").width;
    let (screen, (on, turned)) = mount(|| {
        let on = signal(false);
        let spin = animated(
            move || if on.get() { 360.0f32 } else { 0.0 },
            Transition::tween(1000, Easing::Linear).repeat(None),
        );
        let turned = signal(false);
        let view = canvas_of((
            node().test_id("spinner").style(move || {
                at(100, 100, 40, 40)
                    .background(rgb(BLUE))
                    .rotate(spin.get())
            }),
            node().test_id("turn").style(move || {
                at(300, 100, 40, 40)
                    .background(rgb(RED))
                    .rotate(if turned.get() { 90 } else { 0 })
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(1000, Easing::Linear),
                    )
            }),
        ));
        ((on, turned), view)
    });
    near(bounds_at(&screen, "spinner"), 40.0, 0.01, "not turned yet");
    on.set(true);
    screen.run_until_idle();
    screen.advance(ms(125));
    // 45° after an eighth of a second: the bounds of the diamond.
    let diagonal = 40.0 * std::f32::consts::SQRT_2;
    near(bounds_at(&screen, "spinner"), diagonal, 1.0, "turned 45°");
    screen.advance(ms(500));
    near(
        bounds_at(&screen, "spinner"),
        diagonal,
        1.0,
        "half a turn later: 225°",
    );
    screen.advance(ms(500));
    near(
        bounds_at(&screen, "spinner"),
        diagonal,
        1.0,
        "a full turn later: 405°",
    );
    assert!(screen.stats().animations >= 1, "it keeps turning");

    turned.set(true);
    screen.run_until_idle();
    screen.advance(ms(500));
    near(
        bounds_at(&screen, "turn"),
        diagonal,
        1.0,
        "the transition halfway: 45°",
    );
    screen.advance(ms(600));
    near(bounds_at(&screen, "turn"), 40.0, 0.01, "at 90°");
}

#[test]
fn reduced_motion_ends_a_decorative_transform_transition_at_once() {
    let opts = RenderOptions {
        reduced_motion: true,
        ..RenderOptions::from_env()
    };
    let (screen, turned) = mount_with(opts, || {
        let turned = signal(false);
        let view = canvas_of(node().test_id("turn").style(move || {
            at(300, 100, 40, 40)
                .background(rgb(RED))
                .rotate(if turned.get() { 45 } else { 0 })
                .transition(
                    TransitionProp::Paint,
                    Transition::tween(1000, Easing::Linear),
                )
        }));
        (turned, view)
    });
    turned.set(true);
    screen.run_until_idle();
    let diagonal = 40.0 * std::f32::consts::SQRT_2;
    let w = screen.get_by_test_id("turn").rect().expect("drawn").width;
    near(w, diagonal, 0.01, "at its target at once");
}

#[test]
fn a_scroll_container_in_a_scaled_subtree_scrolls_and_reveals() {
    let screen = render_with(RenderOptions::from_env(), || {
        let target = node_ref();
        let rows: Vec<View> = (0..30)
            .map(|i| {
                let row = text(format!("row {i}")).style(style().height(20).shrink(0.0));
                if i == 25 {
                    row.node_ref(target).into_view()
                } else {
                    row.into_view()
                }
            })
            .collect();
        node()
            .child((
                scroll(column(rows))
                    .test_id("list")
                    .style(style().size(120, 100).position(Position::Relative)),
                {
                    let label = ("reveal").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || target.scroll_into_view()),
            ))
            .style(style().transform_origin(0, 0).scale(1.5))
    });
    // The wheel over the scaled list scrolls it.
    screen.get_by_test_id("list").scroll(0.0, 40.0);
    assert!(screen.query_by_text("row 1").is_some());
    screen.get_by_role(Role::Button, "reveal").click();
    let row = screen.get_by_text("row 25").rect().expect("drawn");
    let list = screen.get_by_test_id("list").rect().expect("drawn");
    assert!(
        row.y >= list.y - 0.5 && row.y + row.height <= list.y + list.height + 0.5,
        "revealed inside the scaled list: {row:?} in {list:?}"
    );
    near(list.height, 150.0, 0.01, "the list is drawn 1.5× tall");
}

#[test]
fn a_pressed_state_that_moves_one_axis_keeps_the_other_axis_of_the_translate() {
    let log: Log = Rc::default();
    let screen = render({
        let log = log.clone();
        move || {
            canvas_of(
                // Centred on x = 150 by translate, nudged down 4 px while
                // pressed (Tailwind `-translate-x-1/2 active:translate-y-1`).
                pressable("nudged", &log).style(
                    at(150, 100, 100, 40)
                        .background(rgb(RED))
                        .translate_x(pct(-50))
                        .pressed(style().translate_y(4)),
                ),
            )
        }
    });
    let nudged = || screen.get_by_role(Role::Button, "nudged");
    let rest = nudged().rect().expect("laid out");
    same_rect(rest, Rect::new(100.0, 100.0, 100.0, 40.0), 0.5, "at rest");
    screen.mouse_move(150.0, 120.0);
    screen.mouse_down(MouseButton::Primary);
    let s = screen.screenshot();
    expect(
        &s,
        105,
        142,
        RED,
        8,
        "the pressed box keeps its x and moves down",
    );
    expect(&s, 105, 101, WHITE, 8, "the top row it left");
    screen.mouse_up(MouseButton::Primary);
    let s = screen.screenshot();
    expect(&s, 105, 101, RED, 8, "released: back at rest");
    assert_eq!(log.borrow().as_slice(), ["nudged"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
