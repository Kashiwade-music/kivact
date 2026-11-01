//! Role: Verify shared zoom and pan through real-window input and owner cleanup.
use crate::support::mount;
use kivact::draw::{Point, Size};
use kivact::interaction::{ViewTransform, use_zoom_pan};
use kivact::prelude::*;
use kivact::ui::EditPhase;
use kivact::view::{KeyCode, Modifiers, MouseButton, PointerKind};
use std::{cell::RefCell, rc::Rc};
type Log = Rc<RefCell<Vec<(EditPhase, ViewTransform)>>>;
fn close(a: Point, b: Point) {
    assert!(
        (a.x - b.x).abs() < 0.001 && (a.y - b.y).abs() < 0.001,
        "{a:?} != {b:?}"
    );
}
fn surface() -> Node {
    node()
        .role(Role::Group)
        .name("Surface")
        .test_id("surface")
        .style(style().size(200, 100))
}

#[test]
fn idle_space_reaches_the_application_but_active_pan_consumes_it() {
    use kivact::view::Handled;
    let (screen, (transform, keys)) = mount(|| {
        let transform = signal(ViewTransform::default());
        let keys = signal(Vec::<bool>::new());
        (
            (transform, keys),
            column((
                surface().part(use_zoom_pan(transform).viewport()),
                button("Other"),
            ))
            .on_key(move |event| {
                if event.key == Key::Space {
                    keys.update(|keys| keys.push(true));
                }
                Handled::No
            })
            .on_key_up(move |event| {
                if event.key == Key::Space {
                    keys.update(|keys| keys.push(false));
                }
                Handled::No
            }),
        )
    });
    let target = screen.get_by_test_id("surface");
    target.focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert_eq!(keys.get_untracked(), [true, false]);
    screen.mouse_move(20., 20.);
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    assert_eq!(keys.get_untracked(), [true, false, true]);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(50., 40.);
    close(transform.get_untracked().offset, Point::new(30., 20.));
    // The public driver injects Down without a repeat flag; repeated Down
    // follows the same Space branch, which never reads that flag.
    for _ in 0..2 {
        screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    }
    assert_eq!(keys.get_untracked(), [true, false, true]);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert_eq!(keys.get_untracked(), [true, false, true]);
    screen.mouse_move(70., 50.);
    screen.mouse_up(MouseButton::Primary);
    close(transform.get_untracked().offset, Point::new(50., 30.));
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    screen.get_by_role(Role::Button, "Other").focus();
    target.focus();
    let before = transform.get_untracked();
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(90., 60.);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        transform.get_untracked(),
        before,
        "focus loss clears held Space"
    );
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert_eq!(keys.get_untracked(), [true, false, true, true, false]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn wheel_keeps_the_point_under_the_pointer_and_reports_a_whole_edit() {
    let log = Log::default();
    let seen = log.clone();
    let initial = ViewTransform {
        offset: Point::new(10., 20.),
        scale: 2.,
    };
    let (screen, transform) = mount(move || {
        let transform = signal(initial);
        (
            transform,
            surface().part(
                use_zoom_pan(transform)
                    .on_edit(move |phase| {
                        seen.borrow_mut().push((phase, transform.get_untracked()));
                    })
                    .viewport(),
            ),
        )
    });
    let point = Point::new(60., 40.);
    let content = initial.to_content(point);
    screen.mouse_move(point.x, point.y);
    screen.scroll(0., -200.);
    let after = transform.get_untracked();
    assert!((after.scale - 2. * 0.2_f64.exp()).abs() < 0.00001);
    close(after.to_content(point), content);
    assert_eq!(
        *log.borrow(),
        [(EditPhase::Begin, initial), (EditPhase::Commit, after)]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn middle_pan_commits_and_escape_restores_the_press_snapshot() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, transform) = mount(move || {
        let transform = signal(ViewTransform::default());
        (
            transform,
            surface().part(
                use_zoom_pan(transform)
                    .on_edit(move |phase| {
                        seen.borrow_mut().push((phase, transform.get_untracked()));
                    })
                    .viewport(),
            ),
        )
    });
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_move(50., 40.);
    close(transform.get_untracked().offset, Point::new(30., 20.));
    screen.scroll(0., -200.);
    screen.press_key(Key::Char('+'));
    assert_eq!(transform.get_untracked().scale, 1.);
    screen.mouse_move(70., 50.);
    close(transform.get_untracked().offset, Point::new(50., 30.));
    screen.mouse_up(MouseButton::Middle);
    assert_eq!(
        log.borrow().iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        [EditPhase::Begin, EditPhase::Commit]
    );
    let initial = transform.get_untracked();
    log.borrow_mut().clear();
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_move(80., 50.);
    screen.press_key(Key::Escape);
    assert_eq!(transform.get_untracked(), initial);
    screen.mouse_move(90., 60.);
    screen.mouse_up(MouseButton::Middle);
    assert_eq!(
        *log.borrow(),
        [(EditPhase::Begin, initial), (EditPhase::Cancel, initial)]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn space_primary_pans_and_pointer_cancel_restores_while_plain_primary_is_free() {
    let (screen, transform) = mount(|| {
        let transform = signal(ViewTransform::default());
        (
            transform,
            surface().part(use_zoom_pan(transform).viewport()),
        )
    });
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(40., 40.);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(transform.get_untracked(), ViewTransform::default());
    let target = screen.get_by_test_id("surface");
    target.focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(60., 50.);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    screen.mouse_move(70., 60.);
    screen.mouse_up(MouseButton::Primary);
    close(transform.get_untracked().offset, Point::new(30., 20.));
    let initial = transform.get_untracked();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    let contact = screen.contact_down(PointerKind::Touch, 20., 20.);
    screen.contact_move(contact, 50., 50.);
    assert_ne!(transform.get_untracked(), initial);
    screen.contact_cancel(contact);
    assert_eq!(transform.get_untracked(), initial);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(40., 40.);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(transform.get_untracked(), initial);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn keyboard_zoom_clamps_and_fit_and_coordinate_maps_share_one_transform() {
    let (screen, transform) = mount(|| {
        let transform = signal(ViewTransform::default());
        (
            transform,
            surface().part(
                use_zoom_pan(transform)
                    .scale_range(2., 0.5)
                    .content_size(Size::new(100., 100.))
                    .viewport(),
            ),
        )
    });
    let target = screen.get_by_test_id("surface");
    target.focus();
    let center = Point::new(100., 50.);
    for _ in 0..10 {
        screen.press_key(Key::Char('+'));
    }
    assert_eq!(transform.get_untracked().scale, 2.);
    close(transform.get_untracked().to_content(center), center);
    for _ in 0..20 {
        screen.press_key(Key::Char('-'));
    }
    assert_eq!(transform.get_untracked().scale, 0.5);
    screen.press_key(Key::Char('0'));
    assert_eq!(
        transform.get_untracked(),
        ViewTransform::fit(Size::new(100., 100.), Size::new(200., 100.))
    );
    let current = transform.get_untracked();
    close(
        current.to_content(current.to_view(Point::new(25., 75.))),
        Point::new(25., 75.),
    );
    for scale in [0., -1., f64::NAN, f64::INFINITY] {
        let invalid = ViewTransform {
            scale,
            ..ViewTransform::default()
        };
        close(invalid.to_view(center), center);
        close(invalid.to_content(center), center);
    }
    assert_eq!(
        ViewTransform::fit(Size::new(0., 10.), Size::new(100., 100.)),
        ViewTransform::default()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn disabling_and_owner_disposal_cancel_an_active_pan() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, (transform, shown, disabled)) = mount(move || {
        let transform = signal(ViewTransform::default());
        let shown = signal(true);
        let disabled = signal(false);
        (
            (transform, shown, disabled),
            show(shown, move || {
                let seen = seen.clone();
                surface().disabled(disabled).part(
                    use_zoom_pan(transform)
                        .on_edit(move |phase| {
                            seen.borrow_mut().push((phase, transform.get_untracked()));
                        })
                        .viewport(),
                )
            }),
        )
    });
    for dispose in [false, true] {
        screen.mouse_move(20., 20.);
        screen.mouse_down(MouseButton::Middle);
        screen.mouse_move(50., 50.);
        assert_ne!(transform.get_untracked(), ViewTransform::default());
        if dispose {
            shown.set(false);
        } else {
            disabled.set(true);
        }
        screen.run_until_idle();
        assert_eq!(transform.get_untracked(), ViewTransform::default());
        screen.mouse_up(MouseButton::Middle);
        if !dispose {
            disabled.set(false);
            screen.run_until_idle();
        }
    }
    assert_eq!(
        log.borrow().iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        [
            EditPhase::Begin,
            EditPhase::Cancel,
            EditPhase::Begin,
            EditPhase::Cancel
        ]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn focus_loss_clears_space_and_invalid_ranges_use_defaults() {
    let (screen, transform) = mount(|| {
        let transform = signal(ViewTransform::default());
        (
            transform,
            column((
                surface().part(
                    use_zoom_pan(transform)
                        .scale_range(f64::NAN, -1.)
                        .viewport(),
                ),
                button("Other"),
            )),
        )
    });
    screen.get_by_test_id("surface").focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    screen.get_by_role(Role::Button, "Other").focus();
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(40., 40.);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(transform.get_untracked(), ViewTransform::default());
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    transform.set(ViewTransform {
        scale: 200.,
        ..ViewTransform::default()
    });
    screen.run_until_idle();
    screen.get_by_test_id("surface").press_key(Key::Char('+'));
    assert_eq!(transform.get_untracked().scale, 100.);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn focus_departure_cancels_pan_before_later_pointer_updates() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, transform) = mount(move || {
        let transform = signal(ViewTransform::default());
        (
            transform,
            column((
                surface().part(
                    use_zoom_pan(transform)
                        .on_edit(move |phase| seen.borrow_mut().push((phase, transform.get())))
                        .viewport(),
                ),
                button("Other"),
            )),
        )
    });
    screen.get_by_test_id("surface").focus();
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_move(50., 40.);
    assert_ne!(transform.get_untracked(), ViewTransform::default());
    screen.get_by_role(Role::Button, "Other").focus();
    assert_eq!(transform.get_untracked(), ViewTransform::default());
    screen.mouse_move(80., 60.);
    screen.mouse_up(MouseButton::Middle);
    assert_eq!(transform.get_untracked(), ViewTransform::default());
    assert_eq!(
        log.borrow()
            .iter()
            .map(|(phase, _)| *phase)
            .collect::<Vec<_>>(),
        [EditPhase::Begin, EditPhase::Cancel]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn pan_release_applies_position_when_the_caller_consumed_motion() {
    use kivact::view::{Handled, PointerPhase};
    let (screen, transform) = mount(|| {
        let transform = signal(ViewTransform::default());
        (
            transform,
            surface()
                .on_pointer(|event| {
                    if event.phase == PointerPhase::Move {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
                .part(use_zoom_pan(transform).viewport()),
        )
    });
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_move(70., 50.);
    assert_eq!(transform.get_untracked(), ViewTransform::default());
    screen.mouse_up(MouseButton::Middle);
    close(transform.get_untracked().offset, Point::new(50., 30.));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
