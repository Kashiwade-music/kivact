//! Role: Tests for layout and scrolling with real text shaping.
//!
//! Layout and scrolling on real windows with real shaping: text wraps to
//! the width its siblings leave, a single-line field never wraps, a wheel
//! scrolls the inner container and, once it is at its edge, the outer one,
//! and `NodeRef::scroll_into_view` reveals a target in nested containers
//! without overshooting (a target taller than the viewport aligns to its
//! top), and `Screen::scroll` wheels at the current pointer position.

use crate::support::mount;
use kivact::draw::Rect;
use kivact::prelude::*;
use kivact::style::{Align, Position, pct};
use kivact::test::{Screen, render};
use kivact::view::RowHeight;
use kivact::view::View;

const LONG: &str = "The quick brown fox jumps over the lazy dog and keeps running \
    across the whole field until the line has to wrap at least once more.";

fn rect(screen: &Screen, id: &str) -> Rect {
    screen.get_by_test_id(id).rect().expect("laid out")
}

/// A text whose width its column imposes keeps the layout when new
/// content fits it at the same height; the same content in a row, whose
/// width it decides, moves what follows; and a column that later becomes
/// a row lays the text out at the width of its current content.
#[test]
fn a_text_stretched_across_a_column_keeps_the_layout_its_width_does_not_change() {
    let (screen, (label, flip)) = mount(|| {
        let label = signal(String::from("v9"));
        let flip = signal(false);
        let view = column((
            column((text(move || label.get()).test_id("stretched"),)).style(move || {
                if flip.get() {
                    style().direction(kivact::style::Direction::Row)
                } else {
                    style()
                }
            }),
            row((
                text(move || label.get()).test_id("in row"),
                node()
                    .test_id("after")
                    .style(style().size(10, 10).shrink(0.0)),
            ))
            .style(style().align_items(Align::Start)),
        ));
        ((label, flip), view)
    });
    let (stretched, after) = (rect(&screen, "stretched"), rect(&screen, "after"));
    label.set("v10 and a much longer label".to_string());
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_test_id("stretched").name(),
        "v10 and a much longer label"
    );
    assert_eq!(
        rect(&screen, "stretched"),
        stretched,
        "the column imposes it"
    );
    assert!(
        rect(&screen, "after").x > after.x,
        "the row follows its text"
    );
    flip.set(true);
    screen.run_until_idle();
    assert_eq!(
        rect(&screen, "stretched").width,
        rect(&screen, "in row").width,
        "in a row the text is as wide as its current content"
    );
}

/// A fixed-width card in an over-full row shrinks down to its text's
/// longest word: a new text with a shorter longest word, wrapping into
/// as many lines, lets it shrink further, and its sibling grows back.
#[test]
fn a_shrinking_card_follows_its_texts_minimum_width() {
    fn card(label: Signal<String>) -> View {
        row((
            column((text(move || label.get())
                .test_id("text")
                .style(style().text_wrap(true)),))
            .test_id("card")
            .style(style().width(150)),
            node().test_id("other").style(style().width(150).height(10)),
        ))
        .style(style().width(200).align_items(Align::Start))
        .into_view()
    }
    let (screen, label) = mount(|| {
        let label = signal(String::from("aaaaaaaaaaaaaaaaaa bb"));
        (label, card(label))
    });
    let other = rect(&screen, "other");
    let next = "aaaaaaaaa bbbbbbbbb";
    label.set(next.to_string());
    screen.run_until_idle();
    let fresh = render(move || card(signal(next.to_string())));
    assert_eq!(
        rect(&fresh, "text").height,
        rect(&screen, "text").height,
        "the new text wraps into as many lines"
    );
    assert!(
        rect(&fresh, "other").width > other.width,
        "the longest word was what held the card"
    );
    assert_eq!(rect(&screen, "card"), rect(&fresh, "card"));
    assert_eq!(rect(&screen, "other"), rect(&fresh, "other"));
}

