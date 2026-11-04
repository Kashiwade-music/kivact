//! Role: Tests for reactive setup functions and utilities.
//!
//! The free setup functions (`signal`, `memo`, `effect`, `effect_on`,
//! `on_cleanup`, `on_mount`, `context`, `context`) and the associated
//! constructors of the reactive handles, called with no setup-context argument.

use kivact::state::{callback, selector};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::state::effect_on;
use kivact::prelude::*;
use kivact::test::render;
use kivact::view::context;

type Log = Rc<RefCell<Vec<String>>>;

fn log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

fn take(l: &Log) -> Vec<String> {
    std::mem::take(&mut *l.borrow_mut())
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
