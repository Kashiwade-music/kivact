//! Role: Retained tree, mount, flows, lists and the minimal test harness.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::binding;
use kivact::diagnostics::ErrorKind;
use kivact::draw::Size;
use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact::view::{Handled, KeyEvent, RowHeight, callback, error_boundary};

type Log = Rc<RefCell<Vec<String>>>;

fn log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

fn take(l: &Log) -> Vec<String> {
    std::mem::take(&mut *l.borrow_mut())
}

fn texts(screen: &Screen) -> Vec<String> {
    fn walk(n: &kivact::test::SemanticNode, out: &mut Vec<String>) {
        if n.role == Some(Role::Text) {
            out.push(n.name.clone());
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    let mut v = Vec::new();
    walk(&screen.snapshot(), &mut v);
    v
}

#[test]
fn show_keeps_owner_while_true_and_disposes_on_false() {
    let l = log();
    let l2 = l.clone();
    let (screen, (open, tick)) = mount(move || {
        let open = signal(true);
        let tick = signal(0);
        let l3 = l2.clone();
        (
            (open, tick),
            show(
                move || open.get() && tick.get() >= 0,
                move || {
                    let count = var(0);
                    let l4 = l3.clone();
                    l3.borrow_mut().push("mount".into());
                    on_cleanup(move || l4.borrow_mut().push("dispose".into()));
                    text(move || {
                        count.update(|c| *c += 1);
                        format!("tick {}", tick.get())
                    })
                },
            )
            .fallback(|| text("closed")),
        )
    });
    assert_eq!(take(&l), vec!["mount"]);
    tick.set(1); // condition stays true: owner kept, only the text changes
    screen.run_until_idle();
    assert!(take(&l).is_empty());
    screen.get_by_text("tick 1");
    open.set(false);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["dispose"]);
    screen.get_by_text("closed");
    assert!(screen.query_by_text("tick 1").is_none());
    open.set(true);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["mount"]);
}

#[test]
fn show_some_updates_value_without_remount() {
    let l = log();
    let l2 = l.clone();
    let (screen, sel) = mount(move || {
        let sel = signal(Some(1u32));
        let l3 = l2.clone();
        (
            sel,
            show_some(sel, move |v: Value<u32>| {
                l3.borrow_mut().push("mount".into());
                text(move || format!("item {}", v.get()))
            })
            .fallback(|| text("none")),
        )
    });
    sel.set(Some(2));
    screen.run_until_idle();
    screen.get_by_text("item 2");
    assert_eq!(take(&l), vec!["mount"], "Some→Some keeps the owner");
    sel.set(None);
    screen.run_until_idle();
    screen.get_by_text("none");
}

#[test]
fn dynamic_over_a_memoed_arm_picks_the_first_true_case_and_dynamic_rebuilds() {
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
        (
            n,
            column((
                dynamic(arm, |arm| match arm {
                    2 => text("big"),
                    1 => text("positive"),
                    _ => text("zero"),
                }),
                dynamic(
                    move || n.get() % 2 == 0,
                    |even| {
                        if even { text("even") } else { text("odd") }
                    },
                ),
            )),
        )
    });
    assert_eq!(texts(&screen), vec!["zero", "even"]);
    n.set(3);
    screen.run_until_idle();
    assert_eq!(texts(&screen), vec!["positive", "odd"]);
    n.set(12);
    screen.run_until_idle();
    assert_eq!(texts(&screen), vec!["big", "even"]);
}

