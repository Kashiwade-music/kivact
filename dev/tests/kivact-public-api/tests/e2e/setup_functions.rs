//! Role: Tests for reactive setup functions and utilities.
//!
//! The free setup functions (`signal`, `memo`, `effect`, `effect_on`,
//! `on_cleanup`, `on_mount`, `context`, `context`) and the associated
//! constructors of the reactive handles, called with no setup-context argument.

use kivact::view::{View, callback, selector};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::diagnostics::ErrorKind;
use kivact::effect_on;
use kivact::platform;
use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact::view::context;

type Log = Rc<RefCell<Vec<String>>>;

fn log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

fn take(l: &Log) -> Vec<String> {
    std::mem::take(&mut *l.borrow_mut())
}

/// The errors of `kind` recorded so far whose location is in this file.
fn located(screen: &Screen, api: &str) -> usize {
    screen
        .errors()
        .iter()
        .filter(|e| matches!(e.kind(), ErrorKind::SetupOnly { api: a } if *a == api))
        .filter(|e| {
            e.location()
                .is_some_and(|l| l.file().ends_with("setup_functions.rs"))
        })
        .count()
}

/// A plain-function component: state from the free setup functions.
fn counter(initial: i32, log: Log) -> impl IntoView {
    let count = signal(initial);
    let doubled = memo(move || count.get() * 2);
    effect(move || log.borrow_mut().push(format!("doubled {}", doubled.get())));
    column((
        text(move || count.get().to_string()).test_id("count"),
        {
            let label = ("+1").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .on_press(move || count.update(|c| *c += 1)),
    ))
}

#[test]
fn plain_function_components_use_free_setup_functions() {
    let l = log();
    let l2 = l.clone();
    let screen = render(move || counter(2, l2));
    assert_eq!(screen.get_by_test_id("count").text(), "2");
    assert_eq!(take(&l), vec!["doubled 4"]);
    screen.get_by_role(Role::Button, "+1").click();
    assert_eq!(screen.get_by_test_id("count").text(), "3");
    assert_eq!(take(&l), vec!["doubled 6"]);
    assert!(screen.errors().is_empty());
}

#[test]
fn signal_in_a_handler_is_a_located_setup_only_error() {
    let (screen, ()) = mount(|| {
        (
            (),
            {
                let label = ("late").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(|| {
                let _ = signal(5);
            }),
        )
    });
    screen.get_by_role(Role::Button, "late").click();
    assert_eq!(located(&screen, "signal"), 1);
}

#[test]
fn creation_outside_every_app_is_setup_only() {
    let r = std::panic::catch_unwind(|| signal(1));
    let msg = match r {
        Ok(_) => String::from("no panic"),
        Err(p) => p
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_else(|| String::from("non-string panic")),
    };
    assert!(
        msg.contains("`signal` can only be called during component setup"),
        "{msg}"
    );
}

#[test]
fn watch_skips_the_first_value_and_runs_untracked() {
    let l = log();
    let l2 = l.clone();
    let (screen, (source, other)) = mount(move || {
        let source = signal(1);
        let other = signal(10);
        effect_on(source, move |v| {
            // `other` is read inside `f`: not a dependency.
            l2.borrow_mut().push(format!("{v} {}", other.get()));
        });
        ((source, other), ())
    });
    screen.run_until_idle();
    assert!(take(&l).is_empty(), "the initial value is skipped");
    source.set(2);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["2 10"]);
    other.set(11);
    screen.run_until_idle();
    assert!(take(&l).is_empty(), "reads inside f are untracked");
    source.set(3);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["3 11"]);
    source.set(3);
    screen.run_until_idle();
    assert!(take(&l).is_empty(), "an equal write does not notify");
}

#[test]
fn watch_follows_derived_sources_and_may_write() {
    let (screen, (a, mirror)) = mount(|| {
        let a = signal(1);
        let mirror = signal(0);
        effect_on(move || a.get() * 10, move |v| mirror.set(*v));
        (
            (a, mirror),
            text(move || mirror.get().to_string()).test_id("m"),
        )
    });
    assert_eq!(mirror.get_untracked(), 0);
    a.set(4);
    screen.run_until_idle();
    assert_eq!(screen.get_by_test_id("m").text(), "40");
    assert!(screen.errors().is_empty());
}

