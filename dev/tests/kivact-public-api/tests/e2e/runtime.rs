//! Role: Tests for reactive graph, turns, and handle lifetimes.
//!
//! Reactive graph, turn and handle lifetime: turn ordering, the handle part of
//! the lifecycle/leak check, and graph unit tests. Pure contexts are in
//! `purity.rs`; handles of another App or a disposed owner in
//! `handle_validity.rs`.

use kivact::state::callback;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::prelude::*;
use kivact::state::Binding;
use kivact::state::binding;

type Log = Rc<RefCell<Vec<String>>>;

fn log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

fn take(l: &Log) -> Vec<String> {
    std::mem::take(&mut *l.borrow_mut())
}

#[test]
fn memo_cutoff_stops_equal_values() {
    let runs = Rc::new(Cell::new(0));
    let r2 = runs.clone();
    let l = log();
    let l2 = l.clone();
    let (screen, s) = mount(move || {
        let s = signal(1);
        let parity = memo(move || {
            r2.set(r2.get() + 1);
            s.get() % 2
        });
        effect(move || l2.borrow_mut().push(format!("parity {}", parity.get())));
        (s, text(move || parity.get().to_string()))
    });
    assert_eq!(take(&l), vec!["parity 1"]);
    let before = runs.get();
    s.set(3); // parity unchanged
    screen.run_until_idle();
    assert_eq!(runs.get(), before + 1, "memo recomputed once");
    assert!(take(&l).is_empty(), "effect cut off by memo equality");
    s.set(3); // same value: nothing at all
    screen.run_until_idle();
    assert_eq!(runs.get(), before + 1);
    s.set(4);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["parity 0"]);
}

#[test]
fn diamond_is_glitch_free_and_runs_once() {
    let l = log();
    let l2 = l.clone();
    let (screen, a) = mount(move || {
        let a = signal(1);
        let b = memo(move || a.get() * 2);
        let c = memo(move || a.get() + 1);
        let d = memo(move || b.get() + c.get());
        effect(move || {
            l2.borrow_mut()
                .push(format!("{} {} {}", b.get(), c.get(), d.get()))
        });
        (a, ())
    });
    assert_eq!(take(&l), vec!["2 2 4"]);
    a.set(2);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["4 3 7"], "one consistent run");
}

#[test]
fn dynamic_dependencies_switch() {
    let runs = Rc::new(Cell::new(0));
    let r2 = runs.clone();
    let (screen, (flag, a, b)) = mount(move || {
        let flag = signal(true);
        let a = signal(1);
        let b = signal(10);
        let m = memo(move || {
            r2.set(r2.get() + 1);
            if flag.get() { a.get() } else { b.get() }
        });
        ((flag, a, b), text(move || m.get().to_string()))
    });
    assert_eq!(runs.get(), 1);
    b.set(11); // not a dependency yet
    screen.run_until_idle();
    assert_eq!(runs.get(), 1);
    flag.set(false);
    screen.run_until_idle();
    assert_eq!(runs.get(), 2);
    screen.get_by_text("11");
    a.set(2); // no longer a dependency
    screen.run_until_idle();
    assert_eq!(runs.get(), 2);
}

#[test]
fn handler_writes_are_batched_into_one_effect_run_and_one_batch() {
    let l = log();
    let l2 = l.clone();
    let (screen, ()) = mount(move || {
        let a = signal(0);
        let b = signal(0);
        effect(move || l2.borrow_mut().push(format!("{}+{}", a.get(), b.get())));
        (
            (),
            column((
                text(move || format!("{}/{}", a.get(), b.get())),
                {
                    let label = ("both").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    a.set(1);
                    b.set(2);
                    a.update(|v| *v += 10);
                }),
            )),
        )
    });
    take(&l);
    let before = screen.stats().batches;
    screen.get_by_role(Role::Button, "both").click();
    assert_eq!(take(&l), vec!["11+2"]);
    screen.get_by_text("11/2");
    // click produces the pointer/focus events and the ack; exactly one batch changes the text
    assert_eq!(screen.stats().batches, before + 1);
}