#[test]
fn error_boundary_shows_fallback_and_reset_remounts() {
    fn fragile(fail: Signal<bool>) -> impl IntoView {
        if fail.get_untracked() {
            panic!("fragile failed");
        }
        text("healthy")
    }
    let (screen, fail) = mount(|| {
        let fail = signal(true);
        (
            fail,
            error_boundary(move || fragile(fail)).fallback(|err, reset| {
                column((
                    text(format!("error: {err}")),
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
            }),
        )
    });
    screen.get_by_text("error: panicked: fragile failed");
    assert!(screen.errors().is_empty(), "handled by the boundary");
    fail.set(false);
    screen.get_by_role(Role::Button, "retry").click();
    screen.get_by_text("healthy");
}

#[derive(Clone, Debug, PartialEq)]
struct Item {
    id: u32,
    label: String,
}

fn items(ids: &[u32]) -> Vec<Item> {
    ids.iter()
        .map(|&id| Item {
            id,
            label: format!("item {id}"),
        })
        .collect()
}

#[test]
fn for_each_keeps_row_owners_uses_minimal_moves_and_reports_duplicates() {
    let l = log();
    let l2 = l.clone();
    let (screen, list) = mount(move || {
        let list = signal(items(&[1, 2, 3, 4, 5]));
        let l3 = l2.clone();
        (
            list,
            column(for_each(
                list,
                |it| it.id,
                move |id, it| {
                    let l4 = l3.clone();
                    l4.borrow_mut().push(format!("mount {id}"));
                    on_cleanup(move || l4.borrow_mut().push(format!("dispose {id}")));
                    text(move || it.get().label)
                },
            )),
        )
    });
    assert_eq!(take(&l).len(), 5);
    // update one item: no remount, only its text changes
    list.update(|v| v[2].label = "three".into());
    screen.run_until_idle();
    assert!(take(&l).is_empty());
    assert_eq!(
        texts(&screen),
        vec!["item 1", "item 2", "three", "item 4", "item 5"]
    );
    // move the last to the front: exactly one Move op
    let before = screen.stats();
    list.update(|v| {
        let last = v.pop().expect("non-empty");
        v.insert(0, last);
    });
    screen.run_until_idle();
    let after = screen.stats();
    assert_eq!(after.structural - before.structural, 1, "one Move");
    assert_eq!(
        texts(&screen),
        vec!["item 5", "item 1", "item 2", "three", "item 4"]
    );
    assert!(take(&l).is_empty());
    // remove + insert
    list.set(items(&[5, 7, 2]));
    screen.run_until_idle();
    let mut got = take(&l);
    got.sort();
    assert_eq!(got, vec!["dispose 1", "dispose 3", "dispose 4", "mount 7"]);
    assert_eq!(texts(&screen), vec!["item 5", "item 7", "item 2"]);
    // duplicate keys: later item dropped, diagnostic
    list.set(items(&[5, 5, 2]));
    screen.run_until_idle();
    assert_eq!(texts(&screen), vec!["item 5", "item 2"]);
    assert!(
        screen
            .errors()
            .iter()
            .any(|e| matches!(e.kind(), ErrorKind::DuplicateKey { .. }))
    );
    list.set(Vec::new());
    screen.run_until_idle();
    assert!(texts(&screen).is_empty());
}

#[test]
fn index_each_follows_indices() {
    let (screen, list) = mount(|| {
        let list = signal(vec![10, 20, 30]);
        (
            list,
            column(for_index(list, |i, v| {
                text(move || format!("{i}:{}", v.get()))
            })),
        )
    });
    assert_eq!(texts(&screen), vec!["0:10", "1:20", "2:30"]);
    list.set(vec![11, 20]);
    screen.run_until_idle();
    assert_eq!(texts(&screen), vec!["0:11", "1:20"]);
    list.set(vec![11, 20, 5, 6]);
    screen.run_until_idle();
    assert_eq!(texts(&screen), vec!["0:11", "1:20", "2:5", "3:6"]);
}

#[test]
fn virtual_list_mounts_only_the_visible_window() {
    let (screen, data) = mount(|| {
        let data = signal(items(&(0..10_000).collect::<Vec<_>>()));
        (
            data,
            virtual_list(data, |it| it.id, |_, it| text(move || it.get().label))
                .row_height(RowHeight::Fixed(20.0))
                .style(style().height(400))
                .test_id("list"),
        )
    });
    let visible = texts(&screen);
    assert!(
        visible.len() <= 60,
        "only a window is mounted: {}",
        visible.len()
    );
    assert_eq!(visible.first().map(String::as_str), Some("item 0"));
    screen.get_by_test_id("list").scroll(0.0, 20_000.0);
    let visible = texts(&screen);
    assert!(visible.iter().any(|t| t == "item 1000"), "{visible:?}");
    assert!(!visible.iter().any(|t| t == "item 0"));
    data.update(|v| v[1000].label = "changed".into());
    screen.run_until_idle();
    assert!(texts(&screen).iter().any(|t| t == "changed"));
}

#[test]
fn text_field_binding_is_two_way_without_echo() {
    let (screen, name) = mount(|| {
        let name = signal(String::from("a"));
        (
            name,
            column((
                kivact::control::node()
                    .name("Name")
                    .part(kivact::interaction::use_text_edit(name).control()),
                text(move || format!("hello {}", name.get())),
            )),
        )
    });
    screen.get_by_role(Role::TextBox, "Name").type_text("bc");
    assert_eq!(name.get_untracked(), "abc");
    screen.get_by_text("hello abc");
    let before = screen.stats();
    // typing again produced exactly one text-changed round trip without an echo op
    screen.get_by_role(Role::TextBox, "Name").type_text("d");
    let after = screen.stats();
    assert_eq!(
        after.set_props - before.set_props,
        1,
        "only the greeting text changed"
    );
    name.set("zz".into());
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Name").value().as_deref(),
        Some("zz")
    );
    // IME composition does not reach the binding until commit
    let tb = screen.get_by_role(Role::TextBox, "Name");
    tb.compose("にほ");
    assert_eq!(name.get_untracked(), "zz");
    tb.type_text("日本");
    assert_eq!(name.get_untracked(), "zz日本");
}

#[test]
fn node_ref_focus() {
    let (screen, (target, show_it)) = mount(move || {
        let target = node_ref();
        let show_it = signal(true);
        (
            (target, show_it),
            column((
                show(show_it, move || {
                    {
                        let label = ("target").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    }
                    .node_ref(target)
                }),
                {
                    let label = ("focus it").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || target.focus()),
            )),
        )
    });
    let _ = target;
    screen.get_by_role(Role::Button, "focus it").click();
    assert!(screen.get_by_role(Role::Button, "target").is_focused());
    show_it.set(false);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "focus it").click();
    assert!(
        screen
            .errors()
            .iter()
            .any(|e| *e.kind() == ErrorKind::Platform)
    );
}

