//! Role: Tests for CSS calc() and advanced layout length units.
//!
//! Layout lengths beyond px and percentages: calc lengths (`pct(100) -
//! 32`) in sizes, insets, margins, padding and gaps, checked on the laid
//! out boxes of real windows (and their resizes).

use kivact::draw::Rect;
use kivact::platform;
use kivact::prelude::*;
use kivact::style::Position;
use kivact::style::pct;
use kivact::test::{RenderOptions, Screen, render_with};

/// The window-space box of the element with test id `id`.
#[track_caller]
pub(crate) fn rect_of(screen: &Screen, id: &str) -> Rect {
    screen
        .get_by_test_id(id)
        .rect()
        .unwrap_or_else(|| panic!("{id} is laid out"))
}

/// `a` is `(x, y, width, height)` within half a pixel.
#[track_caller]
pub(crate) fn assert_rect(a: Rect, (x, y, w, h): (f32, f32, f32, f32), what: &str) {
    let close = |p: f32, q: f32| (p - q).abs() <= 0.5;
    assert!(
        close(a.x, x) && close(a.y, y) && close(a.width, w) && close(a.height, h),
        "{what}: {a:?} is not ({x}, {y}, {w}, {h})"
    );
}

/// An 800 × 600 window (or `w × h`).
pub(crate) fn window(w: f32, h: f32) -> RenderOptions {
    RenderOptions {
        width: w,
        height: h,
        ..RenderOptions::from_env()
    }
}

#[test]
fn calc_lengths_size_place_and_space_boxes_against_their_containing_block() {
    let screen = render_with(window(800.0, 600.0), || {
        column((
            // calc(100% - 32px) wide, calc(25% - 10px) high, margin
            // calc(5% + 1px) on the left.
            node().test_id("bar").style(
                style()
                    .width(pct(100) - 32)
                    .height(pct(25) - 10)
                    .margin_x(pct(5) + 1),
            ),
            // Placed by calc insets inside a 100 px high box; its padding
            // is calc too.
            node()
                .test_id("holder")
                .child(
                    node().test_id("pinned").style(
                        style()
                            .position(Position::Absolute)
                            .left(pct(50) - 20)
                            .bottom(pct(10) + 2)
                            .size(40, 40),
                    ),
                )
                .style(style().height(100).padding_x(pct(10) - 4)),
            // A row gap of calc(10% - 70px) (10% of the row's 780 px).
            row((
                node().test_id("a").style(style().size(20, 20)),
                node().test_id("b").style(style().size(20, 20)),
            ))
            .style(style().gap(pct(10) - 70)),
        ))
        .style(style().height(pct(100)).padding(10))
    });
    // The column's content box is 780 × 580 at (10, 10).
    assert_rect(
        rect_of(&screen, "bar"),
        (10.0 + 39.0 + 1.0, 10.0, 780.0 - 32.0, 145.0 - 10.0),
        "the calc bar",
    );
    // Absolute: against the holder's padding box (780 wide, 100 high).
    let holder = rect_of(&screen, "holder");
    assert_rect(
        rect_of(&screen, "pinned"),
        (
            holder.x + 390.0 - 20.0,
            holder.y + 100.0 - 12.0 - 40.0,
            40.0,
            40.0,
        ),
        "the calc-inset box",
    );
    let (a, b) = (rect_of(&screen, "a"), rect_of(&screen, "b"));
    assert!(
        (b.x - (a.x + 20.0 + 8.0)).abs() <= 0.5,
        "an 8 px calc gap: {a:?} {b:?}"
    );

    // The percentages follow the window.
    screen.resize(600.0, 400.0);
    assert_rect(
        rect_of(&screen, "bar"),
        (10.0 + 29.0 + 1.0, 10.0, 580.0 - 32.0, 95.0 - 10.0),
        "the calc bar after a resize",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn aspect_ratio_sizes_a_box_from_its_other_side() {
    let screen = render_with(window(800.0, 600.0), || {
        column((
            // width: 320px; aspect-ratio: 16 / 9
            node()
                .test_id("video")
                .style(style().width(320).aspect_ratio(16.0 / 9.0)),
            // Stretched across the column: 800 wide, so 200 high.
            node().test_id("banner").style(style().aspect_ratio(4)),
            // In a row, 50 high: 50 wide.
            row(node()
                .test_id("square")
                .style(style().height(50).aspect_ratio(1))),
        ))
    });
    assert_rect(
        rect_of(&screen, "video"),
        (0.0, 0.0, 320.0, 180.0),
        "16 / 9",
    );
    assert_rect(
        rect_of(&screen, "banner"),
        (0.0, 180.0, 800.0, 200.0),
        "4 / 1",
    );
    assert_rect(
        rect_of(&screen, "square"),
        (0.0, 380.0, 50.0, 50.0),
        "1 / 1",
    );
    screen.resize(400.0, 600.0);
    assert_rect(
        rect_of(&screen, "banner"),
        (0.0, 180.0, 400.0, 100.0),
        "4 / 1 after a resize",
    );
}

#[test]
fn window_size_switches_a_layout_at_a_breakpoint_and_stands_in_for_viewport_units() {
    use kivact::style::Direction;
    let screen = render_with(window(1000.0, 600.0), || {
        // @media (max-width: 860px) { .shell { flex-direction: column } }
        let narrow = memo(move || platform::window_size().width < 860.0);
        column((
            row((
                node().test_id("side").style(style().size(200, 50)),
                node().test_id("main").style(style().size(300, 50)),
            ))
            .style(move || {
                if narrow.get() {
                    style().direction(Direction::Column)
                } else {
                    style()
                }
            }),
            // width: min(300px, 34vw)
            node().test_id("vw").style(move || {
                style()
                    .height(10)
                    .width(platform::window_size().width * 0.34)
                    .max_width(300)
            }),
        ))
    });
    assert_rect(
        rect_of(&screen, "main"),
        (200.0, 0.0, 300.0, 50.0),
        "beside",
    );
    assert_rect(rect_of(&screen, "vw"), (0.0, 50.0, 300.0, 10.0), "300px");

    screen.resize(800.0, 600.0);
    assert_rect(rect_of(&screen, "main"), (0.0, 50.0, 300.0, 50.0), "below");
    assert_rect(rect_of(&screen, "vw"), (0.0, 100.0, 272.0, 10.0), "34vw");

    // A scale change keeps the logical size.
    screen.set_scale(2.0);
    assert_rect(
        rect_of(&screen, "vw"),
        (0.0, 100.0, 272.0, 10.0),
        "34vw at 2x",
    );
    screen.resize(1000.0, 600.0);
    assert_rect(
        rect_of(&screen, "main"),
        (200.0, 0.0, 300.0, 50.0),
        "beside again",
    );
}
