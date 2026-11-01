//! Role: Tests for control flow builders and flow primitives.
//!
//! The flow builders (`show`, `show_some`, `case`,
//! `switch`, `for_each`, `for_index`, `virtual_list`, `dynamic`,
//! `error_boundary`) with plain-function components.

use kivact::primitive::NodeExt;
use kivact::view::View;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::control::Node;
use kivact::diagnostics::ErrorKind;
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::view::{RowHeight, error_boundary};

/// A component with its own state: a click counter.
fn clicker(label: &'static str) -> impl IntoView {
    let n = signal(0);
    {
        let label = (move || format!("{label} {}", n.get())).into_value();
        node()
            .role(Role::Button)
            .focusable(true)
            .name(label)
            .child(text(label))
    }
    .on_press(move || n.update(|v| *v += 1))
}

#[test]
fn show_sets_up_plain_components_in_the_branch() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            show(open, || clicker("a")).fallback(|| text("closed")),
        )
    });
    screen.get_by_role(Role::Button, "a 0").click();
    screen.get_by_role(Role::Button, "a 1");
    open.set(false);
    screen.run_until_idle();
    screen.get_by_text("closed");
    open.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "a 0"); // fresh branch, fresh state
    assert!(screen.errors().is_empty());
}

fn editor(id: Value<u64>) -> impl IntoView {
    let edits = signal(0);
    column((
        text(move || format!("editing {}", id.get())),
        {
            let label = (move || format!("edits {}", edits.get())).into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .on_press(move || edits.update(|e| *e += 1)),
    ))
}

#[test]
fn show_some_takes_a_component_point_free_and_keeps_it_on_some_to_some() {
    let (screen, sel) = mount(|| {
        let sel = signal(Some(1u64));
        (sel, show_some(sel, editor).fallback(|| text("nothing")))
    });
    screen.get_by_text("editing 1");
    screen.get_by_role(Role::Button, "edits 0").click();
    sel.set(Some(2));
    screen.run_until_idle();
    screen.get_by_text("editing 2");
    screen.get_by_role(Role::Button, "edits 1"); // the branch (and its state) is kept
    sel.set(None);
    screen.run_until_idle();
    screen.get_by_text("nothing");
}

#[test]
fn dynamic_over_a_memoed_arm_picks_the_first_true_case() {
    let (screen, n) = mount(|| {
        let n = signal(0);
        let arm = memo(move || {
            if n.get() > 10 {
                2
            } else if n.get() > 0 {
                1
            } else {
                0
            }
        });
        let view = dynamic(arm, |arm| match arm {
            2 => text("big").into_view(),
            1 => clicker("pos").into_view(),
            _ => text("zero").into_view(),
        });
        (n, view)
    });
    screen.get_by_text("zero");
    n.set(1);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "pos 0").click();
    n.set(2); // same case: branch kept
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "pos 1");
    n.set(20);
    screen.run_until_idle();
    screen.get_by_text("big");
}

#[derive(Clone, PartialEq)]
struct Task {
    id: u64,
    title: &'static str,
}

fn task_row(id: u64, task: Value<Task>) -> impl IntoView {
    let clicks = signal(0);
    {
        let label = (move || format!("{} #{id} {}", task.get().title, clicks.get())).into_value();
        node()
            .role(Role::Button)
            .focusable(true)
            .name(label)
            .child(text(label))
    }
    .on_press(move || clicks.update(|c| *c += 1))
}

fn tasks(ids: &[u64]) -> Vec<Task> {
    ids.iter().map(|&id| Task { id, title: "t" }).collect()
}

#[test]
fn for_each_rows_are_plain_functions_and_keep_state_by_key() {
    let (screen, list) = mount(|| {
        let list = signal(tasks(&[1, 2, 3]));
        (list, column(for_each(list, |t| t.id, task_row)))
    });
    screen.get_by_role(Role::Button, "t #2 0").click();
    list.set(tasks(&[3, 2, 1, 4]));
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "t #2 1"); // moved, state kept
    screen.get_by_role(Role::Button, "t #4 0");
    // Closures without annotations work as rows too.
    let (screen, ()) = mount(|| {
        let view = column(for_each(
            vec![1u32, 2],
            |n| *n,
            |k, v| text(move || format!("{k}:{}", v.get())),
        ));
        ((), view)
    });
    screen.get_by_text("2:2");
}