#[test]
fn watch_delivers_the_first_change_after_a_failed_initial_run() {
    let l = log();
    let l2 = l.clone();
    let (screen, a) = mount(move || {
        let a = signal(0);
        effect_on(
            move || {
                let v = a.get();
                assert!(v != 0, "initial source fails");
                v
            },
            move |v| l2.borrow_mut().push(v.to_string()),
        );
        (a, ())
    });
    screen.run_until_idle();
    assert_eq!(
        screen.errors().len(),
        1,
        "the failed initial run is reported"
    );
    assert!(take(&l).is_empty());
    a.set(1);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["1"], "the first real change is delivered");
}

#[test]
fn lifecycle_hooks_register_on_the_current_owner() {
    let l = log();
    let l2 = l.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let l3 = l2.clone();
        let view = show(open, move || {
            let (a, b) = (l3.clone(), l3.clone());
            on_mount(move || a.borrow_mut().push("mounted".into()));
            on_cleanup(move || b.borrow_mut().push("cleaned".into()));
            text("child")
        });
        (open, view)
    });
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["mounted"]);
    open.set(false);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["cleaned"]);
}

#[test]
fn on_cleanup_inside_an_effect_runs_before_each_rerun() {
    let l = log();
    let l2 = l.clone();
    let (screen, s) = mount(move || {
        let s = signal(0);
        effect(move || {
            let v = s.get();
            let l3 = l2.clone();
            on_cleanup(move || l3.borrow_mut().push(format!("cleanup {v}")));
        });
        (s, ())
    });
    screen.run_until_idle();
    s.set(1);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["cleanup 0"]);
}

#[derive(Clone, Copy)]
struct Theme2(u32);

#[test]
fn context_is_found_in_setup_and_in_handlers() {
    let seen = Rc::new(Cell::new(0));
    let s2 = seen.clone();
    let screen = render(move || {
        provide(Theme2(7), move || {
            let in_setup = context::<Theme2>().0;
            assert_eq!(
                kivact::view::try_context::<Theme2>().map(|t| t.0),
                Some(in_setup)
            );
            let missing = kivact::view::try_context::<u8>();
            assert!(missing.is_none());
            {
                let label = (format!("ctx {in_setup}")).into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || s2.set(context::<Theme2>().0))
        })
    });
    screen.get_by_role(Role::Button, "ctx 7").click();
    assert_eq!(seen.get(), 7, "the handler runs for its node's owner");
}

#[test]
fn expecting_a_missing_context_is_a_located_panic() {
    let context_line = line!() + 2;
    let screen = render(|| {
        let _ = context::<Theme2>();
        text("unreachable")
    });
    let e = screen.errors();
    assert!(e.iter().any(|e| {
        matches!(e.kind(), ErrorKind::Panic)
            && e.to_string().contains(std::any::type_name::<Theme2>())
            && e.location().is_some_and(|l| {
                l.file().ends_with("setup_functions.rs") && l.line() == context_line
            })
    }));
}

#[test]
fn handle_constructors_create_under_the_setup_owner() {
    let l = log();
    let l2 = l.clone();
    let (screen, (selected, var, trigger, cb)) = mount(move || {
        let selected = signal(None::<u32>);
        let sel = selector(selected);
        let var = var(0u32);
        let trigger = signal(());
        let l3 = l2.clone();
        // No annotation: `E` comes from the use below.
        let cb = callback(move |v| l3.borrow_mut().push(format!("cb {v}")));
        let l4 = l2.clone();
        effect(move || {
            trigger.get();
            l4.borrow_mut().push(format!("is 1: {}", sel.is(&1)));
        });
        cb.call(1u32);
        ((selected, var, trigger, cb), ())
    });
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["cb 1", "is 1: false"]);
    selected.set(Some(1));
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["is 1: true"]);
    trigger.notify();
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["is 1: true"]);
    var.set(3);
    assert_eq!(var.get(), 3);
    cb.call(9);
    assert_eq!(take(&l), vec!["cb 9"]);
}

