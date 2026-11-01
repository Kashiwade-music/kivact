//! Role: Input handling contract tests.
//!
//! Input as the window delivers it: one pointer event for the mouse, touch
//! and pen, with the held buttons; mouse presses counted into clicks,
//! touch and pen contacts with pointer ids and kinds of their own, keys and
//! the IME passed on as they come.

use arvane::test_display::VirtualWindow;
use arvane::{ShellEvent, Window as _};
use kivact_types::{
    Key, MouseButton, Point, PointerButtons, PointerId, PointerInput, PointerKind, PointerPhase,
};

use crate::support::open;

const MS: u64 = 1_000_000;

/// The mouse event of `phase` as the window makes and delivers it.
fn mouse_event(
    window: &mut VirtualWindow,
    phase: PointerPhase,
    position: Option<Point>,
    button: Option<MouseButton>,
) -> ShellEvent {
    let event = window.mouse(phase, position, button);
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
fn taps_count_into_double_taps_like_clicks() {
    use PointerKind::{Pen, Touch};
    use PointerPhase::{Down, Move, Up};
    let mut window = open(100.0, 100.0);
    let mut now = 0;
    let mut tap = |window: &mut arvane::test_display::VirtualWindow,
                   after_ms: u64,
                   id: u64,
                   kind: PointerKind,
                   x: f32| {
        now += after_ms * MS;
        window.set_now(now);
        let down = pointer(window.contact(contact(id, kind, Down, x)));
        let moved = pointer(window.contact(contact(id, kind, Move, x)));
        let up = pointer(window.contact(contact(id, kind, Up, x)));
        assert_eq!(
            (moved.clicks, up.clicks),
            (down.clicks, down.clicks),
            "a contact's events carry its Down's count"
        );
        down.clicks
    };
    assert_eq!(tap(&mut window, 0, 7, Touch, 50.0), 1);
    assert_eq!(tap(&mut window, 100, 8, Touch, 53.0), 2, "double tap");
    assert_eq!(tap(&mut window, 100, 7, Pen, 50.0), 3, "triple");
    // Too late, or too far (more than 4 px along an axis).
    assert_eq!(tap(&mut window, 500, 7, Touch, 50.0), 1);
    assert_eq!(tap(&mut window, 100, 7, Touch, 55.0), 1);
    // A mouse click does not count with taps.
    mouse_event(
        &mut window,
        PointerPhase::Move,
        Some(Point { x: 55.0, y: 10.0 }),
        None,
    );
    assert_eq!(clicks(&press(&mut window, MouseButton::Primary)), 1);
    release(&mut window, MouseButton::Primary);
    assert_eq!(tap(&mut window, 100, 7, Touch, 55.0), 2);
}

#[test]
fn a_contact_whose_end_was_lost_is_canceled() {
    use PointerKind::{Pen, Touch};
    use PointerPhase::{Cancel, Down, Move, Up};
    let mut window = open(100.0, 100.0);
    let first = pointer(window.contact(contact(7, Touch, Down, 1.0)));
    pointer(window.contact(contact(7, Touch, Move, 6.0)));
    // A second Down of the platform's id 7: its end was lost, so the old
    // contact is canceled where it was last reported before the new one
    // begins (never two Downs of one pointer, never a contact left open).
    let again = window.contact(contact(7, Touch, Down, 2.0));
    let [ShellEvent::Pointer(canceled), ShellEvent::Pointer(down)] = again.as_slice() else {
        panic!("a cancel and a down: {again:?}");
    };
    assert_eq!(
        (canceled.id, canceled.phase, canceled.position.x),
        (first.id, Cancel, 6.0)
    );
    assert_eq!(
        (down.id, down.phase, down.position.x),
        (PointerId(3), Down, 2.0)
    );
    // Losing the focus (and the pointer capture) cancels every contact in
    // progress before the focus change; their ends then report nothing.
    let pen = pointer(window.contact(contact(8, Pen, Down, 3.0)));
    let lost = window.focus(false);
    let [
        ShellEvent::Pointer(a),
        ShellEvent::Pointer(b),
        ShellEvent::Focused(false),
    ] = lost.as_slice()
    else {
        panic!("two cancels and the focus loss: {lost:?}");
    };
    assert_eq!(
        (a.id, a.phase, b.id, b.phase),
        (down.id, Cancel, pen.id, Cancel)
    );
    assert_eq!(b.kind, Pen);
    assert_eq!(window.contact(contact(7, Touch, Up, 2.0)), []);
    assert_eq!(window.contact(contact(8, Pen, Up, 3.0)), []);
    assert_eq!(window.focus(true), [ShellEvent::Focused(true)]);
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

#[test]
fn the_context_menu_key_is_passed_on_as_the_context_menu_key() {
    let mut window = open(100.0, 100.0);
    let event = ShellEvent::Key {
        key: Key::ContextMenu,
        physical: None,
        code: Default::default(),
        repeat: false,
        text: None,
    };
    assert_eq!(window.input(event.clone()), event);
}

#[test]
fn a_mouse_chord_keeps_its_position_and_buttons_and_each_release_its_click_count() {
    let mut window = open(120.0, 80.0);
    mouse_event(
        &mut window,
        PointerPhase::Move,
        Some(Point { x: 25.0, y: 30.0 }),
        None,
    );
    assert_eq!(clicks(&press(&mut window, MouseButton::Primary)), 1);
    release(&mut window, MouseButton::Primary);
    window.set_now(100 * MS);
    assert_eq!(clicks(&press(&mut window, MouseButton::Primary)), 2);
    let secondary = pointer(vec![press(&mut window, MouseButton::Secondary)]);
    assert!(secondary.buttons.contains(MouseButton::Primary));
    assert!(secondary.buttons.contains(MouseButton::Secondary));
    let moved = pointer(vec![mouse_event(
        &mut window,
        PointerPhase::Move,
        Some(Point { x: 40.0, y: 35.0 }),
        None,
    )]);
    assert_eq!(moved.id, PointerId::MOUSE);
    assert_eq!(moved.kind, PointerKind::Mouse);
    assert_eq!(moved.button, None);
    assert_eq!(moved.buttons, secondary.buttons);
    let left = pointer(vec![mouse_event(
        &mut window,
        PointerPhase::Leave,
        None,
        None,
    )]);
    assert_eq!(left.position, moved.position);
    assert_eq!(left.buttons, moved.buttons);
    let primary_up = pointer(vec![release(&mut window, MouseButton::Primary)]);
    assert_eq!(
        primary_up.clicks, 2,
        "another held button cannot replace this press's click count"
    );
    assert!(!primary_up.buttons.contains(MouseButton::Primary));
    assert!(primary_up.buttons.contains(MouseButton::Secondary));
    let cancelled: Vec<_> = window
        .focus(false)
        .into_iter()
        .filter_map(|event| match event {
            ShellEvent::Pointer(input) => Some(input),
            _ => None,
        })
        .collect();
    assert_eq!(cancelled.len(), 1);
    assert_eq!(cancelled[0].phase, PointerPhase::Cancel);
    assert!(cancelled[0].buttons.is_empty());
    assert_eq!(cancelled[0].button, None);
    // Nothing is held any more: losing the focus again ends nothing.
    assert!(
        window
            .focus(false)
            .iter()
            .all(|event| !matches!(event, ShellEvent::Pointer(_)))
    );
}

#[test]
fn a_pen_contact_keeps_its_pressure_tilt_and_changed_button() {
    let mut window = open(100.0, 100.0);
    let mut input = contact(93, PointerKind::Pen, PointerPhase::Down, 10.0);
    input.tilt = Some((18.0, -12.0));
    let down = pointer(window.contact(input));
    assert_eq!(down.tilt, input.tilt);
    assert_eq!(down.pressure, Some(0.5));
    assert_eq!(down.button, Some(MouseButton::Primary));
    assert!(down.buttons.contains(MouseButton::Primary));
    input.phase = PointerPhase::Move;
    input.button = None;
    input.position.x = 30.0;
    input.pressure = Some(0.8);
    input.tilt = Some((21.0, -9.0));
    let moved = pointer(window.contact(input));
    assert_eq!(moved.id, down.id);
    assert_eq!(
        (moved.position, moved.pressure, moved.tilt),
        (input.position, input.pressure, input.tilt)
    );
    assert_eq!(moved.button, None);
    input.phase = PointerPhase::Up;
    input.button = Some(MouseButton::Primary);
    input.buttons = PointerButtons::NONE;
    let up = pointer(window.contact(input));
    assert_eq!(up.id, down.id);
    assert!(up.buttons.is_empty());
    assert_eq!(up.button, Some(MouseButton::Primary));
}