#[test]
fn index_each_rows_follow_their_index() {
    let (screen, list) = mount(|| {
        let list = signal(vec!["a", "b"]);
        let view = column(for_index(list, |i, v| {
            let seen = signal(0);
            {
                let label = (move || format!("{i}={} {}", v.get(), seen.get())).into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || seen.update(|s| *s += 1))
        }));
        (list, view)
    });
    screen.get_by_role(Role::Button, "0=a 0").click();
    list.set(vec!["z", "b", "c"]);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "0=z 1");
    screen.get_by_role(Role::Button, "2=c 0");
}

#[test]
fn virtual_list_mounts_visible_rows_with_plain_rows() {
    let (screen, ()) = mount(|| {
        let items: Vec<u32> = (0..1000).collect();
        let view = virtual_list(
            items,
            |n| *n,
            |k, v| text(move || format!("row {k} {}", v.get())),
        )
        .row_height(RowHeight::Fixed(20.0))
        .style(style().height(200));
        ((), view)
    });
    screen.get_by_text("row 0 0");
    assert!(screen.query_by_text("row 900 900").is_none());
}

/// A plain-function component that creates state: fine in `dynamic`'s
/// builder (a setup frame), a located error in its pure source.
fn mode_view(mode: u32) -> impl IntoView {
    let local = signal(mode);
    text(move || format!("mode {}", local.get()))
}

#[test]
fn dynamic_builds_components_in_a_setup_frame_and_tracks_only_the_source() {
    let builds = Rc::new(Cell::new(0));
    let b2 = builds.clone();
    let cleanups = Rc::new(Cell::new(0));
    let c2 = cleanups.clone();
    let (screen, (mode, other)) = mount(move || {
        let mode = signal(1u32);
        let other = signal(0);
        let view = dynamic(mode, move |m| {
            b2.set(b2.get() + 1);
            let _ = other.get(); // untracked: not a dependency
            let c3 = c2.clone();
            on_cleanup(move || c3.set(c3.get() + 1));
            mode_view(m)
        });
        ((mode, other), view)
    });
    screen.get_by_text("mode 1");
    assert_eq!(builds.get(), 1);
    other.set(5);
    screen.run_until_idle();
    assert_eq!(builds.get(), 1, "the builder is untracked");
    mode.set(2);
    screen.run_until_idle();
    screen.get_by_text("mode 2");
    assert_eq!(
        (builds.get(), cleanups.get()),
        (2, 1),
        "rebuilt, old branch disposed"
    );
    // Point-free with a component taking the value.
    let (screen, ()) = mount(|| ((), dynamic(move || 3u32, mode_view)));
    screen.get_by_text("mode 3");
    assert!(screen.errors().is_empty());
}

#[test]
fn dynamic_keeps_the_branch_when_the_source_value_is_equal() {
    let builds = Rc::new(Cell::new(0));
    let b2 = builds.clone();
    let (screen, n) = mount(move || {
        let n = signal(1u32);
        let view = dynamic(
            move || n.get() / 10,
            move |tens| {
                b2.set(b2.get() + 1);
                clicker(if tens == 0 { "ones" } else { "tens" })
            },
        );
        (n, view)
    });
    screen.get_by_role(Role::Button, "ones 0").click();
    n.set(2); // the source re-runs and yields the same value
    screen.run_until_idle();
    assert_eq!(builds.get(), 1, "an equal value does not rebuild");
    screen.get_by_role(Role::Button, "ones 1"); // the branch state is kept
    n.set(12);
    screen.run_until_idle();
    assert_eq!(builds.get(), 2);
    screen.get_by_role(Role::Button, "tens 0");
}

#[test]
fn a_component_in_the_pure_source_of_dynamic_is_a_located_error() {
    let source = || {
        let _ = mode_view(1);
        1u32
    };
    let (screen, ()) = mount(move || ((), dynamic(source, mode_view)));
    assert!(screen.errors().iter().any(|e| matches!(
        e.kind(),
        ErrorKind::SetupOnly { api: "signal" }
    )
        && e.location().is_some_and(|l| l.file().ends_with("flows.rs"))));
}

fn failing(label: &'static str) -> impl IntoView {
    {
        let label = label.into_value();
        node()
            .role(Role::Button)
            .focusable(true)
            .name(label)
            .child(text(label))
    }
    .on_press(move || Err::<(), _>(UiError::msg(format!("{label} failed"))))
}