#[test]
fn test_ids_are_unscoped_and_ambiguity_reports_located_candidates() {
    const ROW_LINE: u32 = line!() + 2;
    fn row_c(n: u32) -> impl IntoView {
        text(format!("row {n}")).test_id("row")
    }
    let (screen, ()) = mount(|| ((), column((row_c(1), row_c(2), text("x").test_id("solo")))));
    assert_eq!(screen.all_by_test_id("row").len(), 2);
    screen.get_by_test_id("solo");
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        screen.get_by_test_id("row");
    }))
    .expect_err("ambiguous");
    let msg = err.downcast_ref::<String>().cloned().unwrap_or_default();
    assert!(
        msg.contains("ambiguous") && msg.contains("candidates: 2"),
        "{msg}"
    );
    // Both candidates are named by where `row_c` created them: the same
    // location, shown for each of them in the report and the dump.
    let at = format!("view.rs:{ROW_LINE}:9");
    let candidates: Vec<&str> = msg
        .lines()
        .filter(|l| l.starts_with("  node ") && l.contains("\"row "))
        .collect();
    assert_eq!(candidates.len(), 2, "{msg}");
    assert!(candidates.iter().all(|l| l.ends_with(&at)), "{msg}");
    let dump = screen.debug();
    assert_eq!(dump.matches("#row @").count(), 2, "{dump}");
    let column = &screen.snapshot().children[0];
    for row in &column.children[..2] {
        let row_at = row.at.expect("a text node records its builder's call");
        assert!(row_at.file().ends_with("view.rs"), "{row_at}");
        assert_eq!((row_at.line(), row_at.column()), (ROW_LINE, 9));
    }
    // `within` tells the two instances apart.
    let first = screen.get_by_text("row 1");
    assert_eq!(screen.within(first).get_by_test_id("row").text(), "row 1");
}

#[test]
fn keyboard_activation_disabled_and_bubbling() {
    let l = log();
    let (l2, l3) = (l.clone(), l.clone());
    let (screen, enabled) = mount(move || {
        let enabled = signal(false);
        let (la, lb) = (l2.clone(), l3.clone());
        (
            enabled,
            node()
                .on_key(move |k: KeyEvent| {
                    la.borrow_mut().push(format!("parent {:?}", k.key));
                    Handled::No
                })
                .child(
                    {
                        let label = ("go").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    }
                    .disabled(move || !enabled.get())
                    .on_press(move || lb.borrow_mut().push("pressed".into())),
                ),
        )
    });
    let go = screen.get_by_role(Role::Button, "go");
    // A press on the disabled button (a checked `click` would refuse it).
    go.force_click();
    assert!(take(&l).is_empty(), "disabled button ignores clicks");
    enabled.set(true);
    screen.run_until_idle();
    go.press_key(Key::Enter);
    go.press_key(Key::Space);
    go.press_key(Key::Char('x'));
    assert_eq!(
        take(&l),
        vec![
            "parent Enter",
            "pressed",
            "parent Space",
            "pressed",
            "parent Char('x')"
        ]
    );
}

#[test]
fn on_mount_runs_after_the_host_acknowledges() {
    let order = log();
    let o2 = order.clone();
    let (screen, ()) = mount(move || {
        let o3 = o2.clone();
        on_mount(move || o3.borrow_mut().push("mounted".into()));
        o2.borrow_mut().push("setup".into());
        ((), text("hi"))
    });
    let _ = screen;
    assert_eq!(take(&order), vec!["setup", "mounted"]);
}

