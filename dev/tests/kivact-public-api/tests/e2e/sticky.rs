//! Role: Tests for sticky and fixed positioning behavior.
//!
//! `Position::Sticky` and `Position::Fixed` on real windows: a sticky box
//! keeps its insets from its scroll container's edges while that scrolls
//! (both axes, within its parent), and a fixed box sits at its insets in
//! the window whatever scrolls or clips around it. Both are drawn, hit,
//! reported (`Element::rect`, `NodeRef::rect`) and exposed where they are
//! (G-07). Checked on the frame's pixels (scale 1), the boxes and real
//! input.

use std::cell::Cell;
use std::rc::Rc;

use kivact::control::Scroll;
use kivact::prelude::*;
use kivact::style::{Color, Style, pct};
use kivact::style::{Length, Position};
use kivact::test::{Screen, render_with};
use kivact::view::{Placement, overlay};

use crate::layout_units::{assert_rect, rect_of, window};
use crate::paint::{expect, pixel};
use crate::support::mount_with;

const HEAD: u32 = 0x2050c0;
const ROW: u32 = 0xe0e0e0;
const CORNER: u32 = 0xc02020;
const RULER: u32 = 0x20a040;
const TRACK: u32 = 0xa0a020;
const LANE: u32 = 0xf8f8f8;

/// A box of `w × h` filled with `color`.
fn block(color: u32, w: i32, h: i32) -> Node {
    node().style(style().size(w, h).background(rgb(color)))
}

/// Logs the presses of `name`.
fn pressable(name: &'static str, presses: &Rc<Cell<u32>>) -> Node {
    let presses = presses.clone();
    node()
        .role(Role::Button)
        .name(name)
        .on_press(move || presses.set(presses.get() + 1))
}

/// A 400 × 300 scroller at (100, 100) holding a table 1200 px wide (its
/// 400 px rows overflow with their content): a
/// sticky header row (top 0) over 40 rows of 30 px.
fn table(header_presses: &Rc<Cell<u32>>) -> Scroll {
    let header = pressable("header", header_presses)
        .test_id("head")
        .child(block(HEAD, 1200, 32))
        .style(style().position(Position::Sticky).top(0).z_index(2));
    let rows: Vec<Node> = (0..40)
        .map(|i| {
            node()
                .test_id(if i == 0 { "row0" } else { "row" })
                .child(block(ROW, 1200, 29))
                .style(style().height(30))
        })
        .collect();
    scroll(column((header, rows)))
        .both_axes()
        .test_id("scroller")
        .style(
            style()
                .position(Position::Absolute)
                .left(100)
                .top(100)
                .size(400, 300),
        )
}