#[test]
fn error_boundary_fallback_replaces_the_child_and_reset_remounts() {
    let (screen, ()) = mount(|| {
        let view = error_boundary(|| failing("save")).fallback(|e, reset| {
            column((
                text(e.to_string()).test_id("error"),
                {
                    let label = ("retry").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || reset()),
            ))
        });
        ((), view)
    });
    screen.get_by_role(Role::Button, "save").click();
    assert!(
        screen
            .get_by_test_id("error")
            .text()
            .contains("save failed")
    );
    assert!(screen.query_by_text("save").is_none());
    screen.get_by_role(Role::Button, "retry").click();
    screen.get_by_role(Role::Button, "save");
    assert!(
        screen.errors().is_empty(),
        "handled errors do not reach the sink"
    );
}

#[test]
fn error_boundary_on_error_receives_and_keeps_the_child() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let g2 = got.clone();
    let (screen, ()) = mount(move || {
        let view = error_boundary(|| (clicker("count"), failing("save")))
            .on_error(move |e| g2.borrow_mut().push(e.to_string()));
        ((), view)
    });
    screen.get_by_role(Role::Button, "count 0").click();
    screen.get_by_role(Role::Button, "save").click();
    screen.get_by_role(Role::Button, "save").click();
    assert_eq!(got.borrow().len(), 2);
    assert!(got.borrow()[0].contains("save failed"));
    screen.get_by_role(Role::Button, "count 1"); // child (and its state) kept
    assert!(screen.errors().is_empty());
}

#[test]
fn error_boundary_with_both_runs_on_error_first_then_shows_the_fallback() {
    let order = Rc::new(RefCell::new(Vec::new()));
    let (o2, o3, o4) = (order.clone(), order.clone(), order.clone());
    let (screen, ()) = mount(move || {
        let outer = error_boundary(move || {
            let (o5, o6) = (o2.clone(), o3.clone());
            error_boundary(|| failing("save"))
                .on_error(move |_| o5.borrow_mut().push("on_error"))
                .fallback(move |_, _| {
                    o6.borrow_mut().push("fallback");
                    failing("fallback button")
                })
        })
        .on_error(move |e| {
            o4.borrow_mut().push(if e.to_string().contains("fallback") {
                "parent"
            } else {
                "parent (child error)"
            })
        });
        ((), outer)
    });
    screen.get_by_role(Role::Button, "save").click();
    assert_eq!(*order.borrow(), vec!["on_error", "fallback"]);
    // While the fallback is shown, its errors go to the parent handler.
    screen.get_by_role(Role::Button, "fallback button").click();
    assert_eq!(*order.borrow(), vec!["on_error", "fallback", "parent"]);
}

#[test]
fn a_panicking_on_error_without_fallback_forwards_the_original_error_too() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let g2 = got.clone();
    let (screen, ()) = mount(move || {
        let view = error_boundary(|| {
            error_boundary(|| failing("save")).on_error(|_| panic!("on_error broke"))
        })
        .on_error(move |e| g2.borrow_mut().push(e.to_string()));
        ((), view)
    });
    screen.get_by_role(Role::Button, "save").click();
    let got = got.borrow();
    assert_eq!(got.len(), 2, "{got:?}");
    assert!(got.iter().any(|e| e.contains("on_error broke")), "{got:?}");
    assert!(
        got.iter().any(|e| e.contains("save failed")),
        "the original error is not lost: {got:?}"
    );
    screen.get_by_role(Role::Button, "save"); // the child stays
}

#[test]
fn error_boundary_without_handlers_forwards_and_isolates_setup_panics() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let g2 = got.clone();
    let (screen, ()) = mount(move || {
        let view = error_boundary(move || {
            column((
                error_boundary(|| failing("save")),
                error_boundary(|| -> Node { panic!("setup broke") }),
                text("sibling"),
            ))
        })
        .on_error(move |e| g2.borrow_mut().push(e.kind().clone()));
        ((), view)
    });
    screen.get_by_text("sibling");
    assert_eq!(
        *got.borrow(),
        vec![ErrorKind::Panic],
        "the setup panic reached the parent"
    );
    screen.get_by_role(Role::Button, "save").click();
    assert_eq!(
        got.borrow().len(),
        2,
        "the handler error reached the parent"
    );
}