/// A long scrolling list of rows, each a colored swatch and a label;
/// `edited` recolors the first swatch and relabels the second row.
fn swatches(edited: Signal<bool>) -> View {
    let rows: Vec<View> = (0..120u32)
        .map(|i| {
            row((
                node().style(move || {
                    let color = if i == 0 && edited.get() {
                        0x2a9d8f
                    } else {
                        0x264653 + (i % 7) * 0x101010
                    };
                    style().size(16, 16).shrink(0.0).background(rgb(color))
                }),
                text(move || {
                    if i == 1 && edited.get() {
                        "edited".to_string()
                    } else {
                        format!("row {i}")
                    }
                }),
            ))
            .test_id(format!("row {i}"))
            .style(
                style()
                    .height(24)
                    .shrink(0.0)
                    .gap(8)
                    .align_items(Align::Center),
            )
            .into_view()
        })
        .collect();
    scroll(column(rows))
        .test_id("list")
        .style(style().grow(1).min_height(0))
        .into_view()
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
                kivact::control::node()
                    .name("long field")
                    .part(kivact::interaction::use_text_edit(signal(LONG.to_string())).control())
                    .style(style().width(120)),
                kivact::control::node()
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
fn new_text_moves_the_layout_only_when_its_size_changes() {
    let screen = render(|| {
        let label = signal(String::from("12"));
        let para = signal(String::from("one line"));
        column((
            row((
                {
                    let label = ("same width").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || label.set("21".into())),
                {
                    let label = ("wider").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || label.set("1234567890".into())),
                {
                    let label = ("wrap").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || para.set(format!("{LONG} {LONG}"))),
            )),
            row((
                text(move || label.get()).test_id("label"),
                node()
                    .test_id("after")
                    .style(style().size(10, 10).shrink(0.0)),
            ))
            .style(style().align_items(Align::Start)),
            text(move || para.get())
                .test_id("para")
                .style(style().text_wrap(true)),
            node()
                .test_id("below")
                .style(style().size(10, 10).shrink(0.0)),
        ))
    });
    let after = rect(&screen, "after");
    let (para, below) = (rect(&screen, "para"), rect(&screen, "below"));
    let name = |id: &str| screen.get_by_test_id(id).name();
    // Digits are as wide as each other: the new text keeps its size, and
    // nothing moves.
    screen.get_by_role(Role::Button, "same width").click();
    assert_eq!(name("label"), "21");
    assert_eq!(rect(&screen, "after"), after);
    // A wider text pushes its sibling.
    screen.get_by_role(Role::Button, "wider").click();
    assert_eq!(name("label"), "1234567890");
    assert!(
        rect(&screen, "after").x > after.x,
        "{:?}",
        rect(&screen, "after")
    );
    // Text that wraps grows taller and moves what follows it.
    screen.get_by_role(Role::Button, "wrap").click();
    let (para2, below2) = (rect(&screen, "para"), rect(&screen, "below"));
    assert!(para2.height >= 2.0 * para.height, "{para:?} {para2:?}");
    assert!(below2.y > below.y, "{below:?} {below2:?}");
}

/// Frames draw only what is in view and keep what did not change between
/// frames: rows edited while scrolled out of view, then scrolled back,
/// look exactly as in a fresh frame of the edited list, and the rows left
/// alone keep their pixels.
#[test]
fn rows_edited_out_of_view_are_drawn_as_a_fresh_frame_draws_them() {
    let (screen, edited) = mount(|| {
        let edited = signal(false);
        (edited, swatches(edited))
    });
    let shot = |screen: &Screen, id: &str| screen.get_by_test_id(id).screenshot().hash();
    let before = (
        shot(&screen, "row 0"),
        shot(&screen, "row 1"),
        shot(&screen, "row 2"),
    );
    let list = screen.get_by_test_id("list");
    list.scroll(0.0, 5000.0);
    let bottom = screen.screenshot().hash();
    edited.set(true);
    screen.run_until_idle();
    assert_eq!(
        screen.screenshot().hash(),
        bottom,
        "edits out of view change no pixel"
    );
    list.scroll(0.0, -5000.0);
    let after = (
        shot(&screen, "row 0"),
        shot(&screen, "row 1"),
        shot(&screen, "row 2"),
    );
    assert_ne!(after.0, before.0, "the recolored swatch shows");
    assert_ne!(after.1, before.1, "the new label shows");
    assert_eq!(after.2, before.2, "the untouched row keeps its pixels");
    let fresh = render(|| swatches(signal(true)));
    assert_eq!(
        screen.screenshot().hash(),
        fresh.screenshot().hash(),
        "the same pixels as a fresh frame"
    );
}

/// Layouts after hiding, editing, removing, reordering and remounting
/// differently sized subtrees have the same geometry and pixels as fresh
/// windows. In particular, showing edited descendants cannot reuse their
/// old sizes or the sizes of the removed rows.
#[test]
fn edited_hidden_and_remounted_rows_match_fresh_layouts() {
    fn rows(keys: Signal<Vec<u32>>, tall: Signal<bool>, hidden: Signal<bool>) -> View {
        column((
            column(for_each(
                keys,
                |key| *key,
                move |key, _| {
                    node()
                        .test_id(format!("tile {key}"))
                        .child(text(format!("row {key}")))
                        .style(move || {
                            style()
                                .width(pct(100) - (key % 3) as i32)
                                .height(if tall.get() { 27 } else { 19 } + (key % 4) as i32)
                                .shrink(0)
                                .background(rgb(0x2255cc))
                        })
                },
            ))
            .hidden(hidden)
            .style(style().width(pct(100)).shrink(0)),
            node()
                .test_id("after tiles")
                .style(style().size(12, 12).background(rgb(0xcc3322)).shrink(0)),
        ))
        .style(style().width(320).align_items(Align::Start))
        .into_view()
    }

    let (screen, (keys, tall, hidden)) = mount(|| {
        let keys = signal((0..6).collect::<Vec<u32>>());
        let tall = signal(false);
        let hidden = signal(false);
        ((keys, tall, hidden), rows(keys, tall, hidden))
    });
    let check = || {
        let expected_keys = keys.get_untracked();
        let expected_tall = tall.get_untracked();
        let expected_hidden = hidden.get_untracked();
        let fresh_keys = expected_keys.clone();
        let fresh = render(move || {
            rows(
                signal(fresh_keys),
                signal(expected_tall),
                signal(expected_hidden),
            )
        });
        assert_eq!(rect(&screen, "after tiles"), rect(&fresh, "after tiles"));
        if !expected_hidden {
            for key in expected_keys {
                let id = format!("tile {key}");
                assert_eq!(rect(&screen, &id), rect(&fresh, &id), "{id}");
            }
        }
        assert_eq!(screen.screenshot().hash(), fresh.screenshot().hash());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
        assert!(fresh.errors().is_empty(), "{:?}", fresh.errors());
    };
    check();
    for cycle in 0..4 {
        hidden.set(true);
        screen.run_until_idle();
        tall.set(cycle % 2 == 0);
        keys.set((100 * cycle..100 * cycle + 9).collect());
        screen.run_until_idle();
        check();
        hidden.set(false);
        screen.run_until_idle();
        check();
        keys.set(Vec::new());
        screen.run_until_idle();
        check();
        keys.set((100 * cycle + 20..100 * cycle + 35).rev().collect());
        tall.set(cycle % 2 != 0);
        screen.run_until_idle();
        check();
        keys.update(|keys| keys.reverse());
        screen.run_until_idle();
        check();
    }
}

/// A form, a virtual list and a clipping list of lines (rows that scroll
/// out of it are hidden from assistive technology).
fn form(agree: Signal<bool>, note: Signal<String>) -> View {
    let items: Vec<u32> = (0..60).collect();
    let lines: Vec<View> = (0..40)
        .map(|i| text(format!("line {i}")).into_view())
        .collect();
    column((
        node()
            .role(Role::CheckBox)
            .focusable(true)
            .name("agree")
            .child(text("agree"))
            .checked(agree)
            .on_press(move || agree.update(|value| *value = !*value)),
        kivact::control::node()
            .name("note")
            .part(kivact::interaction::use_text_edit(note).control()),
        virtual_list(
            items,
            |i| *i,
            |i, _| {
                let label = (format!("item {i}")).into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
        )
        .row_height(RowHeight::Fixed(24.0))
        .test_id("items")
        .style(style().height(120)),
        scroll(column(lines))
            .role(Role::List)
            .test_id("lines")
            .style(style().height(100)),
    ))
    .into_view()
}

/// The accessibility tree the host keeps up to date from changes alone
/// (a toggle, typing, scrolls, focus moves) reads exactly like one a fresh
/// window builds for the same state.
#[test]
fn the_accessibility_tree_kept_up_to_date_reads_like_a_fresh_one() {
    let screen = render(|| form(signal(false), signal(String::new())));
    screen.get_by_role(Role::CheckBox, "agree").click();
    let note = screen.get_by_role(Role::TextBox, "note");
    note.focus();
    note.type_text("hi");
    let lines = screen.get_by_test_id("lines");
    lines.scroll(0.0, 300.0);
    lines.scroll(0.0, -100.0);
    screen.get_by_test_id("items").scroll(0.0, 200.0);
    screen.get_by_role(Role::CheckBox, "agree").focus();

    let fresh = render(|| form(signal(true), signal("hi".to_string())));
    fresh.get_by_test_id("lines").scroll(0.0, 200.0);
    fresh.get_by_test_id("items").scroll(0.0, 200.0);
    fresh.get_by_role(Role::CheckBox, "agree").focus();
    // Node ids differ between the windows; the focus line names one.
    let tree = |screen: &Screen| {
        screen
            .accessibility_tree()
            .lines()
            .filter(|l| !l.starts_with("focus:"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(tree(&screen), tree(&fresh));
    assert!(tree(&screen).contains("line 9"), "{}", tree(&screen));
}

/// A clipping list whose lines scroll inside a scroller without a role,
/// with an image between the last line in view and the next: `named`
/// names the image, which makes it a row of the list.
fn nested_lines(named: Signal<bool>) -> View {
    let rows: Vec<View> = (0..12)
        .map(|i| {
            if i == 5 {
                node()
                    .role(Role::Image)
                    .name(move || {
                        if named.get() {
                            "logo".to_string()
                        } else {
                            String::new()
                        }
                    })
                    .style(style().size(24, 24).shrink(0.0))
                    .into_view()
            } else {
                text(format!("line {i}"))
                    .style(style().height(24).shrink(0.0))
                    .into_view()
            }
        })
        .collect();
    node()
        .role(Role::List)
        .test_id("list")
        .style(style().height(100).clip(true))
        .child(
            scroll(column(rows))
                .role(Role::Generic)
                .test_id("scroller")
                .style(style().height(100)),
        )
        .into_view()
}

/// Rows of a clipping list that a scroller without a role moves, and a
/// decorative image named into a row (its neighbour leaves the rows near
/// the view), are hidden from assistive technology exactly as a fresh
/// window hides them.
#[test]
fn lines_scrolled_or_pushed_out_of_a_list_read_like_a_fresh_window() {
    let tree = |screen: &Screen| {
        screen
            .accessibility_tree()
            .lines()
            .filter(|l| !l.starts_with("focus:"))
            .collect::<Vec<_>>()
            .join(
                "
",
            )
    };
    let (screen, named) = mount(|| {
        let named = signal(false);
        (named, nested_lines(named))
    });
    named.set(true);
    screen.run_until_idle();
    let fresh = render(|| nested_lines(signal(true)));
    assert_eq!(tree(&screen), tree(&fresh));
    screen.get_by_test_id("scroller").scroll(0.0, 150.0);
    fresh.get_by_test_id("scroller").scroll(0.0, 150.0);
    assert_eq!(tree(&screen), tree(&fresh));
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

/// `Screen::scroll` wheels where the pointer already is: over the first
/// box it scrolls that box and not its neighbour, and after the pointer
/// moves to the neighbour the next wheel scrolls the neighbour.
#[test]
fn screen_scroll_wheels_at_the_current_pointer_position() {
    let screen = render(|| {
        let pane = |id: &'static str| {
            scroll(column((node()
                .test_id(format!("{id} content"))
                .style(style().height(400).shrink(0.0)),)))
            .test_id(id)
            .style(style().width(100).height(100).shrink(0.0))
        };
        row((pane("left"), pane("right")))
    });
    let (left_top, right_top) = (
        rect(&screen, "left content").y,
        rect(&screen, "right content").y,
    );
    let left = rect(&screen, "left");
    screen.mouse_move(left.center().x, left.center().y);
    screen.scroll(0.0, 50.0);
    assert!(
        rect(&screen, "left content").y < left_top,
        "the hit box scrolled"
    );
    assert_eq!(
        rect(&screen, "right content").y,
        right_top,
        "its neighbour did not"
    );
    screen.scroll(0.0, -50.0);
    assert_eq!(
        rect(&screen, "left content").y,
        left_top,
        "a second wheel at the same point"
    );

    let right = rect(&screen, "right");
    screen.mouse_move(right.center().x, right.center().y);
    screen.scroll(0.0, 50.0);
    assert!(rect(&screen, "right content").y < right_top);
    assert_eq!(rect(&screen, "left content").y, left_top);
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
    let at = |shot: &kivact::test::Screenshot, x: f32, y: f32| {
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

/// A list `height` tall whose first ten rows fill 200 px, with a row just
/// below them (205..245) that draws into them only through its
/// descendants: an absolutely placed badge 12 px above a box, a card with
/// a wide shadow in a wrapper of its size, a child translated up out of
/// its wrapper, and a child rotated and scaled out of its wrapper.
fn spilling_rows(height: f32) -> View {
    let mut rows: Vec<View> = (0..10u32)
        .map(|_| node().style(style().height(20).shrink(0.0)).into_view())
        .collect();
    rows.push(node().style(style().height(5).shrink(0.0)).into_view());
    rows.push(
        row((
            node().style(style().size(100, 40).shrink(0.0)).child(
                node().style(
                    style()
                        .position(kivact::style::Position::Absolute)
                        .top(-12)
                        .left(10)
                        .size(30, 20)
                        .background(rgb(0xff0000)),
                ),
            ),
            node()
                .style(style().size(80, 40).shrink(0.0).margin_x(20))
                .child(
                    node().style(
                        style()
                            .size(80, 40)
                            .background(rgb(0xffffff))
                            .shadow(kivact::draw::Shadow::new(0.0, 0.0, 24.0, rgb(0x000000))),
                    ),
                ),
            node().style(style().size(80, 40).shrink(0.0)).child(
                node().style(
                    style()
                        .size(80, 40)
                        .background(rgb(0x0000ff))
                        .translate(0, -30),
                ),
            ),
            node()
                .style(style().size(40, 40).shrink(0.0).margin_x(20))
                .child(
                    node().style(
                        style()
                            .size(30, 30)
                            .margin(5)
                            .background(rgb(0x00ff00))
                            .rotate(45)
                            .scale(2),
                    ),
                ),
        ))
        .style(style().height(40).shrink(0.0))
        .into_view(),
    );
    scroll(column(rows))
        .test_id("list")
        .style(style().height(height).shrink(0.0).background(rgb(0xffffff)))
        .into_view()
}

/// Culling never changes a pixel: a row just below a list's viewport that
/// draws into it only through its descendants (a badge placed above it, a
/// card's shadow inside a wrapper of the card's size, a child translated
/// up, a child rotated and scaled up out of its wrapper) shows exactly as
/// in a taller list that culls nothing.
#[test]
fn descendants_that_draw_out_of_a_row_below_the_view_show() {
    let screen = render(|| spilling_rows(200.0));
    let reference = render(|| spilling_rows(260.0));
    let (shot, whole) = (screen.screenshot(), reference.screenshot());
    let top = rect(&screen, "list");
    let at = |shot: &kivact::test::Screenshot, x: u32, y: u32| {
        shot.pixel(top.x as u32 + x, top.y as u32 + y)
            .expect("in the frame")
    };
    assert_eq!(at(&shot, 20, 196), 0xff0000, "the badge shows");
    assert!(at(&shot, 160, 199) & 0xff < 250, "the card's shadow shows");
    assert_eq!(at(&shot, 250, 190), 0x0000ff, "the translated child shows");
    // A 30 px square rotated 45° and scaled 2× about its centre (340, 225)
    // reaches 42 px up.
    assert_eq!(
        at(&shot, 340, 192),
        0x00ff00,
        "the rotated, scaled child shows"
    );
    let differing = (0..200)
        .flat_map(|y| (0..400).map(move |x| (x, y)))
        .filter(|(x, y)| at(&shot, *x, *y) != at(&whole, *x, *y))
        .count();
    assert_eq!(differing, 0, "pixels differing from the unculled list");
}

/// A node built but not mounted yet (its parent is not in the window's
/// tree) has nothing to scroll into view: the request is dropped instead
/// of written for a node the engine does not have.
#[test]
fn scroll_into_view_of_a_node_not_mounted_yet_is_dropped() {
    let screen = render(|| {
        let target = node_ref();
        // The list's rows are built while their column is mounted, after
        // the target was bound and before the column joins the window.
        column((
            node()
                .test_id("target")
                .node_ref(target)
                .style(style().height(20)),
            for_each(
                || vec![1],
                |row: &i32| *row,
                move |_, _| {
                    target.scroll_into_view();
                    text("row")
                },
            ),
        ))
    });
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.get_by_test_id("target");
}

/// A scroll box over a slider-like track (an absolute thumb and fill), an
/// absolute badge whose text and height change (it may reach below the
/// content, growing the scroll range), and a box that may become
/// absolute.
fn absolute_boxes(
    at: Signal<f32>,
    label: Signal<String>,
    tall: Signal<bool>,
    out: Signal<bool>,
) -> View {
    scroll(
        column((
            node()
                .test_id("track")
                .style(
                    style()
                        .position(Position::Relative)
                        .width(200)
                        .height(8)
                        .shrink(0),
                )
                .child((
                    node().test_id("fill").style(move || {
                        style()
                            .position(Position::Absolute)
                            .left(0)
                            .top(0)
                            .height(pct(100))
                            .width(at.get() + 8.0)
                            .background(rgb(0x2255cc))
                    }),
                    node().test_id("thumb").style(move || {
                        style()
                            .position(Position::Absolute)
                            .left(at.get())
                            .top(-4)
                            .size(16, 16)
                            .background(rgb(0xcc3322))
                    }),
                )),
            text("between").test_id("between"),
            node()
                .test_id("holder")
                .style(style().position(Position::Relative).height(40).shrink(0))
                .child(
                    column(text(move || label.get()).test_id("badge text"))
                        .test_id("badge")
                        .style(move || {
                            style()
                                .position(Position::Absolute)
                                .right(0)
                                .top(10)
                                .height(if tall.get() { 400 } else { 20 })
                                .background(rgb(0x22aa55))
                        }),
                ),
            node().test_id("mover").style(move || {
                let s = style().size(30, 30).shrink(0).background(rgb(0x777777));
                if out.get() {
                    s.position(Position::Absolute).left(50).top(0)
                } else {
                    s
                }
            }),
            node().test_id("last").style(style().size(10, 10).shrink(0)),
        ))
        .style(style().width(pct(100)).gap(4)),
    )
    .test_id("scroller")
    .style(style().width(300).height(150).shrink(0))
    .into_view()
}

/// A change of an absolute box (its insets, its size, the text inside it)
/// lays out only its parent again, unless the parent's size or overflow
/// changes; either way, the boxes, the scroll range and the frame match a
/// fresh window of the same state, and a box that becomes absolute leaves
/// the flow.
#[test]
fn absolute_boxes_moved_resized_or_edited_lay_out_like_a_fresh_window() {
    let ids = [
        "track",
        "fill",
        "thumb",
        "between",
        "holder",
        "badge",
        "badge text",
        "mover",
        "last",
    ];
    let (screen, (at, label, tall, out)) = mount(|| {
        let at = signal(0.0f32);
        let label = signal(String::from("new"));
        let tall = signal(false);
        let out = signal(false);
        ((at, label, tall, out), absolute_boxes(at, label, tall, out))
    });
    let check = |what: &str| {
        let state = (
            at.get_untracked(),
            label.get_untracked(),
            tall.get_untracked(),
            out.get_untracked(),
        );
        let fresh_state = state.clone();
        let fresh = render(move || {
            let (a, l, t, o) = fresh_state;
            absolute_boxes(signal(a), signal(l), signal(t), signal(o))
        });
        for id in ids {
            assert_eq!(rect(&screen, id), rect(&fresh, id), "{what}: {id}");
        }
        assert_eq!(
            screen.screenshot().hash(),
            fresh.screenshot().hash(),
            "{what}"
        );
        // The scroll range: wheel both to the end.
        let scroller = rect(&screen, "scroller");
        for s in [&screen, &fresh] {
            s.mouse_move(scroller.center().x, scroller.center().y);
            s.scroll(0.0, 2000.0);
        }
        assert_eq!(
            rect(&screen, "last"),
            rect(&fresh, "last"),
            "{what}: scrolled"
        );
        assert_eq!(
            rect(&screen, "badge"),
            rect(&fresh, "badge"),
            "{what}: scrolled"
        );
        screen.scroll(0.0, -2000.0);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    };
    check("mounted");
    let thumb = rect(&screen, "thumb");
    let between = rect(&screen, "between");
    for x in [40.0, 41.5, 120.0, 0.0] {
        at.set(x);
        screen.run_until_idle();
        check(&format!("thumb at {x}"));
        assert_eq!(
            rect(&screen, "thumb").x - thumb.x,
            x.round(),
            "moved by its inset"
        );
        assert_eq!(rect(&screen, "between"), between, "the flow stays");
    }
    label.set("a much longer label".into());
    screen.run_until_idle();
    check("edited badge");
    tall.set(true);
    screen.run_until_idle();
    check("tall badge");
    tall.set(false);
    screen.run_until_idle();
    check("short badge");
    let last = rect(&screen, "last");
    out.set(true);
    screen.run_until_idle();
    check("absolute mover");
    assert!(rect(&screen, "last").y < last.y, "the mover left the flow");
    out.set(false);
    screen.run_until_idle();
    check("relative mover");
    assert_eq!(rect(&screen, "last"), last);
}

/// `scroll(..)` scrolls vertically; `.orientation(Horizontal)` sideways and
/// `.both_axes()` both ways. The caller's `.style(..)` before or after the
/// direction sets the size without changing the axis.
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
                .orientation(Orientation::Horizontal)
                .style(style().width(300).height(100))
                .test_id("horizontal"),
            scroll(column((boxed("b0"), boxed("b1"))))
                .style(style().width(300).height(100))
                .both_axes()
                .test_id("both"),
        ))
    });
    let (v, h, b) = (
        rect(&screen, "v0"),
        rect(&screen, "h0"),
        rect(&screen, "b0"),
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
