//! Role: Input handling contract tests.
//!
//! Input as the window delivers it: one pointer event for the mouse, touch
//! and pen, with the held buttons; mouse presses counted into clicks,
//! touch and pen contacts with pointer ids and kinds of their own, keys and
//! the IME passed on as they come.

use crate::support::Harness;
use arvane::ShellEvent;
use kivact_types::PointerSample;
use kivact_types::{
    ButtonPress, Key, MouseButton, Point, PointerButtons, PointerId, PointerKind, PointerPhase,
};

use crate::support::open;

const MS: u64 = 1_000_000;

/// The mouse event of `phase` as the window makes and delivers it.
fn mouse_event(window: &mut Harness, phase: PointerPhase, position: Option<Point>) -> ShellEvent {
    window.script().mouse(phase, position);
    window
        .events()
        .into_iter()
        .next()
        .expect("the test placed the mouse first")
}

fn press(window: &mut Harness, button: MouseButton) -> ShellEvent {
    mouse_event(
        window,
        PointerPhase::Down(ButtonPress { button, clicks: 1 }),
        None,
    )
}

fn release(window: &mut Harness, button: MouseButton) -> ShellEvent {
    mouse_event(
        window,
        PointerPhase::Up(ButtonPress { button, clicks: 1 }),
        None,
    )
}

fn clicks(event: &ShellEvent) -> u8 {
    match event {
        ShellEvent::Pointer(input) => {
            input
                .phase
                .press()
                .expect("invariant: a press or release")
                .clicks
        }
        other => panic!("not a button: {other:?}"),
    }
}

#[test]
fn presses_count_into_double_and_triple_clicks_within_time_and_distance() {
    let mut window = open(200.0, 200.0);
    let mouse = window.probe().environment().pointer;
    assert_eq!(mouse.double_click_ns, 500 * MS);
    assert_eq!(mouse.double_click_distance, (4.0, 4.0));
    let mut at = 0;
    let mut click = |window: &mut Harness, after_ms: u64, x: f64, y: f64, button: MouseButton| {
        at += after_ms * MS;
        window.script().set_now(at);
        mouse_event(window, PointerPhase::Move, Some(Point { x, y }));
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
    let wheel = ShellEvent::Wheel(kivact_types::WheelInput {
        delta: kivact_types::Point { x: 0.0, y: -40.0 },
        position: kivact_types::Point::default(),
        modifiers: kivact_types::Modifiers::default(),
    });
    window.script().input(wheel.clone());
    assert_eq!(window.events(), [wheel]);
}

fn contact(id: u64, kind: PointerKind, phase: PointerPhase, x: f64) -> PointerSample {
    PointerSample {
        id: PointerId(id),
        kind,
        phase,
        position: Point { x, y: 10.0 },
        buttons: PointerButtons::NONE.with(
            MouseButton::Primary,
            matches!(phase, PointerPhase::Down(_) | PointerPhase::Move),
        ),
        pressure: Some(0.5),
        tilt: None,
        button: None,
        press: None,
        time_ns: 0,
        modifiers: kivact_types::Modifiers::default(),
        is_primary: true,
    }
}

const SINGLE: ButtonPress = ButtonPress {
    button: MouseButton::Primary,
    clicks: 1,
};
/// The `Down` and `Up` of a single click of the primary button.
const DOWN: PointerPhase = PointerPhase::Down(SINGLE);
const UP: PointerPhase = PointerPhase::Up(SINGLE);

fn pointer(events: Vec<ShellEvent>) -> PointerSample {
    match events.as_slice() {
        [ShellEvent::Pointer(input)] => *input,
        other => panic!("not one pointer event: {other:?}"),
    }
}

#[test]
fn contacts_get_pointer_ids_and_kinds_of_their_own() {
    use PointerKind::{Mouse, Pen, Touch};
    use PointerPhase::{Cancel, Move};
    let mut window = open(100.0, 100.0);
    assert!(window.probe().capabilities().touch && window.probe().capabilities().pen);
    // A finger: the platform's id 7 becomes pointer 2 (1 is the mouse),
    // kept until it lifts.
    let down = pointer(contact_events(&mut window, contact(7, Touch, DOWN, 1.0)));
    assert_eq!(
        (down.id, down.kind, down.phase),
        (PointerId(2), Touch, DOWN)
    );
    assert_eq!(
        (down.position.x, down.position.y, down.pressure),
        (1.0, 10.0, Some(0.5))
    );
    let moved = pointer(contact_events(&mut window, contact(7, Pen, Move, 2.0)));
    assert_eq!(
        (moved.id, moved.kind, moved.phase, moved.position.x),
        (PointerId(2), Touch, Move, 2.0),
        "the kind is the one of the contact's Down"
    );
    // A pen meanwhile is another pointer.
    let pen = pointer(contact_events(&mut window, contact(8, Pen, DOWN, 3.0)));
    assert_eq!((pen.id, pen.kind), (PointerId(3), Pen));
    let up = pointer(contact_events(&mut window, contact(7, Touch, UP, 2.0)));
    assert_eq!((up.id, up.phase), (PointerId(2), UP));
    let canceled = pointer(contact_events(&mut window, contact(8, Pen, Cancel, 3.0)));
    assert_eq!(
        (canceled.id, canceled.kind, canceled.phase),
        (PointerId(3), Pen, Cancel)
    );
    // The platform reuses id 7: a new contact, a new pointer.
    let again = pointer(contact_events(&mut window, contact(7, Touch, DOWN, 4.0)));
    assert_eq!(again.id, PointerId(4));
    // A contact is never the mouse.
    let mouse = pointer(contact_events(&mut window, contact(9, Mouse, DOWN, 5.0)));
    assert_eq!((mouse.id, mouse.kind), (PointerId(5), Touch));
    // Nothing is reported for a contact that never began (a hovering pen).
    window.script().contact(contact(40, Pen, Move, 1.0));
    assert_eq!(window.events(), []);
    window.script().contact(contact(41, Touch, UP, 1.0));
    assert_eq!(window.events(), []);
}

#[test]
fn keys_and_the_ime_pass_through_in_order() {
    let mut window = open(100.0, 100.0);
    let events = [
        ShellEvent::Key(kivact_types::KeyInput {
            key: Key::Char('ф'),
            code: Default::default(),
            phase: kivact_types::KeyPhase::Down {
                repeat: false,
                text: Some("ф".into()),
            },
            modifiers: kivact_types::Modifiers::default(),
            pointer: None,
        }),
        ShellEvent::Key(kivact_types::KeyInput {
            key: Key::Enter,
            code: Default::default(),
            phase: kivact_types::KeyPhase::Down {
                repeat: true,
                text: None,
            },
            modifiers: kivact_types::Modifiers::default(),
            pointer: None,
        }),
        ShellEvent::Ime(kivact_types::ImeInput::Preedit {
            text: "にほ".into(),
            selection: (Some((6, 6))).map(|(start, end)| start..end),
        }),
        ShellEvent::Ime(kivact_types::ImeInput::Preedit {
            text: "日本".into(),
            selection: (None).map(|(start, end)| start..end),
        }),
        ShellEvent::Ime(kivact_types::ImeInput::Commit("日本".into())),
        ShellEvent::Ime(kivact_types::ImeInput::Selection(0..2)),
        ShellEvent::Ime(kivact_types::ImeInput::End),
    ];
    for event in events.iter().cloned() {
        window.script().input(event);
    }
    let delivered = window.events();
    assert_eq!(delivered, events);
}

fn contact_events(window: &mut Harness, contact: PointerSample) -> Vec<ShellEvent> {
    window.script().contact(contact);
    window.events()
}
