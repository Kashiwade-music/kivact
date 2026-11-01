//! Role: The two-endpoint range on the range scalar's track, thumbs and body.

use super::*;
use kivact::ui::EditPhase;

#[test]
fn two_endpoint_range_preserves_order_finite_bounds_and_keyboard_target() {
    let (screen, endpoints) = kivact::test::render_with_handle(crate::options(), || {
        let endpoints = signal((24.0, 180.0));
        (
            endpoints,
            ui::range("Frame range", endpoints)
                .limits(0.0, 240.0)
                .test_id("range"),
        )
    });
    let start = screen.get_by_role(Role::Slider, "Frame range start");
    let end = screen.get_by_role(Role::Slider, "Frame range end");
    start.a11y_with_value(A11yAction::SetValue, "200");
    assert_eq!(endpoints.get_untracked(), (180.0, 180.0));
    end.a11y_with_value(A11yAction::SetValue, "300");
    assert_eq!(endpoints.get_untracked(), (180.0, 240.0));
    start.a11y_with_value(A11yAction::SetValue, "NaN");
    assert_eq!(endpoints.get_untracked(), (180.0, 240.0));
    start.a11y(A11yAction::Select);
    screen.get_by_test_id("range").press_key(Key::ArrowLeft);
    assert_eq!(endpoints.get_untracked(), (179.0, 240.0));
    clean(&screen);
}

#[test]
fn a_range_track_press_moves_the_nearer_end_which_the_keys_then_edit() {
    let (screen, span) = kivact::test::render_with_handle(crate::options(), || {
        let span = signal((20.0, 60.0));
        (
            span,
            ui::range("Selection", span)
                .limits(0.0, 100.0)
                .test_id("range"),
        )
    });
    let range = screen.get_by_test_id("range");
    let owner = range.rect().expect("range");
    // The value at a point: the track less one end's 20 px region spans 0..100.
    let at = |x: f32| f64::from((x - 10.0) / (owner.width - 20.0)) * 100.0;
    range.focus();
    let x = owner.width * 0.9;
    range.drag(
        Point::new(x, owner.height / 2.0),
        Point::new(x, owner.height / 2.0),
    );
    let (low, high) = span.get_untracked();
    assert_eq!(low, 20.0, "the start stays");
    assert!(
        (high - at(x)).abs() < 1e-3,
        "the nearer end follows: {high}"
    );
    screen.press_key(Key::ArrowLeft);
    assert!(
        (span.get_untracked().1 - (at(x) - 1.0)).abs() < 1e-3,
        "the keys edit the end the press moved"
    );
    // A press on the start's thumb keeps the distance to its center.
    let start = screen
        .get_by_role(Role::Slider, "Selection start")
        .rect()
        .expect("start");
    let grab = Point::new(start.center().x - owner.x + 3.0, owner.height / 2.0);
    range.drag(grab, Point::new(grab.x + owner.width * 0.2, grab.y));
    let moved = f64::from(owner.width * 0.2 / (owner.width - 20.0)) * 100.0;
    assert!((span.get_untracked().0 - 20.0 - moved).abs() < 1e-3);
    screen.press_key(Key::ArrowRight);
    assert!((span.get_untracked().0 - 21.0 - moved).abs() < 1e-3);
    clean(&screen);
}

#[test]
fn a_range_drag_is_one_edit_that_escape_writes_back() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, span) = kivact::test::render_with_handle(crate::options(), move || {
        let span = signal((20.0, 60.0));
        (
            span,
            ui::range("Selection", span)
                .limits(0.0, 100.0)
                .interval(rgb(0x8b7cff))
                .movable(true)
                .on_edit(move |phase| seen.borrow_mut().push(phase))
                .test_id("interval"),
        )
    });
    let owner = screen.get_by_test_id("interval").rect().expect("owner");
    let body = screen
        .get_by_role(Role::Button, "Selection interval")
        .rect()
        .expect("interval")
        .center();
    screen.get_by_test_id("interval").focus();
    screen.mouse_move(body.x, body.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(body.x + owner.width * 0.1, body.y);
    // The travel along the track less one end's 20 px region spans 0..100.
    let moved = f64::from(owner.width * 0.1 / (owner.width - 20.0)) * 100.0;
    let (low, high) = span.get_untracked();
    assert!((low - 20.0 - moved).abs() < 1e-6 && (high - 60.0 - moved).abs() < 1e-6);
    screen.press_key(Key::Escape);
    assert_eq!(
        span.get_untracked(),
        (20.0, 60.0),
        "both ends from before the press"
    );
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [EditPhase::Begin, EditPhase::Cancel]);
    log.borrow_mut().clear();
    screen.get_by_test_id("interval").press_key(Key::ArrowRight);
    assert_eq!(
        span.get_untracked(),
        (21.0, 61.0),
        "the body moves by one step"
    );
    assert_eq!(*log.borrow(), [EditPhase::Begin, EditPhase::Commit]);
    clean(&screen);
}
