//! Role: Tests for pointer states and event handling.
//!
//! The pointer on real windows: state styles stack focus < hover <
//! pressed (pressed only while hovered), Enter and Leave go to the nearest
//! listener, clicks are counted within time and distance, only a primary
//! press that is released on the same node activates, a release outside
//! the window ends the press, and a transform moves the drawing, not the
//! hit area. A touch or pen contact presses like the primary button,
//! hovers only while it lasts, ends without activating when canceled, and
//! leaves the mouse's hover alone (several pointers at once: `multi_touch`).
//! Key events carry the mouse's window position while it is in the window.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{Screen, render};
use kivact::view::{Handled, PointerEvent, PointerId};
use kivact::view::{MouseButton, PointerKind, PointerPhase};

type Log = Rc<RefCell<Vec<String>>>;

fn center(screen: &Screen, id: &str) -> (f64, f64) {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    (r.x + r.width / 2.0, r.y + r.height / 2.0)
}

#[test]
fn state_styles_stack_focus_hover_and_pressed_while_hovered() {
    let screen = render(|| {
        column((
            node()
                .role(Role::Button)
                .name("state")
                .test_id("state")
                .focusable(true)
                .on_press(|| {})
                .style(
                    style()
                        .size(40, 40)
                        .background(rgb(0x101010))
                        .focus(style().background(rgb(0x00a000)))
                        .hover(style().background(rgb(0x0000c0)))
                        .pressed(style().background(rgb(0xc00000))),
                ),
            // Only a pressed style: it still shows while pressed.
            node()
                .test_id("pressed only")
                .on_pointer(|_| Handled::Yes)
                .style(
                    style()
                        .size(40, 40)
                        .background(rgb(0x101010))
                        .pressed(style().background(rgb(0xc00000))),
                ),
        ))
        .style(style().align_items(Align::Start))
    });
    let (x, y) = center(&screen, "state");
    let color = |x: f64, y: f64| screen.screenshot().pixel(x as u32, y as u32);
    assert_eq!(color(x, y), Some(0x101010));
    screen.get_by_test_id("state").focus();
    assert_eq!(color(x, y), Some(0x00a000), "focus");
    screen.mouse_move(x, y);
    assert_eq!(color(x, y), Some(0x0000c0), "hover over focus");
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(color(x, y), Some(0xc00000), "pressed over hover");
    screen.mouse_move(x, y + 100.0);
    assert_eq!(color(x, y), Some(0x00a000), "pressed only while hovered");
    screen.mouse_move(x, y);
    assert_eq!(color(x, y), Some(0xc00000), "back over it");
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(color(x, y), Some(0x0000c0), "released, still hovered");
    screen.mouse_leave();
    assert_eq!(color(x, y), Some(0x00a000));

    let (px, py) = center(&screen, "pressed only");
    screen.mouse_move(px, py);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(color(px, py), Some(0xc00000));
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(color(px, py), Some(0x101010));
}

#[test]
fn enter_leave_click_counts_and_the_release_outside_the_window() {
    let log: Log = Rc::default();
    let out = log.clone();
    let screen = render(move || {
        let o = out.clone();
        column((
            node()
                .test_id("listener")
                .on_pointer(move |e: PointerEvent| {
                    let what = match e.phase() {
                        PointerPhase::Down(press) => format!("Down {}", press.clicks),
                        PointerPhase::Up(press) => format!("Up {}", press.clicks),
                        phase => format!("{phase:?}"),
                    };
                    if e.phase() != PointerPhase::Move {
                        o.borrow_mut().push(what);
                    }
                    Handled::Yes
                })
                .style(style().size(200, 60))
                .child(row((
                    node().test_id("left").style(style().size(60, 40)),
                    node().test_id("right").style(style().size(60, 40)),
                ))),
            node().test_id("outside").style(style().size(60, 40)),
        ))
        .style(style().align_items(Align::Start))
    });
    let (lx, ly) = center(&screen, "left");
    let (rx, ry) = center(&screen, "right");
    let take = || std::mem::take(&mut *log.borrow_mut());
    screen.mouse_move(lx, ly);
    screen.mouse_move(rx, ry);
    assert_eq!(
        take(),
        ["Enter"],
        "moving between its children sends nothing"
    );
    let (ox, oy) = center(&screen, "outside");
    screen.mouse_move(ox, oy);
    assert_eq!(take(), ["Leave"]);

    screen.get_by_test_id("left").double_click();
    assert_eq!(take(), ["Enter", "Down 1", "Up 1", "Down 2", "Up 2"]);
    screen.advance(Duration::from_millis(600));
    screen.click_at(lx, ly);
    assert_eq!(take(), ["Down 1", "Up 1"], "too late for a double click");
    screen.click_at(lx + 30.0, ly);
    assert_eq!(take(), ["Down 1", "Up 1"], "too far for a double click");

    // A press held while the pointer leaves the window still ends there.
    screen.mouse_move(lx, ly);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(-50.0, -50.0);
    screen.mouse_up(MouseButton::Primary);
    let ended = take();
    assert!(
        ended.first().is_some_and(|e| e.starts_with("Down"))
            && ended.iter().any(|e| e.starts_with("Up") || e == "Cancel"),
        "{ended:?}"
    );
}