#[test]
fn effect_cleanup_runs_before_rerun_and_on_dispose_child_before_parent() {
    fn child(l: Log) -> impl Component {
        on_cleanup(move || l.borrow_mut().push("child cleanup".into()));
        text("child")
    }
    let l = log();
    let (l2, l3) = (l.clone(), l.clone());
    let (screen, (open, n)) = mount(move || {
        let open = signal(true);
        let n = signal(0);
        let lc = l2.clone();
        (
            (open, n),
            show(open, move || {
                let (la, lb, lc2) = (lc.clone(), lc.clone(), lc.clone());
                on_cleanup(move || la.borrow_mut().push("parent cleanup".into()));
                effect(move || {
                    let v = n.get();
                    lb.borrow_mut().push(format!("run {v}"));
                    let lb2 = lb.clone();
                    on_cleanup(move || lb2.borrow_mut().push(format!("cleanup {v}")));
                });
                let _ = &lc2;
                child(lc.clone())
            }),
        )
    });
    let _ = l3;
    assert_eq!(take(&l), vec!["run 0"]);
    n.set(1);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["cleanup 0", "run 1"]);
    open.set(false);
    screen.run_until_idle();
    assert_eq!(
        take(&l),
        vec!["child cleanup", "parent cleanup", "cleanup 1"]
    );
}

#[test]
fn small_handles_trigger_var_binding_callback_value() {
    struct Payload(u32);
    let (screen, (trig, calls, bound, derived, snapshot, payload)) = mount(|| {
        let trig = signal(());
        let calls = var(0u32);
        let hits = signal(0u32);
        let payload = signal(Payload(7));
        // Var writes are allowed in pure contexts (caches)
        let cached = memo(move || {
            trig.get();
            calls.update(|c| *c += 1);
            calls.get()
        });
        let base = signal(2i32);
        let doubled = memo(move || base.with(|v| v * 2));
        let constant: Value<i32> = 40.into_value();
        let setter = callback(move |v| base.set(v));
        let bound = binding(move || base.get(), move |value| base.set(value));
        let direct: Binding<i32> = base.into_binding();
        let snapshot = memo(move || {
            bound.get_untracked()
                + bound.with_untracked(|n| *n)
                + direct.get_untracked()
                + direct.with_untracked(|n| *n)
        });
        let bump = callback(move |()| hits.update(|h| *h += 1));
        let bumped = bump.bind(());
        let f = setter.to_fn();
        let derived = memo(move || doubled.get() + constant.get());
        (
            (trig, calls, bound, derived, snapshot, payload),
            column((
                text(move || format!("cached {}", cached.get())),
                {
                    let label = ("bump").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    assert_eq!(payload.into_value().with(|value| value.0), 7);
                    bumped.call(());
                }),
                {
                    let label = ("set9").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || f(9)),
                text(move || format!("hits {}", hits.get())),
            )),
        )
    });
    assert_eq!(payload.into_value().with(|value| value.0), 7);
    screen.get_by_text("cached 1");
    trig.notify();
    screen.run_until_idle();
    screen.get_by_text("cached 2");
    assert_eq!(calls.get(), 2);
    screen.get_by_role(Role::Button, "bump").click();
    screen.get_by_text("hits 1");
    assert_eq!(derived.get_untracked(), 44);
    assert_eq!(snapshot.get(), 8);
    bound.set(5);
    screen.run_until_idle();
    assert_eq!(bound.get_untracked(), 5);
    assert_eq!(derived.get_untracked(), 50);
    assert_eq!(snapshot.get(), 8, "untracked reads add no dependencies");
    screen.get_by_role(Role::Button, "set9").click();
    assert_eq!(derived.get_untracked(), 58);
}

// ---------------------------------------------------------------------------
// Regression: Selector prunes key nodes that the running computation just read.

// ---------------------------------------------------------------------------
// Regression: re-reading a memo inside its own `with` after invalidating it panics
// with a raw RefCell error and leaves the memo stale.

// ---------------------------------------------------------------------------
// Regression: Selector::is is stale within the turn that changed its source.

// ---------------------------------------------------------------------------
// Regression: the stabilisation cycle limit scales with the graph size
// (100 × live nodes), so an Internal-effect cycle in a large App spins for
// hundreds of thousands of mounts before `Cycle` is reported (intended: a
// fixed limit of 100 runs per effect).