/// A post-layout callback changes geometry without another input action.
/// This covers mount writes, but does not isolate host control from waker/redraw.
#[test]
fn on_mount_layout_writes_settle_without_another_input() {
    use kivact::primitive::{Constraints, primitive};
    let before = Rc::new(Cell::new(None));
    let observed = before.clone();
    let (screen, ()) = mount(move || {
        let width = signal(40.0_f32);
        let measured = Rc::new(Cell::new(None));
        let geometry = measured.clone();
        on_mount(move || {
            observed.set(measured.get());
            width.set(160.0);
        });
        (
            (),
            primitive()
                .measure((), move |_: &(), constraints: Constraints| {
                    let size = Size::new(constraints.max_width().unwrap_or(0.0), 24.0);
                    geometry.set(Some(size));
                    size
                })
                .test_id("mount resized")
                .style(move || style().width(width.get()).height(24)),
        )
    });
    assert_eq!(before.get().expect("mount sees initial layout").width, 40.0);
    assert_eq!(
        screen.get_by_test_id("mount resized").rect().unwrap().width,
        160.0
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn mount_callback_work_is_included_in_core_timing() {
    let did_run = Rc::new(Cell::new(false));
    let mounted = did_run.clone();
    let screen = render(move || {
        on_mount(move || {
            std::thread::sleep(Duration::from_millis(15));
            mounted.set(true);
        });
        node()
    });
    assert!(did_run.get());
    assert!(screen.stats().core_ns >= 15_000_000);
}

#[test]
fn on_mount_reads_attached_list_geometry_after_commit_and_layout() {
    use kivact::primitive::{Constraints, primitive};

    let mounted = Rc::new(RefCell::new(Vec::new()));
    let observed = mounted.clone();
    let (screen, items) = mount(move || {
        let items = signal(Vec::<u32>::new());
        let rows = for_each(
            items,
            |id| *id,
            move |id, _| {
                let measured = Rc::new(Cell::new(None));
                let geometry = measured.clone();
                let observed = observed.clone();
                on_mount(move || observed.borrow_mut().push((id, measured.get())));
                primitive()
                    .measure((), move |_: &(), constraints: Constraints| {
                        let size = Size::new(constraints.max_width().unwrap_or(0.0), 20.0);
                        geometry.set(Some(size));
                        size
                    })
                    .test_id(format!("mounted-row-{id}"))
            },
        );
        (items, column(rows).style(style().width(120).padding(12)))
    });
    assert!(mounted.borrow().is_empty());
    items.set(vec![1, 2]);
    screen.run_until_idle();
    let observations = mounted.borrow();
    assert_eq!(observations.len(), 2, "each inserted row mounts once");
    for (index, (id, size)) in observations.iter().enumerate() {
        assert_eq!(*id, index as u32 + 1);
        assert_eq!(
            *size,
            Some(Size::new(96.0, 20.0)),
            "on_mount sees layout measured under the attached parent's constraints"
        );
        let rect = screen
            .get_by_test_id(&format!("mounted-row-{id}"))
            .rect()
            .expect("laid out");
        assert_eq!((rect.x, rect.y), (12.0, 12.0 + index as f32 * 20.0));
        assert_eq!((rect.width, rect.height), (96.0, 20.0));
    }
    assert!(screen.errors().is_empty());
}

#[test]
fn stale_events_are_dropped() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            column((
                text("a"),
                show(open, || {
                    {
                        let label = ("b").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    }
                    .on_press(|| ())
                }),
            )),
        )
    });
    let b = screen.get_by_role(Role::Button, "b");
    let at = b.rect().expect("laid out");
    open.set(false);
    screen.run_until_idle();
    assert!(!b.exists());
    // A click where the button was reaches nothing and breaks nothing.
    screen.click_at(at.x + at.width / 2.0, at.y + at.height / 2.0);
    assert!(screen.errors().is_empty());
}

#[test]
fn counter_example_end_to_end() {
    fn counter(initial: i32) -> impl IntoView {
        let count = signal(initial);
        let doubled = memo(move || count.get() * 2);
        column((
            text(move || format!("count: {} / doubled: {}", count.get(), doubled.get())),
            {
                let label = ("増やす").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || count.update(|c| *c += 1)),
        ))
        .style(style().gap(8).padding(12))
    }
    let screen = render(|| counter(0));
    screen.get_by_text("count: 0 / doubled: 0");
    let before = screen.stats();
    screen.get_by_role(Role::Button, "増やす").click();
    screen.get_by_text("count: 1 / doubled: 2");
    let after = screen.stats();
    assert_eq!(
        after.set_props - before.set_props,
        1,
        "one text prop changed"
    );
    let _ = Cell::new(0);
}

