//! Role: Tests for layout and scrolling with real text shaping.
//!
//! Layout and scrolling on real windows with real shaping: text wraps to
//! the width its siblings leave, a single-line field never wraps, a wheel
//! scrolls the inner container and, once it is at its edge, the outer one,
//! and `NodeRef::scroll_into_view` reveals a target in nested containers
//! without overshooting (a target taller than the viewport aligns to its
//! top), and `Screen::scroll` wheels at the current pointer position.

use crate::support::mount;
use kivact::ErrorKind;
use kivact::geometry::Rect;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{Screen, render};
use kivact::view::View;

const LONG: &str = "The quick brown fox jumps over the lazy dog and keeps running \
    across the whole field until the line has to wrap at least once more.";

fn rect(screen: &Screen, id: &str) -> Rect {
    screen.get_by_test_id(id).rect().expect("laid out")
}

fn inside(inner: Rect, outer: Rect) -> bool {
    inner.y >= outer.y - 0.5 && inner.y + inner.height <= outer.y + outer.height + 0.5
}

#[test]
fn text_wraps_to_the_width_left_by_its_siblings_and_fields_never_wrap() {
    let screen = render(|| {
        column((
            row((
                text(LONG).test_id("long"),
                node()
                    .test_id("fixed")
                    .style(style().size(100, 20).shrink(0.0)),
            ))
            .style(style().align_items(Align::Start)),
            column((
                text("A").test_id("one line"),
                kivact::view::node()
                    .name("long field")
                    .part(kivact::interaction::use_text_edit(signal(LONG.to_string())).control())
                    .style(style().width(120)),
                kivact::view::node()
                    .name("short field")
                    .part(kivact::interaction::use_text_edit(signal("a".to_string())).control())
                    .style(style().width(120)),
            ))
            .style(style().align_items(Align::Start)),
        ))
    });
    let (long, fixed, line) = (
        rect(&screen, "long"),
        rect(&screen, "fixed"),
        rect(&screen, "one line"),
    );
    assert_eq!(fixed.x, 700.0, "{fixed:?}");
    assert_eq!(long.width, 700.0, "{long:?}");
    assert!(
        long.height >= 2.0 * line.height,
        "wrapped: {long:?} {line:?}"
    );
    let field = |name: &str| screen.get_by_role(Role::TextBox, name);
    let (lf, sf) = (field("long field"), field("short field"));
    assert_eq!(
        lf.rect().map(|r| r.height),
        sf.rect().map(|r| r.height),
        "a single-line field does not wrap"
    );
    lf.press_key(Key::End);
    lf.type_text("X");
    assert!(lf.value().is_some_and(|v| v.ends_with("X")));
}

#[test]
fn wheels_and_scroll_into_view_in_nested_containers() {
    let screen = render(|| {
        let (target, tall) = (node_ref(), node_ref());
        column((
            row((
                {
                    let label = ("reveal").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || target.scroll_into_view()),
                {
                    let label = ("reveal tall").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || tall.scroll_into_view()),
            )),
            scroll(column((
                node().test_id("top").style(style().height(30).shrink(0.0)),
                scroll(column((
                    node().style(style().height(200).shrink(0.0)),
                    node()
                        .test_id("target")
                        .node_ref(target)
                        .style(style().height(20).shrink(0.0)),
                )))
                .test_id("inner")
                .style(style().height(60).shrink(0.0)),
                node().style(style().height(500).shrink(0.0)),
            )))
            .test_id("outer")
            .style(style().height(80)),
            scroll(column((
                node().style(style().height(300).shrink(0.0)),
                node()
                    .test_id("tall")
                    .node_ref(tall)
                    .style(style().height(500).shrink(0.0)),
            )))
            .test_id("viewport")
            .style(style().height(200)),
        ))
    });
    let top_y = rect(&screen, "top").y;
    // A wheel over the inner container scrolls it (up to its end) and
    // not the outer one; at its end the next wheel scrolls the outer one.
    let target_before = rect(&screen, "target").y;
    screen.get_by_test_id("inner").scroll(0.0, 200.0);
    assert!(
        rect(&screen, "target").y < target_before,
        "the inner one scrolled"
    );
    assert_eq!(rect(&screen, "top").y, top_y, "the outer one did not");
    screen.get_by_test_id("inner").scroll(0.0, 20.0);
    assert!(
        rect(&screen, "top").y < top_y,
        "at the edge the outer one scrolls"
    );
    screen.get_by_test_id("outer").scroll(0.0, -2000.0);
    screen.get_by_test_id("inner").scroll(0.0, -2000.0);
    assert_eq!(rect(&screen, "top").y, top_y);

    screen.get_by_role(Role::Button, "reveal").click();
    let (t, inner, outer) = (
        rect(&screen, "target"),
        rect(&screen, "inner"),
        rect(&screen, "outer"),
    );
    assert!(
        inside(t, inner) && inside(t, outer),
        "{t:?} {inner:?} {outer:?}"
    );
    assert!(
        (t.y + t.height - (inner.y + inner.height)).abs() < 0.5,
        "no overshoot: {t:?} {inner:?}"
    );
    screen.get_by_role(Role::Button, "reveal tall").click();
    assert_eq!(
        rect(&screen, "tall").y,
        rect(&screen, "viewport").y,
        "top edge"
    );
}