#[test]
fn constructors_outside_setup_are_located_errors() {
    let (screen, ()) = mount(|| {
        (
            (),
            {
                let label = ("late").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(|| {
                let _ = var(1);
            }),
        )
    });
    screen.get_by_role(Role::Button, "late").click();
    assert_eq!(located(&screen, "var"), 1);
}

#[test]
fn into_value_in_a_handler_is_a_located_setup_only_error() {
    let (screen, ()) = mount(|| {
        (
            (),
            {
                let label = ("late").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(|| {
                let _: Value<String> = "label".into_value();
            }),
        )
    });
    screen.get_by_role(Role::Button, "late").click();
    assert_eq!(located(&screen, "IntoValue::into_value"), 1);
}

#[test]
fn into_value_keeps_handles_memoizes_closures_and_freezes_constants() {
    type Read = (Signal<i32>, Value<i32>, Value<i32>, Value<String>);
    let (screen, (s, same, doubled, label)): (Screen, Read) = mount(|| {
        let s = signal(1);
        let same = s.into_value();
        let doubled = (move || s.get() * 2).into_value();
        let label = "label".into_value();
        ((s, same, doubled, label), text(label))
    });
    assert_eq!(screen.get_by_role(Role::Text, "label").text(), "label");
    s.set(3);
    assert_eq!(same.get_untracked(), 3);
    assert_eq!(doubled.get_untracked(), 6);
    assert_eq!(label.get_untracked(), "label");

    // A handle passes through: no new node. A closure is one memo, a
    // constant no node (a `Const` value).
    let nodes = |f: fn()| {
        let screen = render(move || {
            f();
            View::empty()
        });
        screen.stats().live_reactive
    };
    let base = nodes(|| {
        let _ = signal(0);
    });
    let handle = nodes(|| {
        let _ = signal(0).into_value();
    });
    let closure = nodes(|| {
        let s = signal(0);
        let _ = (move || s.get()).into_value();
    });
    let constant = nodes(|| {
        let _ = signal(0);
        let _: Value<i32> = 5.into_value();
    });
    assert_eq!((handle, closure, constant), (base, base + 1, base));
}

fn app() -> impl IntoView {
    let count = signal(0);
    {
        let label = (move || format!("count {}", count.get())).into_value();
        node()
            .role(Role::Button)
            .focusable(true)
            .name(label)
            .child(text(label))
    }
    .on_press(move || count.update(|c| *c += 1))
}

#[test]
fn render_takes_a_plain_root_function() {
    let screen = kivact::test::render(app);
    screen.get_by_role(Role::Button, "count 0").click();
    screen.get_by_role(Role::Button, "count 1");
    let options = kivact::test::RenderOptions {
        reduced_motion: true,
        ..Default::default()
    };
    let screen = kivact::test::render_with(options, || {
        let reduced = platform::reduced_motion();
        text(format!("reduced {reduced}"))
    });
    screen.get_by_text("reduced true");
}

#[test]
fn a_root_setup_panic_is_reported_and_mounts_nothing() {
    let screen = kivact::test::render(|| -> kivact::control::Node { panic!("root broke") });
    assert!(
        screen
            .errors()
            .iter()
            .any(|e| *e.kind() == ErrorKind::Panic && e.to_string().contains("root broke"))
    );
}

#[derive(Clone, PartialEq)]
struct Row {
    id: u32,
    done: bool,
}

#[test]
fn a_keyed_item_field_writes_back_and_keeps_its_last_value_once_gone() {
    let (screen, (rows, done)) = mount(|| {
        let rows = signal(vec![Row { id: 1, done: false }, Row { id: 2, done: false }]);
        let done = rows
            .item(|row| row.id, 2)
            .field(|row| &row.done, |row| &mut row.done);
        ((rows, done), checkbox("Second", done))
    });
    screen.get_by_role(Role::CheckBox, "Second").click();
    assert!(rows.with_untracked(|rows| rows[1].done && !rows[0].done));
    assert_eq!(
        rows.update(|rows| rows.len()),
        2,
        "update returns the closure's value"
    );

    rows.update(|rows| rows.retain(|row| row.id != 2));
    screen.run_until_idle();
    assert!(done.get_untracked(), "a removed item reads its last value");
    done.set(false);
    screen.run_until_idle();
    assert_eq!(
        rows.with_untracked(Vec::len),
        1,
        "writing a removed item does nothing"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
