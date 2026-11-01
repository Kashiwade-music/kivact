//! Role: Tests for CSS calc() and advanced layout length units.
//!
//! Layout lengths beyond px and percentages: calc lengths (`pct(100) -
//! 32`) in sizes, insets, margins, padding and gaps, checked on the laid
//! out boxes of real windows (and their resizes).

use kivact::draw::Rect;
use kivact::platform;
use kivact::prelude::*;
use kivact::style::Position;
use kivact::style::{Align, pct};
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
fn independent_padding_and_margin_edges_compose_and_survive_resize() {
    let screen = render_with(window(400.0, 300.0), || {
        column((node().test_id("child").style(
            style()
                .grow(1)
                .margin_top(3)
                .margin_right(5)
                .margin_bottom(7)
                .margin_left(11),
        ),))
        .test_id("parent")
        .style(
            style()
                .width(pct(50))
                .height(160)
                .padding_top(11)
                .padding_right(17)
                .padding_bottom(23)
                .padding_left(29),
        )
    });
    assert_rect(
        rect_of(&screen, "child"),
        (40., 14., 138., 116.),
        "all eight edges",
    );
    screen.resize(600., 300.);
    assert_rect(
        rect_of(&screen, "child"),
        (40., 14., 238., 116.),
        "resized content box",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
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
fn content_sizes_fit_a_box_to_its_content_and_are_reported_elsewhere() {
    use kivact::style::Length;
    let screen = render_with(window(800.0, 600.0), || {
        column((
            node()
                .test_id("max")
                .child(text("Hello wide world").test_id("line"))
                .style(style().width(Length::MaxContent)),
            // In a row the width is the main size, and the height follows the
            // wrapped text (in a column taffy measures the height unwrapped).
            row(node()
                .test_id("min")
                .child(text("Hello wide world"))
                .style(style().width(Length::MinContent)))
            .style(style().align_items(Align::Start)),
            // min-width: max-content on a box filling a scroller: as wide as
            // the scroller, or as its content when that is wider (not shrunk
            // by the scroller's row).
            scroll(
                node()
                    .test_id("track")
                    .child(node().test_id("lane").style(style().size(1200, 10)))
                    .style(
                        style()
                            .width(Length::MaxContent)
                            .min_width(pct(100))
                            .shrink(0),
                    ),
            )
            .orientation(Orientation::Horizontal)
            .style(style().height(20)),
            scroll(
                node()
                    .test_id("short")
                    .child(node().style(style().size(300, 10)))
                    .style(
                        style()
                            .width(Length::MaxContent)
                            .min_width(pct(100))
                            .shrink(0),
                    ),
            )
            .orientation(Orientation::Horizontal)
            .style(style().height(20)),
            // Not a size: auto, and reported.
            node()
                .test_id("misplaced")
                .style(style().height(10).min_width(Length::MaxContent)),
        ))
    });
    let (max, min) = (rect_of(&screen, "max"), rect_of(&screen, "min"));
    let line = rect_of(&screen, "line");
    assert!(
        max.width < 800.0 && (max.width - line.width).abs() <= 1.0,
        "max-content is the line's width: {max:?} {line:?}"
    );
    assert!(
        min.width < max.width / 2.0 && min.height > max.height * 2.0,
        "min-content breaks every line: {min:?} vs {max:?}"
    );
    assert_eq!(rect_of(&screen, "track").width, 1200.0, "wider content");
    assert_eq!(
        rect_of(&screen, "short").width,
        800.0,
        "the scroller's width"
    );
    assert_eq!(
        rect_of(&screen, "misplaced").width,
        800.0,
        "auto (stretched)"
    );
    let errors = screen.errors();
    assert!(
        errors.len() == 1
            && *errors[0].kind() == kivact::diagnostics::ErrorKind::Unsupported
            && errors[0].to_string().contains("content size"),
        "one Unsupported report: {errors:?}"
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

/// A border takes room inside the box like padding (CSS `box-sizing:
/// border-box`): the content sits inside both, a size is the border box's,
/// and a width set from a signal moves the content. A change of the
/// border's color writes the paint alone; one of its width writes the
/// layout too.
#[test]
fn a_border_takes_room_inside_the_box_like_padding() {
    let (screen, (width, color)) = crate::support::mount(|| {
        let width = signal(2.0f32);
        let color = signal(0xff0000u32);
        let view = column((
            node()
                .test_id("auto")
                .style(move || {
                    style()
                        .border(width.get(), kivact::style::rgb(color.get()))
                        .padding(8)
                        .align_items(Align::Start)
                })
                .child(node().test_id("child").style(style().size(20, 10))),
            node()
                .test_id("sized")
                .style(
                    style()
                        .size(100, 50)
                        .border(2, kivact::style::rgb(0x0000ff))
                        .padding(8),
                )
                .child(node().test_id("inner").style(style().grow(1))),
            node()
                .test_id("sides")
                .style(
                    style()
                        .border_left(3, kivact::style::rgb(0x0000ff))
                        .border_bottom(5, kivact::style::rgb(0x0000ff))
                        .align_items(Align::Start),
                )
                .child(node().test_id("side child").style(style().size(10, 10))),
        ))
        .style(style().align_items(Align::Start).gap(4));
        ((width, color), view)
    });
    let auto = rect_of(&screen, "auto");
    // 20 × 10 inside 8 px of padding and 2 px of border on each side.
    assert_rect(auto, (auto.x, auto.y, 40.0, 30.0), "auto");
    assert_rect(
        rect_of(&screen, "child"),
        (auto.x + 10.0, auto.y + 10.0, 20.0, 10.0),
        "child",
    );
    // The size is the border box's: the content box is 80 × 30.
    let sized = rect_of(&screen, "sized");
    assert_rect(sized, (sized.x, sized.y, 100.0, 50.0), "sized");
    assert_rect(
        rect_of(&screen, "inner"),
        (sized.x + 10.0, sized.y + 10.0, 80.0, 30.0),
        "inner",
    );
    // Each side takes its own width.
    let sides = rect_of(&screen, "sides");
    assert_rect(sides, (sides.x, sides.y, 13.0, 15.0), "sides");
    assert_rect(
        rect_of(&screen, "side child"),
        (sides.x + 3.0, sides.y, 10.0, 10.0),
        "side child",
    );

    // A new color writes one prop (the paint) and moves nothing.
    let before = screen.stats().set_props;
    color.set(0x00ff00);
    screen.run_until_idle();
    assert_eq!(screen.stats().set_props - before, 1, "the paint alone");
    assert_eq!(rect_of(&screen, "auto"), auto);

    // A new width writes the layout and the paint and moves the content.
    let before = screen.stats().set_props;
    width.set(4.0);
    screen.run_until_idle();
    assert_eq!(
        screen.stats().set_props - before,
        2,
        "the layout and the paint"
    );
    assert_rect(
        rect_of(&screen, "auto"),
        (auto.x, auto.y, 44.0, 34.0),
        "wider",
    );
    assert_rect(
        rect_of(&screen, "child"),
        (auto.x + 12.0, auto.y + 12.0, 20.0, 10.0),
        "moved child",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
