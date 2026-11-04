//! Role: Tests for control flow builders and flow primitives.
//!
//! The flow builders (`show`, `show_some`, `case`,
//! `switch`, `for_each`, `for_index`, `virtual_list`, `dynamic`,
//! `error_boundary`) with plain-function components.

use kivact::primitive::NodeExt;

use crate::support::mount;
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

// ---------------------------------------------------------------------------
// Regression (design survey): errors raised inside an error_boundary's fallback are swallowed.

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

/// A virtual list under a `List` that scrolls through
/// `scroll_to`.
#[test]
fn a_list_navigation_in_a_virtual_list_reaches_the_last_row_and_keeps_the_focus() {
    use kivact::interaction::{ListPolicy, ListSelection, use_list};
    use kivact::test::{MirrorMode, RenderOptions};
    use kivact::ui::ListItem;
    use kivact::view::Key;

    let (screen, nav) = crate::support::mount_with(
        RenderOptions::from_env().mirror(MirrorMode::Required),
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
                ListPolicy::default(),
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