/// Regression: a `virtual_list` whose viewport
/// is as tall as its content mounts every row; that is reported in the
/// trace (and `tracing::warn!`) instead of happening silently.
#[test]
fn unconstrained_virtual_list_is_reported() {
    let note = |screen: &Screen| {
        screen
            .trace()
            .iter()
            .flat_map(|t| t.notes.clone())
            .find(|n| n.contains("virtual_list") && n.contains("bounded height"))
    };
    let (screen, _) = mount(|| {
        let data = signal(items(&(0..300).collect::<Vec<_>>()));
        (
            (),
            column(
                virtual_list(data, |it| it.id, |_, it| text(move || it.get().label))
                    .row_height(RowHeight::Fixed(20.0))
                    .style(style().grow(1)),
            ),
        )
    });
    assert_eq!(texts(&screen).len(), 300, "all rows mounted");
    assert!(note(&screen).is_some(), "{:#?}", screen.trace());
    let (bounded, _) = mount(|| {
        let data = signal(items(&(0..300).collect::<Vec<_>>()));
        (
            (),
            virtual_list(data, |it| it.id, |_, it| text(move || it.get().label))
                .row_height(RowHeight::Fixed(20.0))
                .style(style().height(200)),
        )
    });
    assert!(texts(&bounded).len() < 50);
    assert!(note(&bounded).is_none());
}

// ---------------------------------------------------------------------------
// Regression (design survey): a rejected edit of a bound text field is not reverted on the host.

#[test]
fn rejected_text_edit_is_reverted() {
    let (screen, value) = mount(|| {
        let value = signal(String::from("abc"));
        // A validating binding: only digits are accepted.
        let set = callback(move |s: String| {
            if s.chars().all(|c| c.is_ascii_digit()) {
                value.set(s);
            }
        });
        let b = binding(value.into_value(), move |value| set.try_call(value));
        (
            value,
            kivact::control::node()
                .name("digits")
                .part(kivact::interaction::use_text_edit(b).control()),
        )
    });
    let field = screen.get_by_role(Role::TextBox, "digits");
    field.type_text("x");
    assert_eq!(value.get_untracked(), "abc");
    assert_eq!(
        field.value().as_deref(),
        Some("abc"),
        "the host keeps showing the rejected text"
    );
}

// ---------------------------------------------------------------------------
// Regression (design survey): NodeRef::rect keeps the last rect after the node is unmounted.

#[test]
fn node_ref_rect_becomes_none_when_the_node_is_unmounted() {
    let (screen, (open, rect)) = mount(|| {
        let open = signal(true);
        let r = node_ref();
        let rect = r.rect();
        let v = show(open, move || node().style(style().size(20, 20)).node_ref(r));
        ((open, rect), v)
    });
    assert!(rect.get_untracked().is_some());
    open.set(false);
    screen.run_until_idle();
    assert_eq!(rect.get_untracked(), None, "stale rect after unmount");
}

// ---------------------------------------------------------------------------
// Regression (design survey): virtual_list host child order does not follow item order.

#[test]
fn virtual_list_rows_follow_item_order_for_assistive_technology() {
    let (screen, items) = mount(|| {
        let items = signal((0..50u32).collect::<Vec<_>>());
        let v = virtual_list(items, |v| *v, |k, _| text(format!("row {k}")))
            .row_height(RowHeight::Fixed(20.0))
            .style(style().height(100));
        (items, v)
    });
    items.update(|v| v.swap(0, 1));
    screen.run_until_idle();
    let got = texts(&screen);
    assert_eq!(
        got.get(..2),
        Some(&["row 1".to_string(), "row 0".to_string()][..]),
        "reading / Tab order differs from the visual order: {got:?}"
    );
}

// ---------------------------------------------------------------------------
// Regression (design survey): on_mount of a component without host nodes waits for an unrelated batch.

#[test]
fn on_mount_runs_for_a_component_without_host_nodes() {
    fn no_host_nodes(mounted: Rc<Cell<bool>>) {
        on_mount(move || mounted.set(true));
    }
    let mounted = Rc::new(Cell::new(false));
    let m2 = mounted.clone();
    let (screen, open) = mount(move || {
        let open = signal(false);
        let v = show(open, move || no_host_nodes(m2.clone()));
        (open, v)
    });
    open.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    assert!(
        mounted.get(),
        "on_mount never ran (no batch carried the mount)"
    );
}
