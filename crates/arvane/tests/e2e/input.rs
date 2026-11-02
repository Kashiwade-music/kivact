//! Role: Input handling contract tests.
//!
//! Input as the window delivers it: one pointer event for the mouse, touch
//! and pen, with the held buttons; mouse presses counted into clicks,
//! touch and pen contacts with pointer ids and kinds of their own, keys and
//! the IME passed on as they come.

use arvane::test_display::VirtualWindow;
use arvane::{ShellEvent, Window as _};
use kivact_types::protocol::PointerInput;
use kivact_types::{Key, MouseButton, Point, PointerButtons, PointerId, PointerKind, PointerPhase};

use crate::support::open;

const MS: u64 = 1_000_000;

/// The mouse event of `phase` as the window makes and delivers it.
fn mouse_event(
    window: &mut VirtualWindow,
    phase: PointerPhase,
    position: Option<Point>,
    button: Option<MouseButton>,
) -> ShellEvent {
    let event = window
        .mouse(phase, position, button)
        .expect("the test placed the mouse first");
    window.input(event)
}

fn press(window: &mut VirtualWindow, button: MouseButton) -> ShellEvent {
    mouse_event(window, PointerPhase::Down, None, Some(button))
}

fn release(window: &mut VirtualWindow, button: MouseButton) -> ShellEvent {
    mouse_event(window, PointerPhase::Up, None, Some(button))
}

fn clicks(event: &ShellEvent) -> u8 {
    match event {
        ShellEvent::Pointer(input) => input.clicks,
        other => panic!("not a button: {other:?}"),
    }
}

#[test]
fn presses_count_into_double_and_triple_clicks_within_time_and_distance() {
    let mut window = open(200.0, 200.0);
    let mouse = window.environment().mouse;
    assert_eq!(mouse.double_click_ns, 500 * MS);
    assert_eq!(mouse.double_click_distance, (4.0, 4.0));
    let mut at = 0;
    let mut click = |window: &mut arvane::test_display::VirtualWindow,
                     after_ms: u64,
                     x: f32,
                     y: f32,
                     button: MouseButton| {
        at += after_ms * MS;
        window.set_now(at);
        mouse_event(window, PointerPhase::Move, Some(Point { x, y }), None);
        let down = clicks(&press(window, button));
        let up = clicks(&release(window, button));
        assert_eq!(up, down, "a release carries its press's count");
        down
    };
    let primary = MouseButton::Primary;
    assert_eq!(click(&mut window, 0, 50.0, 50.0, primary), 1);
    assert_eq!(click(&mut window, 100, 52.0, 47.0, primary), 2, "double");
    assert_eq!(click(&mut window, 100, 50.0, 50.0, primary), 3, "triple");
    // Too late.
    assert_eq!(click(&mut window, 500, 50.0, 50.0, primary), 1);
    // Too far (more than 4 px along an axis).
    assert_eq!(click(&mut window, 100, 55.0, 50.0, primary), 1);
    assert_eq!(click(&mut window, 100, 55.0, 54.0, primary), 2);
    // Another button starts over.
    assert_eq!(
        click(&mut window, 100, 55.0, 54.0, MouseButton::Secondary),
        1
    );
    // Other pointer events pass unchanged.
    let wheel = ShellEvent::Wheel { x: 0.0, y: -40.0 };
    assert_eq!(window.input(wheel.clone()), wheel);
}

fn contact(id: u64, kind: PointerKind, phase: PointerPhase, x: f32) -> PointerInput {
    PointerInput {
        id: PointerId(id),
        kind,
        phase,
        position: Point { x, y: 10.0 },
        buttons: PointerButtons::NONE.with(
            MouseButton::Primary,
            matches!(phase, PointerPhase::Down | PointerPhase::Move),
        ),
        button: matches!(phase, PointerPhase::Down | PointerPhase::Up)
            .then_some(MouseButton::Primary),
        pressure: Some(0.5),
        tilt: None,
        clicks: 0,
    }
}

fn pointer(events: Vec<ShellEvent>) -> PointerInput {
    match events.as_slice() {
        [ShellEvent::Pointer(input)] => *input,
        other => panic!("not one pointer event: {other:?}"),
    }
}

#[test]
fn contacts_get_pointer_ids_and_kinds_of_their_own() {
    use PointerKind::{Mouse, Pen, Touch};
    use PointerPhase::{Cancel, Down, Move, Up};
    let mut window = open(100.0, 100.0);
    assert!(window.capabilities().touch && window.capabilities().pen);
    // A finger: the platform's id 7 becomes pointer 2 (1 is the mouse),
    // kept until it lifts.
    let down = pointer(window.contact(contact(7, Touch, Down, 1.0)));
    assert_eq!(
        (down.id, down.kind, down.phase),
        (PointerId(2), Touch, Down)
    );
    assert_eq!(
        (down.position.x, down.position.y, down.pressure),
        (1.0, 10.0, Some(0.5))
    );
    let moved = pointer(window.contact(contact(7, Pen, Move, 2.0)));
    assert_eq!(
        (moved.id, moved.kind, moved.phase, moved.position.x),
        (PointerId(2), Touch, Move, 2.0),
        "the kind is the one of the contact's Down"
    );
    // A pen meanwhile is another pointer.
    let pen = pointer(window.contact(contact(8, Pen, Down, 3.0)));
    assert_eq!((pen.id, pen.kind), (PointerId(3), Pen));
    let up = pointer(window.contact(contact(7, Touch, Up, 2.0)));
    assert_eq!((up.id, up.phase), (PointerId(2), Up));
    let canceled = pointer(window.contact(contact(8, Pen, Cancel, 3.0)));
    assert_eq!(
        (canceled.id, canceled.kind, canceled.phase),
        (PointerId(3), Pen, Cancel)
    );
    // The platform reuses id 7: a new contact, a new pointer.
    let again = pointer(window.contact(contact(7, Touch, Down, 4.0)));
    assert_eq!(again.id, PointerId(4));
    // A contact is never the mouse.
    let mouse = pointer(window.contact(contact(9, Mouse, Down, 5.0)));
    assert_eq!((mouse.id, mouse.kind), (PointerId(5), Touch));
    // Nothing is reported for a contact that never began (a hovering pen).
    assert_eq!(window.contact(contact(40, Pen, Move, 1.0)), []);
    assert_eq!(window.contact(contact(41, Touch, Up, 1.0)), []);
}

#[test]
fn keys_and_the_ime_pass_through_in_order() {
    let mut window = open(100.0, 100.0);
    let events = [
        ShellEvent::Key {
            key: Key::Char('ф'),
            physical: Some('a'),
            code: Default::default(),
            repeat: false,
            text: Some("ф".into()),
        },
        ShellEvent::Key {
            key: Key::Enter,
            physical: None,
            code: Default::default(),
            repeat: true,
            text: None,
        },
        ShellEvent::Preedit {
            text: "にほ".into(),
            selection: Some((6, 6)),
        },
        ShellEvent::Preedit {
            text: "日本".into(),
            selection: None,
        },
        ShellEvent::Commit("日本".into()),
        ShellEvent::ImeSelection { start: 0, end: 2 },
        ShellEvent::ImeDisabled,
    ];
    let delivered: Vec<ShellEvent> = events.iter().cloned().map(|e| window.input(e)).collect();
    assert_eq!(delivered, events);
}