#[test]
fn a_sticky_table_header_stays_on_top_while_rows_scroll_and_follows_sideways() {
    let presses = Rc::new(Cell::new(0));
    let p = presses.clone();
    let screen = render_with(window(800.0, 600.0), move || {
        node()
            .child(table(&p))
            .style(style().size(800, 600).background(Color::WHITE))
    });
    assert_rect(
        rect_of(&screen, "head"),
        (100.0, 100.0, 400.0, 32.0),
        "at rest",
    );
    assert_rect(
        rect_of(&screen, "row0"),
        (100.0, 132.0, 400.0, 30.0),
        "first row",
    );

    // Down 200: the rows move up, the header stays at the scroller's top.
    screen.get_by_test_id("scroller").scroll(0.0, 200.0);
    assert_rect(
        rect_of(&screen, "head"),
        (100.0, 100.0, 400.0, 32.0),
        "stuck",
    );
    assert_rect(
        rect_of(&screen, "row0"),
        (100.0, -68.0, 400.0, 30.0),
        "scrolled",
    );
    let shot = screen.screenshot();
    // Drawn over the rows that scrolled under it.
    expect(&shot, 300, 110, HEAD, 8, "the header over the rows");
    expect(&shot, 300, 150, ROW, 8, "a row below it");
    // Hit where it is drawn: a press on it reaches the header.
    screen.click_at(300.0, 110.0);
    assert_eq!(presses.get(), 1, "the stuck header is pressed");
    // Assistive technology sees it there too.
    let bounds = screen
        .get_by_role(Role::Button, "header")
        .accessibility_bounds()
        .expect("exposed");
    assert_rect(bounds, (100.0, 100.0, 400.0, 32.0), "accessibility bounds");

    // Sideways it moves with the table.
    screen.get_by_test_id("scroller").scroll(150.0, 0.0);
    assert_rect(
        rect_of(&screen, "head"),
        (-50.0, 100.0, 400.0, 32.0),
        "sideways",
    );
    let bounds = screen
        .get_by_role(Role::Button, "header")
        .accessibility_bounds()
        .expect("exposed");
    assert_rect(
        bounds,
        (-50.0, 100.0, 400.0, 32.0),
        "accessibility bounds sideways",
    );

    // Back at the top it is in flow again.
    screen.get_by_test_id("scroller").scroll(-150.0, -200.0);
    assert_rect(
        rect_of(&screen, "head"),
        (100.0, 100.0, 400.0, 32.0),
        "at rest again",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// An arrange timeline: a grid (`220px auto`, `min-width: max-content`) in
/// a 400 × 300 scroller at (0, 0): a corner sticky top and left, a ruler
/// sticky top, and track headers sticky left beside 1000 px lanes.
fn timeline() -> Scroll {
    let sticky = |s: Style| s.position(Position::Sticky);
    let mut cells: Vec<Node> = vec![
        node().test_id("corner").style(sticky(
            style()
                .top(0)
                .left(0)
                .z_index(6)
                .height(40)
                .background(rgb(CORNER)),
        )),
        node().test_id("ruler").style(sticky(
            style()
                .top(0)
                .z_index(5)
                .height(40)
                .width(1000)
                .background(rgb(RULER)),
        )),
    ];
    for i in 0..10 {
        cells.push(
            node()
                .test_id(if i == 0 { "track0" } else { "track" })
                .style(sticky(
                    style().left(0).z_index(4).height(60).background(rgb(TRACK)),
                )),
        );
        cells.push(node().style(style().height(60).width(1000).background(rgb(LANE))));
    }
    scroll(
        grid(cells).style(
            style()
                .grid_template_columns((220, kivact::style::auto()))
                .width(Length::MaxContent)
                .min_width(pct(100)),
        ),
    )
    .both_axes()
    .test_id("arrange")
    .style(style().size(400, 300))
}

#[test]
fn a_timeline_corner_sticks_on_both_axes_over_a_ruler_and_track_headers() {
    let screen = render_with(window(800.0, 600.0), timeline);
    assert_rect(
        rect_of(&screen, "corner"),
        (0.0, 0.0, 220.0, 40.0),
        "corner",
    );
    assert_rect(
        rect_of(&screen, "ruler"),
        (220.0, 0.0, 1000.0, 40.0),
        "ruler",
    );
    assert_rect(
        rect_of(&screen, "track0"),
        (0.0, 40.0, 220.0, 60.0),
        "track",
    );

    screen.get_by_test_id("arrange").scroll(300.0, 200.0);
    assert_rect(
        rect_of(&screen, "corner"),
        (0.0, 0.0, 220.0, 40.0),
        "corner stays",
    );
    assert_rect(
        rect_of(&screen, "ruler"),
        (220.0 - 300.0, 0.0, 1000.0, 40.0),
        "ruler: on top, scrolled sideways",
    );
    assert_rect(
        rect_of(&screen, "track0"),
        (0.0, 40.0 - 200.0, 220.0, 60.0),
        "track header: at the left, scrolled up",
    );
    let shot = screen.screenshot();
    // The corner over the ruler (z 6 over 5), the ruler over the track
    // headers (5 over 4), the headers over the lanes.
    expect(&shot, 100, 20, CORNER, 8, "corner");
    expect(&shot, 300, 20, RULER, 8, "ruler beside it");
    expect(&shot, 100, 60, TRACK, 8, "a track header below the corner");
    expect(&shot, 300, 120, LANE, 8, "a lane");
}

#[test]
fn sticky_boxes_stay_within_their_parent_and_follow_a_page_scroll() {
    let (screen, bar) = mount_with(window(800.0, 600.0), || {
        let bar = node_ref();
        let section = |n: usize| {
            column((
                block(0x404040, 800, 30)
                    .test_id(["h0", "h1"][n])
                    .style(style().position(Position::Sticky).top(0)),
                block(0xd0d0d0, 800, 400),
            ))
        };
        let page = scroll(column((
            // A sticky top bar of the page, then two sections with
            // sticky headers.
            block(0x102030, 800, 50)
                .test_id("topbar")
                .node_ref(bar)
                .style(style().position(Position::Sticky).top(0).z_index(10)),
            section(0),
            section(1),
            // A footer stuck to the bottom while its place is below.
            block(0x303030, 800, 20)
                .test_id("footer")
                .style(style().position(Position::Sticky).bottom(0)),
            block(0xffffff, 800, 600),
        )))
        .test_id("page")
        .style(style().height(pct(100)));
        // A menu anchored below the top bar.
        let menu = overlay(
            bar,
            Placement::Below,
            block(0x00ff00, 100, 20).test_id("menu"),
        );
        (bar, (page, menu))
    });
    let observed = bar.rect();
    // At rest: the footer (in flow at 910) is stuck to the window bottom.
    assert_rect(
        rect_of(&screen, "footer"),
        (0.0, 580.0, 800.0, 20.0),
        "footer stuck",
    );
    assert_rect(
        rect_of(&screen, "h0"),
        (0.0, 50.0, 800.0, 30.0),
        "h0 in flow",
    );

    screen.get_by_test_id("page").scroll(0.0, 300.0);
    assert_rect(
        rect_of(&screen, "topbar"),
        (0.0, 0.0, 800.0, 50.0),
        "topbar stuck",
    );
    // Reported (`NodeRef::rect`) and anchored to where it is stuck.
    assert_rect(
        observed.get_untracked().expect("reported"),
        (0.0, 0.0, 800.0, 50.0),
        "the reported rect",
    );
    assert_rect(
        rect_of(&screen, "menu"),
        (0.0, 50.0, 100.0, 20.0),
        "the menu below it",
    );
    // h0's section spans 50..480 before scrolling, -250..180 now: its
    // header sticks at 0 (under the top bar: insets are its own).
    assert_rect(rect_of(&screen, "h0"), (0.0, 0.0, 800.0, 30.0), "h0 stuck");

    screen.get_by_test_id("page").scroll(0.0, 160.0);
    // Section 0 ends at 480 - 460 = 20: its header is pushed up with it.
    assert_rect(
        rect_of(&screen, "h0"),
        (0.0, -10.0, 800.0, 30.0),
        "h0 pushed out",
    );
    assert_rect(
        rect_of(&screen, "h1"),
        (0.0, 20.0, 800.0, 30.0),
        "h1 in flow",
    );
    // The footer's place (910 - 460 = 450) is now in view: in flow.
    assert_rect(
        rect_of(&screen, "footer"),
        (0.0, 450.0, 800.0, 20.0),
        "footer in flow",
    );
}

/// A toast stack fixed at the bottom right, inside a small scrolled and
/// clipping box, and a maximized panel (`inset: 96px 12px 32px`).
fn with_fixed(
    toast_presses: &Rc<Cell<u32>>,
    maximized: Signal<bool>,
    toast: NodeRef,
) -> impl IntoView {
    let toasts = column((
        pressable("toast", toast_presses).child(block(0x2080ff, 200, 40)),
        block(0x20c080, 200, 40),
    ))
    .test_id("toasts")
    .node_ref(toast)
    .style(
        style()
            .position(Position::Fixed)
            .right(16)
            .bottom(16)
            .gap(8)
            .z_index(50),
    );
    let content = column((
        block(0xeeeeee, 300, 100).test_id("first"),
        toasts,
        block(0xdddddd, 300, 900),
    ));
    (
        scroll(content).test_id("box").style(
            style()
                .position(Position::Absolute)
                .left(20)
                .top(20)
                .size(300, 200),
        ),
        show(maximized, || {
            node().test_id("max").style(
                style()
                    .position(Position::Fixed)
                    .top(96)
                    .left(12)
                    .right(12)
                    .bottom(32)
                    .background(rgb(0x806040)),
            )
        }),
    )
}

#[test]
fn a_fixed_toast_stack_sits_in_the_window_corner_unscrolled_and_unclipped() {
    let presses = Rc::new(Cell::new(0));
    let p = presses.clone();
    let (screen, (maximized, toast)) = mount_with(window(800.0, 600.0), move || {
        let (max, toast) = (signal(false), node_ref());
        let view = node()
            .child(with_fixed(&p, max, toast))
            .style(style().size(800, 600).background(Color::WHITE));
        ((max, toast), view)
    });
    let at_corner = |s: &Screen, what: &str| {
        assert_rect(rect_of(s, "toasts"), (584.0, 496.0, 200.0, 88.0), what);
    };
    at_corner(&screen, "at the bottom right");
    let shot = screen.screenshot();
    // Outside the 300 × 200 box that clips its parent, and drawn.
    expect(&shot, 680, 510, 0x2080ff, 8, "the first toast");
    expect(&shot, 680, 560, 0x20c080, 8, "the second toast");
    assert_eq!(pixel(&shot, 680, 540) & 0xffffff, 0xffffff, "the gap");

    // Its scroll container scrolls: it stays.
    screen.get_by_test_id("box").scroll(0.0, 300.0);
    at_corner(&screen, "after a scroll");
    // Revealing it scrolls nothing (its scroll container does not move it).
    toast.scroll_into_view();
    screen.run_until_idle();
    assert_rect(
        rect_of(&screen, "first"),
        (20.0, 20.0 - 300.0, 300.0, 100.0),
        "the box keeps its scroll",
    );
    expect(&screen.screenshot(), 680, 510, 0x2080ff, 8, "still drawn");
    screen.click_at(680.0, 510.0);
    assert_eq!(presses.get(), 1, "the toast is pressed where it is drawn");
    let bounds = screen
        .get_by_role(Role::Button, "toast")
        .accessibility_bounds()
        .expect("exposed");
    assert_rect(bounds, (584.0, 496.0, 200.0, 40.0), "accessibility bounds");

    // A resize keeps it in the corner.
    screen.resize(640.0, 480.0);
    assert_rect(
        rect_of(&screen, "toasts"),
        (424.0, 376.0, 200.0, 88.0),
        "after a resize",
    );

    // inset: 96px 12px 32px
    maximized.set(true);
    screen.run_until_idle();
    assert_rect(
        rect_of(&screen, "max"),
        (12.0, 96.0, 616.0, 352.0),
        "maximized",
    );
    // The toasts (z-index 50) stay over it.
    expect(
        &screen.screenshot(),
        500,
        390,
        0x2080ff,
        8,
        "toast over the panel",
    );
    expect(&screen.screenshot(), 300, 200, 0x806040, 8, "the panel");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn boxes_inserted_among_fixed_and_overlay_siblings_keep_their_flow_order() {
    let (screen, shown) = mount_with(window(800.0, 600.0), || {
        let (shown, anchor) = (signal(false), node_ref());
        let view = column((
            block(0x111111, 100, 10).test_id("a").node_ref(anchor),
            block(0x222222, 100, 10).style(style().position(Position::Fixed).top(300)),
            overlay(anchor, Placement::Below, block(0x333333, 50, 50)),
            show(shown, || block(0x444444, 100, 20).test_id("b")),
            block(0x555555, 100, 10).test_id("c"),
        ));
        (shown, view)
    });
    assert_rect(rect_of(&screen, "c"), (0.0, 10.0, 100.0, 10.0), "c after a");
    shown.set(true);
    screen.run_until_idle();
    assert_rect(rect_of(&screen, "b"), (0.0, 10.0, 100.0, 20.0), "b after a");
    assert_rect(rect_of(&screen, "c"), (0.0, 30.0, 100.0, 10.0), "c after b");
}

#[test]
fn fixed_membership_changes_refresh_layout_children_after_cached_size_changes() {
    let (screen, (height, fixed)) = mount_with(window(800.0, 600.0), || {
        let height = signal(10);
        let fixed = signal(false);
        let view = column((
            node()
                .test_id("a")
                .style(move || style().size(100, height.get())),
            node().test_id("b").style(move || {
                let base = style().size(100, 20);
                if fixed.get() {
                    base.position(Position::Fixed).left(0).top(200)
                } else {
                    base
                }
            }),
            node().test_id("c").style(style().size(100, 10)),
        ));
        ((height, fixed), view)
    });
    assert_rect(
        rect_of(&screen, "c"),
        (0.0, 30.0, 100.0, 10.0),
        "initial flow",
    );

    height.set(30);
    screen.run_until_idle();
    assert_rect(
        rect_of(&screen, "c"),
        (0.0, 50.0, 100.0, 10.0),
        "height changed",
    );

    fixed.set(true);
    screen.run_until_idle();
    assert_rect(
        rect_of(&screen, "b"),
        (0.0, 200.0, 100.0, 20.0),
        "fixed in viewport",
    );
    assert_rect(
        rect_of(&screen, "c"),
        (0.0, 30.0, 100.0, 10.0),
        "fixed leaves flow",
    );

    height.set(40);
    screen.run_until_idle();
    assert_rect(
        rect_of(&screen, "c"),
        (0.0, 40.0, 100.0, 10.0),
        "height while fixed",
    );

    fixed.set(false);
    screen.run_until_idle();
    assert_rect(
        rect_of(&screen, "b"),
        (0.0, 40.0, 100.0, 20.0),
        "returns to flow",
    );
    assert_rect(
        rect_of(&screen, "c"),
        (0.0, 60.0, 100.0, 10.0),
        "flow restored",
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
