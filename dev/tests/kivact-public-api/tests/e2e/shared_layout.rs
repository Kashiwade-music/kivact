//! Role: Shared layout: layout ids, layout motion in window space and corners.

use std::time::Duration;

use crate::support::mount;
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact::test::Screenshot;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Layout motion of 200 ms, linear, so the box at a time is known.
fn motion() -> Transition {
    Transition::tween(200, Easing::Linear)
}

/// A pixel is red when it is close to 0xff0000 (the moving box's fill).
fn is_red(pixel: u32) -> bool {
    let (r, g, b) = ((pixel >> 16) & 255, (pixel >> 8) & 255, pixel & 255);
    r > 200 && g < 60 && b < 60
}

#[test]
fn layout_id_starts_from_the_departed_box() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        let content = column((
            node().style(style().size(10, 30)),
            show(open, || {
                node()
                    .test_id("detail")
                    .layout_id("hero")
                    .style(style().size(200, 100).layout_transition(motion()))
            })
            .fallback(|| {
                node()
                    .test_id("card")
                    .layout_id("hero")
                    .style(style().size(60, 40).layout_transition(motion()))
            }),
        ));
        (open, content)
    });
    let card = screen.get_by_test_id("card").rect().unwrap();
    open.set(true);
    screen.run_until_idle();
    // Right after the switch the detail is where the card was.
    let start = screen.get_by_test_id("detail").rect().unwrap();
    assert_eq!(
        (start.x, start.y, start.width, start.height),
        (card.x, card.y, card.width, card.height),
        "the detail starts from the departed card's box"
    );
    screen.advance(ms(400));
    let end = screen.get_by_test_id("detail").rect().unwrap();
    assert_eq!(
        (end.width, end.height),
        (200.0, 100.0),
        "ends at its own box"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The moving box (window x 0, width 200) goes to (x 100, width 100). At 0 ms
/// the first frame still shows the box it came from; its edges are pixels.
/// A second row has a 30 px spacer before it (non-zero parent origin).
/// Tier 1 (geometry): the edges are exact pixels, not anti-aliased.
#[test]
fn layout_motion_maps_boxes_exactly() {
    let (screen, (a, b)) = mount(|| {
        let a = signal(true);
        let b = signal(true);
        let boxed = |id: &'static str, big: Signal<bool>| {
            node().test_id(id).style(move || {
                let big = big.get();
                style()
                    .margin_left(if big { 0 } else { 100 })
                    .size(if big { 200 } else { 100 }, if big { 40 } else { 20 })
                    .background(rgb(0xff0000))
                    .layout_transition(motion())
            })
        };
        let content = column((
            boxed("origin-zero", a),
            row((node().style(style().size(30, 20)), boxed("origin-30", b))),
        ));
        ((a, b), content)
    });
    a.set(false);
    b.set(false);
    screen.run_until_idle();
    let check_edges = |id: &str, origin: f64| {
        let rect = screen.get_by_test_id(id).rect().unwrap();
        assert_eq!(rect.x, origin, "{id}: the first frame is at the old box");
        let y = (rect.y + 2.0) as u32;
        let shot = screen.screenshot();
        let x0 = origin as u32;
        assert!(is_red(shot.pixel(x0, y).unwrap()), "{id}: left edge pixel");
        assert!(
            !is_red(shot.pixel(x0.wrapping_sub(1), y).unwrap_or(0)),
            "{id}: no pixel left of the edge"
        );
        assert!(
            is_red(shot.pixel(x0 + 199, y).unwrap()),
            "{id}: right edge in"
        );
        assert!(
            !is_red(shot.pixel(x0 + 200, y).unwrap()),
            "{id}: right edge out"
        );
    };
    check_edges("origin-zero", 0.0);
    check_edges("origin-30", 30.0);
    screen.advance(ms(400));
    let end = screen.get_by_test_id("origin-zero").rect().unwrap();
    assert_eq!((end.x, end.width, end.height), (100.0, 100.0, 20.0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A child with its own layout transition keeps its size while the parent
/// scales: its window rect is not distorted by the parent's scale.
#[test]
fn nested_layout_motion_keeps_children_steady() {
    let (screen, big) = mount(|| {
        let big = signal(false);
        let parent = node().test_id("parent").style(move || {
            style()
                .size(if big.get() { 200 } else { 100 }, 60)
                .layout_transition(motion())
        });
        let child = node()
            .test_id("child")
            .style(|| style().size(20, 20).layout_transition(motion()));
        (big, parent.child(child))
    });
    big.set(true);
    screen.run_until_idle();
    screen.advance(ms(100));
    let child = screen.get_by_test_id("child").rect().unwrap();
    assert_eq!(
        (child.width, child.height),
        (20.0, 20.0),
        "the child is not scaled"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// root, parent and child all have layout transitions and all move 10 px to
/// the right. At 0 ms the child is where it was before: the correction is
/// applied once. The whole tree sits 30 px into the window (non-zero origin).
#[test]
fn three_level_layout_motion_moves_once() {
    let (screen, shifted) = mount(|| {
        let shifted = signal(false);
        let child = node()
            .test_id("child")
            .style(move || style().size(20, 20).layout_transition(motion()));
        let parent = node()
            .test_id("parent")
            .style(move || style().size(60, 40).layout_transition(motion()))
            .child(child);
        let root = node()
            .test_id("root")
            .style(move || {
                style()
                    .margin_left(if shifted.get() { 40 } else { 30 })
                    .size(80, 60)
                    .layout_transition(motion())
            })
            .child(parent);
        (shifted, root)
    });
    let before = screen.get_by_test_id("child").rect().unwrap().x;
    shifted.set(true);
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_test_id("child").rect().unwrap().x,
        before,
        "at 0 ms the child is still at its old position"
    );
    screen.advance(ms(300));
    assert_eq!(
        screen.get_by_test_id("child").rect().unwrap().x,
        before + 10.0,
        "the child moved 10 px once"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A node with an exit and a layout id is removed; after its exit, a new node
/// with the same id mounted somewhere else starts at its own box, not at the
/// box of the removed one (the record is taken once, at begin_exit).
#[test]
fn layout_id_exit_records_once() {
    let (screen, (open, gap)) = mount(|| {
        let open = signal(true);
        let gap = signal(30.0f64);
        let content = column((
            node().style(move || style().size(10, gap.get())),
            show(open, || {
                node().test_id("tagged").layout_id("tag").style(
                    style()
                        .size(40, 20)
                        .exit(style().opacity(0.0))
                        .transition(TransitionProp::All, Transition::tween(200, Easing::Linear))
                        .layout_transition(motion()),
                )
            }),
        ));
        ((open, gap), content)
    });
    open.set(false);
    screen.run_until_idle();
    screen.advance(ms(300));
    assert!(
        screen.query_by_test_id("tagged").is_none(),
        "the exit ended"
    );
    gap.set(90.0);
    screen.run_until_idle();
    open.set(true);
    screen.run_until_idle();
    let fresh = screen.get_by_test_id("tagged").rect().unwrap();
    assert_eq!(fresh.y, 90.0, "the new node is at its own box at once");
    screen.advance(ms(300));
    assert_eq!(screen.get_by_test_id("tagged").rect().unwrap().y, 90.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Corner shape of a box while it scales (tier 1, geometry: ±1 device px, B-25).
/// The 50% coverage crossings of the corner's rows and columns match a static
/// box of the same size and radius. Without the radius correction the arc of
/// radius 20 is stretched by 0.708 in x and moves by 1.7 to 3 px in the top rows.
#[test]
fn radius_is_corrected_while_scaling() {
    const RADIUS: f64 = 20.0;
    let (screen, big) = mount(|| {
        let big = signal(false);
        let moving = node().test_id("moving").style(move || {
            style()
                .size(if big.get() { 200 } else { 100 }, 100)
                .radius(RADIUS)
                .background(rgb(0xff0000))
                .layout_transition(motion())
        });
        (big, moving)
    });
    big.set(true);
    screen.run_until_idle();
    screen.advance(ms(100));
    let rect = screen.get_by_test_id("moving").rect().unwrap();
    let corner = screen
        .screenshot()
        .crop(rect.x as u32, rect.y as u32, CORNER, CORNER);
    let reference = reference_corner(rect.width, RADIUS);
    let moving = corner_crossings(&corner);
    let fixed = corner_crossings(&reference);
    for (index, (m, r)) in moving.iter().zip(&fixed).enumerate() {
        let (Some(m), Some(r)) = (m, r) else {
            panic!("line {index}: no 50% crossing (moving {m:?}, reference {r:?})");
        };
        assert!(
            (m - r).abs() <= 1.0,
            "line {index}: the edge is at {m:.2} px, the reference at {r:.2} px"
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Corner shot size and the rows and columns compared (the arc spans 20 px).
const CORNER: u32 = 20;
const LINES: u32 = 12;

/// Coverage of the red box in a pixel: 0 on the white background, 1 inside.
fn coverage(pixel: u32) -> f64 {
    f64::from(255 - ((pixel >> 8) & 255)) / 255.0
}

/// Where `samples` (pixel centres at i + 0.5) first cross 50% going inward,
/// interpolated between the two pixels around the crossing.
fn crossing(samples: &[f64]) -> Option<f64> {
    samples.windows(2).enumerate().find_map(|(i, w)| {
        (w[0] < 0.5 && w[1] >= 0.5).then(|| i as f64 + 0.5 + (0.5 - w[0]) / (w[1] - w[0]))
    })
}

/// The crossings of the first `LINES` rows, then of the first `LINES` columns.
fn corner_crossings(shot: &Screenshot) -> Vec<Option<f64>> {
    let at = |x: u32, y: u32| coverage(shot.pixel(x, y).expect("the pixel is inside the shot"));
    let rows = (0..LINES).map(|y| crossing(&(0..CORNER).map(|x| at(x, y)).collect::<Vec<_>>()));
    let columns = (0..LINES).map(|x| crossing(&(0..CORNER).map(|y| at(x, y)).collect::<Vec<_>>()));
    rows.chain(columns).collect()
}

/// A static box of `width` x 100 with the radius, its top-left corner.
fn reference_corner(width: f64, radius: f64) -> Screenshot {
    let (screen, ()) = mount(move || {
        (
            (),
            node().style(
                style()
                    .size(width, 100)
                    .radius(radius)
                    .background(rgb(0xff0000)),
            ),
        )
    });
    screen.screenshot().crop(0, 0, CORNER, CORNER)
}
