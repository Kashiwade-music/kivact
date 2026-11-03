//! Role: Retained tree, mount, flows, lists and the minimal test harness.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::diagnostics::ErrorKind;
use kivact::interaction::announcer;
use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact::view::{Handled, KeyEvent, LiveRegion, RowHeight};

type Log = Rc<RefCell<Vec<String>>>;

#[test]
fn announcements_publish_in_separate_persistent_live_regions() {
    let screen = render(|| {
        let notices = announcer();
        column((
            button("Save").on_press(move || notices.announce("Saved", LiveRegion::Polite)),
            button("Retry").on_press(move || notices.announce("Failed", LiveRegion::Assertive)),
            column(notices.outlet()).test_id("notices"),
        ))
    });
    let outlet = screen.get_by_test_id("notices");
    let regions = outlet.children();
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0].live_region(), LiveRegion::Polite);
    assert_eq!(regions[1].live_region(), LiveRegion::Assertive);

    let save = screen.get_by_role(Role::Button, "Save");
    save.click();
    assert_eq!(regions[0].get_by_role(Role::Text, "Saved").text(), "Saved");
    save.click();
    assert_eq!(regions[0].get_by_role(Role::Text, "Saved").text(), "Saved");
    screen.get_by_role(Role::Button, "Retry").click();
    assert_eq!(
        regions[1].get_by_role(Role::Text, "Failed").text(),
        "Failed"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

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
fn node_ref_rect_follows_a_node_bound_again() {
    let (screen, shown) = mount(move || {
        let target = node_ref();
        let rect = target.rect();
        let shown = signal(true);
        (
            shown,
            column((
                show(shown, move || {
                    node().style(style().size(40, 30)).node_ref(target)
                }),
                text(move || match rect.get() {
                    Some(r) => format!("width {}", r.width),
                    None => "unbound".to_string(),
                }),
            )),
        )
    });
    screen.get_by_text("width 40");
    shown.set(false);
    screen.run_until_idle();
    screen.get_by_text("unbound");
    shown.set(true);
    screen.run_until_idle();
    screen.get_by_text("width 40");
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

// ---------------------------------------------------------------------------
// Regression (design survey): a rejected edit of a bound text field is not reverted on the host.

// ---------------------------------------------------------------------------
// Regression (design survey): NodeRef::rect keeps the last rect after the node is unmounted.

// ---------------------------------------------------------------------------
// Regression (design survey): virtual_list host child order does not follow item order.

// ---------------------------------------------------------------------------
// Regression (design survey): on_mount of a component without host nodes waits for an unrelated batch.