// ---------------------------------------------------------------------------
// Regression (design survey): errors raised inside an error_boundary's fallback are swallowed.

#[test]
fn errors_in_an_error_boundary_fallback_are_not_lost() {
    let (screen, ()) = mount(|| {
        (
            (),
            error_boundary(|| -> View { panic!("child failed") })
                .fallback(|_err, _reset| -> View { panic!("fallback failed too") }),
        )
    });
    let msgs: Vec<String> = screen.errors().iter().map(|e| e.to_string()).collect();
    assert!(
        msgs.iter().any(|m| m.contains("fallback failed too")),
        "fallback error vanished: {msgs:?}"
    );
}

/// The root of a `virtual_list` resolves its style like every other node:
/// its disabled state style shows in the frame's pixels, a paint
/// transition animates it on the virtual clock, and it still scrolls.
#[test]
fn a_virtual_list_root_takes_state_styles_and_paint_transitions() {
    let (screen, dim) = mount(|| {
        let dim = signal(false);
        let rows: Vec<u32> = (0..40).collect();
        let view = column((
            virtual_list(rows.clone(), |v| *v, |k, _| text(format!("row {k}")))
                .row_height(RowHeight::Fixed(20.0))
                .disabled(true)
                .style(
                    style()
                        .height(100)
                        .background(rgb(0x0000ff))
                        .disabled(style().background(rgb(0xff0000))),
                )
                .test_id("disabled"),
            virtual_list(rows, |v| *v, |k, _| text(format!("other {k}")))
                .row_height(RowHeight::Fixed(20.0))
                .style(move || {
                    style()
                        .height(100)
                        .background(if dim.get() {
                            rgb(0x000000)
                        } else {
                            rgb(0xffffff)
                        })
                        .transition(
                            kivact::style::TransitionProp::Paint,
                            Transition::tween(100, Easing::Linear),
                        )
                })
                .test_id("animated"),
        ));
        (dim, view)
    });
    // A point right of the row texts shows the list's own background.
    let px = |id: &str| {
        let r = screen.get_by_test_id(id).rect().expect("laid out");
        screen
            .screenshot()
            .pixel((r.x + r.width - 5.0) as u32, (r.y + 50.0) as u32)
            .expect("in the frame")
    };
    assert_eq!(px("disabled"), 0xff0000, "the disabled state style");
    screen.get_by_test_id("disabled").scroll(0.0, 200.0);
    screen.get_by_text("row 12");
    assert_eq!(px("animated"), 0xffffff);
    dim.set(true);
    screen.run_until_idle();
    screen.advance(std::time::Duration::from_millis(50));
    let mid = px("animated") & 0xff;
    assert!(mid > 0x10 && mid < 0xf0, "half way: {mid:#x}");
    screen.advance(std::time::Duration::from_millis(200));
    assert_eq!(px("animated"), 0x000000);
}

/// A virtual list of focusable rows "row N" (24 px high, no overscan, a
/// 120 px viewport) and what drives it.
struct Rows {
    target: Signal<Option<u32>>,
    list: NodeRef,
}

fn rows_list(count: u32, height: RowHeight, keep_focused: bool) -> (kivact::test::Screen, Rows) {
    mount(move || {
        let target = signal(None::<u32>);
        let list = node_ref();
        let rows: Vec<u32> = (0..count).collect();
        let view = virtual_list(
            rows,
            |n| *n,
            |k, _| {
                let name = format!("row {k}");
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(name.clone())
                    .child(text(name))
                    .style(style().height(24))
            },
        )
        .row_height(height)
        .overscan(0.0)
        .keep_focused(keep_focused)
        .scroll_to(target)
        .node_ref(list)
        .test_id("rows")
        .style(style().height(120));
        (Rows { target, list }, view)
    })
}

/// The row `name` is mounted and inside the viewport of the list.
fn row_in_view(screen: &kivact::test::Screen, role: Role, name: &str) -> bool {
    let (Some(row), Some(list)) = (
        screen.query_by_role(role, name).and_then(|row| row.rect()),
        screen.get_by_test_id("rows").rect(),
    ) else {
        return false;
    };
    row.y >= list.y - 0.5 && row.y + row.height <= list.y + list.height + 0.5
}

fn button_in_view(screen: &kivact::test::Screen, n: u32) -> bool {
    row_in_view(screen, Role::Button, &format!("row {n}"))
}

