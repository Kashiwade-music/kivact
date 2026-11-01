//! Role: The two-endpoint range on the range scalar's track, thumbs and body.

use super::clean;
use kivact::draw::Point;
use kivact::prelude::*;
use kivact_ui_creative as ui;

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