/// The mouse cursor follows what is under the pointer, nearest first: the
/// text cursor over an enabled field (and its text), the pointing hand
/// over something a click activates (and its content), the arrow over a
/// disabled one, one with nothing to activate and over nothing in
/// particular.
#[test]
fn the_cursor_shows_text_fields_and_what_a_click_activates() {
    let screen = render(|| {
        column((
            kivact::control::node()
                .name("Name")
                .part(kivact::interaction::use_text_edit(signal(String::from("value"))).control()),
            {
                let label = ("Save").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(|| {}),
            {
                let label = ("Off").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(|| {})
            .disabled(true),
            {
                let label = ("Idle").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
            text("plain").test_id("plain"),
        ))
        .style(style().gap(8).align_items(Align::Start))
    });
    assert_eq!(screen.cursor(), kivact::style::Cursor::Default);
    screen.get_by_role(Role::TextBox, "Name").hover();
    assert_eq!(screen.cursor(), kivact::style::Cursor::Text);
    screen.get_by_role(Role::Button, "Save").hover();
    assert_eq!(screen.cursor(), kivact::style::Cursor::Pointer);
    screen.get_by_text("Save").hover();
    assert_eq!(
        screen.cursor(),
        kivact::style::Cursor::Pointer,
        "over the button's label"
    );
    let off = screen
        .get_by_role(Role::Button, "Off")
        .rect()
        .expect("laid out");
    screen.mouse_move(off.x + off.width / 2.0, off.y + off.height / 2.0);
    assert_eq!(screen.cursor(), kivact::style::Cursor::Default, "disabled");
    screen.get_by_role(Role::Button, "Idle").hover();
    assert_eq!(
        screen.cursor(),
        kivact::style::Cursor::Default,
        "a button without a press handler activates nothing"
    );
    screen.get_by_test_id("plain").hover();
    assert_eq!(screen.cursor(), kivact::style::Cursor::Default);
    screen.mouse_leave();
    assert_eq!(screen.cursor(), kivact::style::Cursor::Default);
}

/// A pointer listener's log of the events it gets: phase, kind, whether it
/// is the mouse (a press's button and click count are in the phase).
fn contact_log(out: Log) -> impl Fn(PointerEvent) -> Handled + 'static {
    move |e: PointerEvent| {
        let mouse = if e.id() == PointerId(1) {
            "mouse"
        } else {
            "own id"
        };
        out.borrow_mut()
            .push(format!("{:?} {:?} {mouse}", e.phase(), e.kind()));
        Handled::No
    }
}

/// A finger taps like a click: the button activates (once), the press shows
/// its pressed style while the finger is down, and nothing stays hovered
/// afterwards (a finger leaves no hover behind).
#[test]
fn a_touch_tap_activates_a_button_and_leaves_no_hover() {
    let log: Log = Rc::default();
    let presses = Rc::new(RefCell::new(0));
    let (out, count) = (log.clone(), presses.clone());
    let screen = render(move || {
        column((node()
            .role(Role::Button)
            .name("go")
            .test_id("go")
            .focusable(true)
            .on_press(move || *count.borrow_mut() += 1)
            .on_pointer(contact_log(out))
            .style(
                style()
                    .size(60, 40)
                    .background(rgb(0x101010))
                    .hover(style().background(rgb(0x0000c0)))
                    .pressed(style().background(rgb(0xc00000))),
            ),))
        .style(style().align_items(Align::Start))
    });
    let go = screen.get_by_role(Role::Button, "go");
    go.tap();
    assert_eq!(*presses.borrow(), 1, "the tap activated the button");
    assert!(go.is_focused(), "a tap focuses like a press");
    assert!(
        screen.hovered().is_none(),
        "no hover after the finger is lifted"
    );
    assert_eq!(
        std::mem::take(&mut *log.borrow_mut()),
        [
            "Enter Touch own id",
            "Down(ButtonPress { button: Primary, clicks: 1 }) Touch own id",
            "Up(ButtonPress { button: Primary, clicks: 1 }) Touch own id",
            "Leave Touch own id",
        ]
    );

    // While the finger is down the button shows its pressed style; lifted,
    // it shows neither pressed nor hover.
    let (x, y) = center(&screen, "go");
    let color = || screen.screenshot().pixel(x as u32, y as u32);
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    assert_eq!(color(), Some(0xc00000), "pressed while touched");
    screen.contact_up(contact);
    assert_eq!(color(), Some(0x101010), "no hover style after the tap");
    assert_eq!(*presses.borrow(), 2);
}
