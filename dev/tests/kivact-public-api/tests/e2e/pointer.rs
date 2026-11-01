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

use kivact::draw::Point;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{Screen, render};
use kivact::view::{Handled, KeyEvent, PointerEvent, PointerId};
use kivact::view::{MouseButton, PointerKind, PointerPhase};

type Log = Rc<RefCell<Vec<String>>>;

fn center(screen: &Screen, id: &str) -> (f32, f32) {
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
    let color = |x: f32, y: f32| screen.screenshot().pixel(x as u32, y as u32);
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
                    let what = match e.phase {
                        PointerPhase::Down | PointerPhase::Up => {
                            format!("{:?} {}", e.phase, e.click_count)
                        }
                        phase => format!("{phase:?}"),
                    };
                    if e.phase != PointerPhase::Move {
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

#[test]
fn only_a_primary_press_released_on_the_same_node_activates() {
    let log: Log = Rc::default();
    let out = log.clone();
    let screen = render(move || {
        let (a, b) = (out.clone(), out.clone());
        column((
            {
                let label = ("a").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || a.borrow_mut().push("a".into())),
            {
                let label = ("b").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || b.borrow_mut().push("b".into())),
            // Drawn, and hit, 100 px to the right of where it is laid out.
            node()
                .role(Role::Button)
                .name("moved")
                .test_id("moved")
                .on_press({
                    let o = out.clone();
                    move || o.borrow_mut().push("moved".into())
                })
                .style(style().size(40, 20).translate(100, 0)),
        ))
        .style(style().align_items(Align::Start))
    });
    let a = screen.get_by_role(Role::Button, "a");
    a.click_with(MouseButton::Secondary);
    assert!(a.is_focused(), "any button focuses");
    assert!(log.borrow().is_empty(), "only the primary one activates");
    let (ra, rb) = (
        a.rect().expect("a"),
        screen.get_by_role(Role::Button, "b").rect().expect("b"),
    );
    screen.mouse_move(ra.x + 5.0, ra.y + 5.0);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(rb.x + 5.0, rb.y + 5.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(
        log.borrow().is_empty(),
        "released elsewhere: {:?}",
        log.borrow()
    );

    // Transforms move what is hit with what is drawn (G-07; before element
    // transforms were inherited and hit-tested, the layout box was hit).
    let r = screen.get_by_test_id("moved").rect().expect("drawn");
    screen.click_at(r.x - 90.0, r.y + 10.0);
    assert!(
        log.borrow().is_empty(),
        "the untransformed layout box is not hit"
    );
    screen.click_at(r.x + 10.0, r.y + 10.0);
    assert_eq!(*log.borrow(), ["moved"], "the drawn box is");
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
/// is the mouse, the button and the click count (moves included).
fn contact_log(out: Log) -> impl Fn(PointerEvent) -> Handled + 'static {
    move |e: PointerEvent| {
        let mouse = if e.id == PointerId(1) {
            "mouse"
        } else {
            "own id"
        };
        out.borrow_mut().push(format!(
            "{:?} {:?} {mouse} {:?} {}",
            e.phase, e.kind, e.button, e.click_count
        ));
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
            "Enter Touch own id None 0",
            "Down Touch own id Some(Primary) 1",
            "Up Touch own id Some(Primary) 1",
            "Leave Touch own id None 0",
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

/// A finger that presses a button and is then canceled by the platform (a
/// system gesture took it over), or slides off before it is lifted, does
/// not activate; the listener that got the touch's `Down` gets its
/// `Cancel`.
#[test]
fn a_touch_press_dragged_and_canceled_does_not_activate() {
    let log: Log = Rc::default();
    let presses = Rc::new(RefCell::new(0));
    let (out, count) = (log.clone(), presses.clone());
    let screen = render(move || {
        column((
            node()
                .role(Role::Button)
                .name("go")
                .test_id("go")
                .on_press(move || *count.borrow_mut() += 1)
                .on_pointer(contact_log(out))
                .style(style().size(60, 40)),
            node().test_id("elsewhere").style(style().size(60, 40)),
        ))
        .style(style().align_items(Align::Start))
    });
    let (x, y) = center(&screen, "go");
    let (ex, ey) = center(&screen, "elsewhere");
    let take = || std::mem::take(&mut *log.borrow_mut());

    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.contact_move(contact, x + 5.0, y);
    screen.contact_cancel(contact);
    assert_eq!(*presses.borrow(), 0, "a canceled touch does not activate");
    assert_eq!(
        take(),
        [
            "Enter Touch own id None 0",
            "Down Touch own id Some(Primary) 1",
            "Move Touch own id None 0",
            "Cancel Touch own id None 0",
            "Leave Touch own id None 0",
        ]
    );
    assert!(screen.hovered().is_none());

    // Dragged off the button and lifted elsewhere: the capture keeps the
    // moves and the release on the button's listener, but nothing
    // activates.
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.contact_move(contact, ex, ey);
    screen.contact_up(contact);
    assert_eq!(*presses.borrow(), 0, "released elsewhere");
    let events = take();
    assert!(
        events.iter().any(|e| e.starts_with("Up Touch")),
        "{events:?}"
    );

    // The mouse elsewhere does not disturb a finger that is down.
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.mouse_move(ex, ey);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    screen.contact_up(contact);
    assert_eq!(*presses.borrow(), 1, "the finger's tap still counts");
    assert!(
        take().iter().all(|e| !e.contains("mouse")),
        "the mouse reached no listener of the finger's"
    );
}

/// A finger on a text field focuses it and puts the caret where it
/// touches, as a click does, and the software keyboard comes up with it
/// (and goes when the focus leaves the field).
#[test]
fn a_touch_focuses_a_text_field_and_shows_the_soft_keyboard() {
    let screen = render(|| {
        let value = signal("hello".to_string());
        column((
            kivact::control::node()
                .name(value)
                .part(kivact::interaction::use_text_edit(kivact::signal(String::new())).control())
                .test_id("field"),
            {
                let label = ("other").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
        ))
        .style(style().align_items(Align::Start))
    });
    assert!(
        !screen.soft_keyboard_shown(),
        "no keyboard before a field has the focus"
    );
    let field = screen.get_by_test_id("field");
    field.tap();
    assert!(field.is_focused(), "the tap focused the field");
    assert!(
        screen.soft_keyboard_shown(),
        "the keyboard came up with the focus"
    );
    field.type_text("!");
    assert!(
        field.value().is_some_and(|v| v.contains('!')),
        "{:?}",
        field.value()
    );
    assert_eq!(
        screen.soft_keyboard_requests(),
        1,
        "typing asks nothing more"
    );
    screen.get_by_role(Role::Button, "other").tap();
    assert!(!field.is_focused(), "a tap elsewhere moved the focus");
    assert!(
        !screen.soft_keyboard_shown(),
        "the keyboard went with the focus"
    );
    // Focus by keyboard or assistive technology asks for it too.
    field.focus();
    assert!(screen.soft_keyboard_shown());
    assert_eq!(screen.soft_keyboard_requests(), 3);
}

/// A pen presses like a finger (and a mouse's primary button): its events
/// say it is a pen, it activates on release and leaves no hover behind.
#[test]
fn a_pen_presses_like_a_finger_and_says_it_is_a_pen() {
    let log: Log = Rc::default();
    let presses = Rc::new(RefCell::new(0));
    let (out, count) = (log.clone(), presses.clone());
    let screen = render(move || {
        column((node()
            .role(Role::Button)
            .name("ink")
            .on_press(move || *count.borrow_mut() += 1)
            .on_pointer(contact_log(out))
            .style(style().size(60, 40)),))
        .style(style().align_items(Align::Start))
    });
    screen
        .get_by_role(Role::Button, "ink")
        .tap_with(PointerKind::Pen);
    assert_eq!(*presses.borrow(), 1);
    assert_eq!(
        std::mem::take(&mut *log.borrow_mut()),
        [
            "Enter Pen own id None 0",
            "Down Pen own id Some(Primary) 1",
            "Up Pen own id Some(Primary) 1",
            "Leave Pen own id None 0",
        ]
    );
    assert!(screen.hovered().is_none());
    // The mouse is the mouse again afterwards.
    screen.get_by_role(Role::Button, "ink").click();
    assert_eq!(*presses.borrow(), 2);
    assert!(
        log.borrow()
            .iter()
            .any(|e| e == "Down Mouse mouse Some(Primary) 1"),
        "{:?}",
        log.borrow()
    );
}

/// A contact does not disturb a mouse resting on the window: each pointer
/// enters and leaves the listener on its own, the finger leaves no hover
/// behind, and the mouse goes on hovering where it is.
#[test]
fn a_touch_leaves_the_hover_of_a_resting_mouse_alone() {
    let log: Log = Rc::default();
    let out = log.clone();
    let screen = render(move || {
        column((node()
            .role(Role::Button)
            .name("go")
            .test_id("go")
            .on_pointer(contact_log(out))
            .style(style().size(60, 40)),))
        .style(style().align_items(Align::Start))
    });
    let (x, y) = center(&screen, "go");
    let take = || std::mem::take(&mut *log.borrow_mut());
    screen.mouse_move(x, y);
    assert_eq!(take(), ["Enter Mouse mouse None 0"]);
    let contact = screen.contact_down(PointerKind::Touch, x + 2.0, y);
    screen.contact_up(contact);
    assert_eq!(
        take(),
        [
            "Enter Touch own id None 0",
            "Down Touch own id Some(Primary) 1",
            "Up Touch own id Some(Primary) 1",
            "Leave Touch own id None 0",
        ]
    );
    assert_eq!(
        screen.hovered().map(|e| e.name()).as_deref(),
        Some("go"),
        "the resting mouse still hovers"
    );
    // A canceled contact leaves the mouse alone too; once the mouse left
    // the window, a tap hovers nothing afterwards.
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.contact_cancel(contact);
    assert_eq!(
        take().last().map(String::as_str),
        Some("Leave Touch own id None 0")
    );
    screen.mouse_leave();
    take();
    screen.get_by_test_id("go").tap();
    assert!(screen.hovered().is_none(), "no mouse in the window");
    assert!(take().iter().all(|e| !e.contains("mouse")));
}

/// Two quick taps at one spot are a double tap: the second contact's
/// events carry a click count of 2, as a double click's do; a late tap
/// starts over.
#[test]
fn a_double_tap_counts_like_a_double_click() {
    let log: Log = Rc::default();
    let out = log.clone();
    let screen = render(move || {
        column((node()
            .role(Role::Button)
            .name("go")
            .on_pointer(contact_log(out))
            .style(style().size(60, 40)),))
        .style(style().align_items(Align::Start))
    });
    let go = screen.get_by_role(Role::Button, "go");
    let downs = || {
        std::mem::take(&mut *log.borrow_mut())
            .into_iter()
            .filter(|e| e.starts_with("Down") || e.starts_with("Up"))
            .collect::<Vec<_>>()
    };
    go.tap();
    go.tap();
    assert_eq!(
        downs(),
        [
            "Down Touch own id Some(Primary) 1",
            "Up Touch own id Some(Primary) 1",
            "Down Touch own id Some(Primary) 2",
            "Up Touch own id Some(Primary) 2",
        ]
    );
    screen.advance(Duration::from_millis(600));
    go.tap();
    assert_eq!(
        downs(),
        [
            "Down Touch own id Some(Primary) 1",
            "Up Touch own id Some(Primary) 1",
        ],
        "too late for a double tap"
    );
}

/// `KeyEvent::pointer` is where the mouse is in the window: `None` before
/// it entered, its position inside (also over a capturing listener and
/// while a button is held), `None` once it left (also with a button held),
/// and a touch contact does not set it. A key release carries it too.
#[test]
fn keys_carry_the_mouse_position_while_it_is_in_the_window() {
    let seen: Rc<RefCell<Vec<Option<Point>>>> = Rc::default();
    let released: Rc<RefCell<Vec<Option<Point>>>> = Rc::default();
    let (log, up_log) = (seen.clone(), released.clone());
    let screen = render(move || {
        let (log, up_log) = (log.clone(), up_log.clone());
        node()
            .test_id("field")
            .role(Role::Group)
            .name("field")
            .focusable(true)
            .style(style().size(200, 100))
            .on_key(move |e: KeyEvent| {
                if e.key == Key::Char('q') {
                    log.borrow_mut().push(e.pointer);
                }
            })
            .on_key_up(move |e: KeyEvent| {
                if e.key == Key::Char('q') {
                    up_log.borrow_mut().push(e.pointer);
                }
            })
            .child(
                node()
                    .test_id("listener")
                    .on_pointer(|_| Handled::Yes)
                    .style(style().size(60, 60)),
            )
    });
    screen.get_by_test_id("field").focus();
    screen.press_key(Key::Char('q'));
    let field = screen.get_by_test_id("field").rect().expect("laid out");
    let inside = Point::new(field.x + 150.0, field.y + 80.0);
    screen.mouse_move(inside.x, inside.y);
    screen.press_key(Key::Char('q'));
    let (x, y) = center(&screen, "listener");
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.press_key(Key::Char('q'));
    screen.mouse_leave();
    screen.press_key(Key::Char('q'));
    screen.mouse_up(MouseButton::Primary);
    let contact = screen.contact_down(PointerKind::Touch, inside.x, inside.y);
    screen.press_key(Key::Char('q'));
    screen.contact_up(contact);
    assert_eq!(
        *seen.borrow(),
        vec![None, Some(inside), Some(Point::new(x, y)), None, None],
        "before entering, inside, over a capturing listener with a button held,          after leaving with the button held, under a touch contact"
    );
    assert_eq!(
        *released.borrow(),
        *seen.borrow(),
        "a release carries the same"
    );
    assert!(screen.get_by_test_id("field").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn handlers_get_the_held_buttons_and_the_changed_button_of_every_pointer() {
    let events: Rc<RefCell<Vec<PointerEvent>>> = Rc::default();
    let record = events.clone();
    let screen = render(move || {
        node()
            .test_id("unified")
            .style(style().size(120, 80))
            .on_pointer(move |event: PointerEvent| {
                record.borrow_mut().push(event);
                Handled::No
            })
    });
    let (x, y) = center(&screen, "unified");
    // A chord held across leaving the window stays captured until its last
    // release; a move names no button, only the ones held.
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_down(MouseButton::Secondary);
    screen.mouse_move(x + 160.0, y);
    screen.mouse_leave();
    screen.mouse_up(MouseButton::Primary);
    screen.mouse_move(x + 165.0, y);
    screen.mouse_up(MouseButton::Secondary);
    {
        let events = events.borrow();
        assert!(
            events
                .iter()
                .any(|event| event.phase == PointerPhase::Enter)
        );
        assert!(
            events
                .iter()
                .filter(|event| event.phase == PointerPhase::Move)
                .all(|event| event.button.is_none())
        );
        assert!(events.iter().any(|event| event.phase == PointerPhase::Move
            && event.buttons.contains(MouseButton::Primary)
            && event.buttons.contains(MouseButton::Secondary)));
        let primary_up = events
            .iter()
            .find(|event| {
                event.phase == PointerPhase::Up && event.button == Some(MouseButton::Primary)
            })
            .expect("captured primary release");
        assert!(!primary_up.buttons.contains(MouseButton::Primary));
        assert!(primary_up.buttons.contains(MouseButton::Secondary));
        let secondary_up = events
            .iter()
            .find(|event| {
                event.phase == PointerPhase::Up && event.button == Some(MouseButton::Secondary)
            })
            .expect("capture survives the first release");
        assert!(secondary_up.buttons.is_empty());
    }
    // A contact holds the primary button from its Down to its Up.
    for kind in [PointerKind::Touch, PointerKind::Pen] {
        events.borrow_mut().clear();
        let contact = screen.contact_down(kind, x, y);
        screen.contact_move(contact, x + 15.0, y + 10.0);
        screen.contact_up(contact);
        let log = events.borrow();
        let down = log
            .iter()
            .find(|event| event.phase == PointerPhase::Down)
            .expect("contact Down");
        assert_eq!(down.kind, kind);
        assert_ne!(down.id, PointerId::MOUSE);
        assert_eq!(down.button, Some(MouseButton::Primary));
        assert!(down.buttons.contains(MouseButton::Primary));
        let moved = log
            .iter()
            .find(|event| event.phase == PointerPhase::Move && event.window_pos.x == x + 15.0)
            .expect("contact Move");
        assert_eq!(moved.id, down.id);
        assert_eq!(moved.button, None);
        assert!(moved.buttons.contains(MouseButton::Primary));
        let up = log
            .iter()
            .find(|event| event.phase == PointerPhase::Up)
            .expect("contact Up");
        assert_eq!(up.id, down.id);
        assert!(up.buttons.is_empty());
        assert!(log.iter().any(|event| event.phase == PointerPhase::Leave));
    }
    // A Cancel and the Leave after it hold nothing.
    for kind in [PointerKind::Touch, PointerKind::Pen] {
        events.borrow_mut().clear();
        let contact = screen.contact_down(kind, x, y);
        screen.contact_cancel(contact);
        let log = events.borrow();
        let cancel = log
            .iter()
            .find(|event| event.phase == PointerPhase::Cancel)
            .expect("contact Cancel");
        assert!(cancel.buttons.is_empty());
        assert_eq!(cancel.button, None);
        assert!(
            log.iter()
                .filter(|event| event.phase == PointerPhase::Leave)
                .all(|event| event.buttons.is_empty())
        );
    }
}