#[test]
fn scroll_to_brings_a_row_that_is_not_mounted_into_view() {
    let (screen, h) = rows_list(2000, RowHeight::Fixed(24.0), true);
    assert!(screen.query_by_role(Role::Button, "row 1500").is_none());
    h.target.set(Some(1500));
    screen.run_until_idle();
    assert!(button_in_view(&screen, 1500));
    assert!(screen.query_by_role(Role::Button, "row 0").is_none());
    // A row above is brought to the top by the smallest move.
    h.target.set(Some(1490));
    screen.run_until_idle();
    assert!(button_in_view(&screen, 1490));
    let top = screen
        .get_by_role(Role::Button, "row 1490")
        .rect()
        .unwrap()
        .y;
    assert_eq!(top, screen.get_by_test_id("rows").rect().unwrap().y);
    // A row already in view does not move the list.
    let before = h.list.scroll_offset().get_untracked();
    h.target.set(Some(1492));
    screen.run_until_idle();
    assert_eq!(h.list.scroll_offset().get_untracked(), before);
    // The same key again scrolls again after a `None`.
    h.target.set(None);
    screen.get_by_test_id("rows").scroll(0.0, -100_000.0);
    assert!(screen.query_by_role(Role::Button, "row 1490").is_none());
    h.target.set(Some(1490));
    screen.run_until_idle();
    assert!(button_in_view(&screen, 1490));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn scroll_to_reaches_rows_with_measured_heights() {
    let (screen, h) = rows_list(400, RowHeight::Measured { estimate: 40.0 }, true);
    h.target.set(Some(300));
    screen.run_until_idle();
    assert!(button_in_view(&screen, 300));
    h.target.set(Some(5));
    screen.run_until_idle();
    assert!(button_in_view(&screen, 5));
    h.target.set(Some(399));
    screen.run_until_idle();
    assert!(button_in_view(&screen, 399));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn scroll_to_a_key_that_is_not_in_the_list_does_nothing() {
    let (screen, h) = rows_list(100, RowHeight::Fixed(24.0), true);
    h.target.set(Some(5000));
    screen.run_until_idle();
    assert_eq!(h.list.scroll_offset().get_untracked().y, 0.0);
    assert!(button_in_view(&screen, 0));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_focused_row_stays_mounted_outside_the_viewport_until_the_focus_leaves() {
    let (screen, h) = rows_list(500, RowHeight::Fixed(24.0), true);
    screen.get_by_role(Role::Button, "row 2").focus();
    assert!(screen.get_by_role(Role::Button, "row 2").is_focused());
    h.target.set(Some(300));
    screen.run_until_idle();
    assert!(button_in_view(&screen, 300));
    let row = screen.get_by_role(Role::Button, "row 2");
    assert!(row.exists());
    assert!(row.is_focused(), "the focus is not lost");
    screen.get_by_test_id("rows").scroll(0.0, 600.0);
    assert!(screen.get_by_role(Role::Button, "row 2").is_focused());
    // Moving the focus to a row in view lets go of the old one.
    let near = (0..500)
        .find(|n| button_in_view(&screen, *n))
        .expect("a row in view");
    screen
        .get_by_role(Role::Button, &format!("row {near}"))
        .focus();
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Button, "row 2").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn without_keep_focused_a_row_that_leaves_is_unmounted() {
    let (screen, h) = rows_list(500, RowHeight::Fixed(24.0), false);
    screen.get_by_role(Role::Button, "row 2").focus();
    h.target.set(Some(300));
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Button, "row 2").is_none());
}

/// A virtual list under a `List` that scrolls through
/// `scroll_to`.
#[test]
fn a_list_navigation_in_a_virtual_list_reaches_the_last_row_and_keeps_the_focus() {
    use kivact::interaction::{ListSelection, use_list};
    use kivact::test::{RenderOptions, WindowMode};
    use kivact::ui::ListItem;
    use kivact::view::Key;

    let (screen, nav) = crate::support::mount_with(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::from_env()
        },
        || {
            let keys: Vec<u32> = (0..300).collect();
            let rows = signal(keys.clone());
            let selected = signal(Vec::<u32>::new());
            let target = signal(None::<u32>);
            let nav = use_list(
                move || {
                    rows.get()
                        .into_iter()
                        .map(|key| ListItem::new(key, format!("row {key}")))
                        .collect::<Vec<_>>()
                },
                ListSelection::Multiple(selected.into()),
            )
            .page_size(5)
            .scroll_with(move |key| target.set(Some(key)));
            let list = virtual_list(
                keys,
                |n| *n,
                move |k, _| {
                    let name = format!("row {k}");
                    (node()
                        .role(Role::Option)
                        .name(name.clone())
                        .child(text(name))
                        .style(style().height(24)))
                    .part(nav.item(k).role(Role::Option))
                },
            )
            .row_height(RowHeight::Fixed(24.0))
            .overscan(0.0)
            .scroll_to(target)
            .test_id("rows")
            .style(style().height(120));
            (nav, list.role(Role::ListBox))
        },
    );
    screen.get_by_role(Role::Option, "row 0").click();
    assert!(screen.get_by_role(Role::Option, "row 0").is_focused());
    screen.press_key(Key::End);
    screen.run_until_idle();
    assert_eq!(nav.active().get_untracked(), Some(299));
    assert!(
        screen.get_by_role(Role::Option, "row 299").is_focused(),
        "the last row has the focus"
    );
    assert!(row_in_view(&screen, Role::Option, "row 299"));
    screen.press_key(Key::Home);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Option, "row 0").is_focused());
    assert!(row_in_view(&screen, Role::Option, "row 0"));
    for _ in 0..30 {
        screen.press_key(Key::PageDown);
    }
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Option, "row 150").is_focused());
    assert!(row_in_view(&screen, Role::Option, "row 150"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Reorder, removal and remount all use the actual engine's sibling order.
#[test]
fn keyed_repeated_reorders_and_retain_keep_focus_state_and_remount_cleanly() {
    let disposed = Rc::new(Cell::new(0));
    let counted = disposed.clone();
    let (screen, rows) = mount(move || {
        let rows = signal(vec![1u32, 2, 3, 4]);
        let view = column(for_each(
            rows,
            |id| *id,
            move |id, _| {
                let checked = signal(false);
                let counted = counted.clone();
                on_cleanup(move || counted.set(counted.get() + 1));
                node()
                    .role(Role::CheckBox)
                    .name(format!("sibling {id}"))
                    .focusable(true)
                    .checked(checked)
                    .style(style().size(120, 24))
                    .on_press(move || checked.update(|value| *value = !*value))
            },
        ));
        (rows, view)
    });
    let order = |ids: &[u32]| {
        let positions: Vec<_> = ids
            .iter()
            .map(|id| {
                screen
                    .get_by_role(Role::CheckBox, &format!("sibling {id}"))
                    .rect()
                    .expect("laid out")
                    .y
            })
            .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    };
    screen.get_by_role(Role::CheckBox, "sibling 3").toggle();
    screen.get_by_role(Role::CheckBox, "sibling 2").focus();
    let before = screen.stats();
    rows.set(vec![4, 3, 2, 1]);
    screen.run_until_idle();
    assert_eq!(
        screen.stats().ops - before.ops,
        3,
        "reversal keeps minimal moves"
    );
    order(&[4, 3, 2, 1]);
    assert!(screen.get_by_role(Role::CheckBox, "sibling 2").is_focused());
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "sibling 3").is_checked(),
        Some(true)
    );

    rows.set(vec![4, 3, 1, 5]);
    screen.run_until_idle();
    order(&[4, 3, 1, 5]);
    assert!(screen.query_by_role(Role::CheckBox, "sibling 2").is_none());
    assert!(screen.get_by_role(Role::CheckBox, "sibling 1").is_focused());
    for ids in [vec![5, 1, 3, 4], vec![4, 3, 1, 5], vec![5, 3, 4, 1]] {
        rows.set(ids.clone());
        screen.run_until_idle();
        order(&ids);
        assert_eq!(
            screen.get_by_role(Role::CheckBox, "sibling 3").is_checked(),
            Some(true)
        );
    }
    rows.set(Vec::new());
    screen.run_until_idle();
    assert_eq!(disposed.get(), 5, "each removed row owner is disposed once");
    assert!(screen.query_by_role(Role::CheckBox, "sibling 3").is_none());
    rows.set(vec![9, 8, 7, 3]);
    screen.run_until_idle();
    order(&[9, 8, 7, 3]);
    assert_eq!(
        screen.get_by_role(Role::CheckBox, "sibling 3").is_checked(),
        Some(false)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