/// 200 rows of 20 px in a 200 px scroll box: row 10 grows to 40 px with
/// `tall`, row 30 casts a wide shadow, row 50 is moved 300 px up by a
/// transform.
fn reaching_rows(tall: Signal<bool>) -> View {
    let rows: Vec<View> = (0..200u32)
        .map(|i| {
            let row = node().test_id(format!("row {i}"));
            match i {
                10 => {
                    row.style(move || style().height(if tall.get() { 40 } else { 20 }).shrink(0.0))
                }
                30 => row.style(
                    style()
                        .height(20)
                        .shrink(0.0)
                        .shadow(kivact::draw::Shadow::new(0.0, 0.0, 40.0, rgb(0x000000))),
                ),
                50 => row.style(
                    style()
                        .height(20)
                        .shrink(0.0)
                        .background(rgb(0xff0000))
                        .translate(0, -300),
                ),
                _ => row.style(style().height(20).shrink(0.0)),
            }
            .into_view()
        })
        .collect();
    scroll(column(rows))
        .test_id("list")
        .style(style().height(200).shrink(0.0).background(rgb(0xffffff)))
        .into_view()
}

/// A long list draws what reaches its viewport from rows outside it (the
/// host finds the rows that reach it without visiting the others): the
/// shadow of a row just above, a row moved into view from below; and once
/// a row above changes its height (a layout pass moves every row after
/// it), the frame is what a fresh window draws for the new layout.
#[test]
fn a_long_list_draws_what_reaches_its_viewport_from_rows_outside_it() {
    let (screen, tall) = mount(|| {
        let tall = signal(false);
        (tall, reaching_rows(tall))
    });
    let list = screen.get_by_test_id("list");
    let top = rect(&screen, "list");
    // Row 30 (600..620) ends 20 px above the viewport; its blur reaches
    // 72 px.
    list.scroll(0.0, 640.0);
    let shot = screen.screenshot();
    let at = |shot: &kivact::test::Screenshot, x: f64, y: f64| {
        shot.pixel((top.x + x) as u32, (top.y + y) as u32)
            .expect("in the frame")
    };
    let shade = at(&shot, 100.0, 2.0) & 0xff;
    assert!(shade < 250, "the shadow of row 30 shows: gray {shade}");
    assert_eq!(at(&shot, 100.0, 150.0), 0xffffff, "nothing else in view");
    // Row 50 (1000..1020) is drawn 300 px up, at 60..80 in the viewport.
    assert_eq!(at(&shot, 100.0, 70.0), 0xff0000, "the moved row shows");

    tall.set(true);
    screen.run_until_idle();
    let fresh = render(|| reaching_rows(signal(true)));
    fresh.get_by_test_id("list").scroll(0.0, 640.0);
    assert_eq!(
        screen.screenshot().hash(),
        fresh.screenshot().hash(),
        "the same pixels as a fresh frame after the layout pass"
    );
}

/// `scroll(..)` scrolls vertically; `.axes(ScrollAxes::Horizontal)` sideways
/// and `.axes(ScrollAxes::Both)` both ways. The caller's `.style(..)` before
/// or after the direction sets the size without changing the axis, and the
/// accessibility orientation follows the axes.
#[test]
fn a_scroll_container_scrolls_along_the_axis_its_builder_names() {
    fn boxed(id: &'static str) -> Node {
        node()
            .test_id(id)
            .style(style().width(400).height(200).shrink(0.0))
    }
    let screen = render(|| {
        column((
            scroll(column((boxed("v0"), boxed("v1"))))
                .style(style().width(300).height(100))
                .test_id("vertical"),
            scroll(row((boxed("h0"), boxed("h1"))))
                .axes(ScrollAxes::Horizontal)
                .style(style().width(300).height(100))
                .test_id("horizontal"),
            scroll(column((boxed("b0"), boxed("b1"))))
                .style(style().width(300).height(100))
                .axes(ScrollAxes::Both)
                .test_id("both"),
        ))
    });
    let (v, h, b) = (
        rect(&screen, "v0"),
        rect(&screen, "h0"),
        rect(&screen, "b0"),
    );
    assert_eq!(
        screen.get_by_test_id("vertical").orientation(),
        Orientation::Vertical
    );
    assert_eq!(
        screen.get_by_test_id("horizontal").orientation(),
        Orientation::Horizontal
    );
    assert_eq!(
        screen.get_by_test_id("both").orientation(),
        Orientation::Unspecified
    );
    screen.get_by_test_id("vertical").scroll(50.0, 50.0);
    let after = rect(&screen, "v0");
    assert_eq!(after.x, v.x, "the vertical one ignores sideways wheels");
    assert!(after.y < v.y, "and scrolls down");
    screen.get_by_test_id("horizontal").scroll(50.0, 50.0);
    let after = rect(&screen, "h0");
    assert!(after.x < h.x, "the horizontal one scrolls sideways");
    assert_eq!(after.y, h.y, "and not down");
    screen.get_by_test_id("both").scroll(50.0, 50.0);
    let after = rect(&screen, "b0");
    assert!(
        after.x < b.x && after.y < b.y,
        "the last one scrolls both ways"
    );
}

/// A scroll container's accessibility orientation follows its axes; setting another is reported.
#[test]
fn orientation_on_a_scroll_container_is_reported_and_the_axes_win() {
    let screen = render(|| {
        scroll(row((text("a"), text("b"))))
            .axes(ScrollAxes::Horizontal)
            .with_node(move |node| node.orientation(Orientation::Vertical))
            .test_id("strip")
    });
    assert_eq!(
        screen.get_by_test_id("strip").orientation(),
        Orientation::Horizontal
    );
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(*errors[0].kind(), ErrorKind::Unsupported);
    assert!(
        errors[0].message().contains("orientation"),
        "{}",
        errors[0].message()
    );
}
