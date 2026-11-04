//! Role: Verify shared zoom and pan through real-window input and owner cleanup.
use crate::support::mount;
use kivact::geometry::{Point, Rect};
use kivact::interaction::{ViewTransform, use_zoom_pan};
use kivact::prelude::*;
use kivact::view::EditPhase;
use kivact::view::{KeyCode, Modifiers, MouseButton};
use std::cell::RefCell;
use std::rc::Rc;
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

fn other() -> Node {
    node()
        .role(Role::Button)
        .name("Other")
        .test_id("Other")
        .focusable(true)
        .style(style().size(50, 30))
}

#[test]
fn space_armed_follows_space_and_turns_off_when_the_focus_leaves() {
    let (screen, armed) = mount(|| {
        let transform = signal(ViewTransform::default());
        let zoom = use_zoom_pan(transform);
        (
            zoom.space_armed(),
            column((surface().part(zoom.viewport()), other())),
        )
    });
    assert!(!armed.get_untracked());
    screen.get_by_test_id("surface").focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(armed.get_untracked(), "Space is held on the surface");
    screen.get_by_test_id("Other").focus();
    assert!(!armed.get_untracked(), "the focus left without a release");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn zero_fits_content_that_does_not_start_at_the_origin() {
    let (screen, transform) = mount(|| {
        let transform = signal(ViewTransform::default());
        (
            transform,
            node()
                .role(Role::Group)
                .name("Surface")
                .test_id("surface")
                .style(style().size(400, 200))
                .part(
                    use_zoom_pan(transform)
                        .content(Rect::new(100.0, 50.0, 200.0, 100.0))
                        .viewport(),
                ),
        )
    });
    screen.get_by_test_id("surface").focus();
    screen.press_key(Key::Char('0'));
    let fit = transform.get_untracked();
    assert!((fit.scale - 2.0).abs() < 0.001, "{fit:?}");
    close(fit.to_view(Point::new(100.0, 50.0)), Point::new(0.0, 0.0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
