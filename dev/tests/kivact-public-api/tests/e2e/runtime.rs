//! Role: Tests for reactive graph, turns, and handle lifetimes.
//!
//! Reactive graph, turn and handle lifetime: turn ordering, the handle part of
//! the lifecycle/leak check, and graph unit tests. Pure contexts are in
//! `purity.rs`; handles of another App or a disposed owner in
//! `handle_validity.rs`.

use kivact::view::{View, callback, selector};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::support::mount;
use kivact::Binding;
use kivact::binding;
use kivact::diagnostics::ErrorKind;
use kivact::motion::FrameRate;
use kivact::motion::{Easing, Transition, animated, frame_loop};
use kivact::platform;
use kivact::prelude::*;
use kivact::task::spawn;
use kivact::test::Screen;
use kivact::view::error_boundary;

type Log = Rc<RefCell<Vec<String>>>;

fn log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

fn take(l: &Log) -> Vec<String> {
    std::mem::take(&mut *l.borrow_mut())
}

fn kinds(screen: &Screen) -> Vec<ErrorKind> {
    screen.errors().iter().map(|e| e.kind().clone()).collect()
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
fn deep_and_shallow_reads_see_one_consistent_state() {
    // "height order": an effect reading a deep memo chain and its source
    // directly never observes a mix of old and new values.
    let l = log();
    let l2 = l.clone();
    let (screen, a) = mount(move || {
        let a = signal(0);
        let m1 = memo(move || a.get() + 1);
        let m2 = memo(move || m1.get() + 1);
        let m3 = memo(move || m2.get() + 1);
        effect(move || {
            let (x, y) = (a.get(), m3.get());
            l2.borrow_mut().push(format!("{x}->{y}"));
        });
        (a, text(move || format!("{}", m3.get() - a.get())))
    });
    for v in 1..5 {
        a.set(v);
        screen.run_until_idle();
    }
    let got = take(&l);
    assert_eq!(got, vec!["0->3", "1->4", "2->5", "3->6", "4->7"]);
    assert_eq!(screen.get_by_text("3").text(), "3");
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
fn memo_cycle_is_reported_with_location_and_app_survives() {
    let (screen, (a, ok)) = mount(move || {
        let a = signal(0);
        let other = signal(0);
        let cell: Rc<Cell<Option<Value<i32>>>> = Rc::new(Cell::new(None));
        let c2 = cell.clone();
        let m1 = memo(move || a.get() + c2.get().map(|m| m.get()).unwrap_or(0));
        let m2 = memo(move || m1.get() + 1);
        cell.set(Some(m2));
        (
            (a, other),
            column((
                text(move || m2.try_get().unwrap_or(-1).to_string()),
                text(move || other.get().to_string()),
            )),
        )
    });
    let _ = a;
    let ks = kinds(&screen);
    assert!(
        ks.iter()
            .any(|k| matches!(k, ErrorKind::Cycle { path } if !path.is_empty())),
        "{ks:?}"
    );
    assert!(screen.errors().iter().any(|e| e.location().is_some()));
    ok.set(5);
    screen.run_until_idle();
    screen.get_by_text("5");
}

#[test]
fn user_effect_cycle_stops_after_limit() {
    let (screen, s) = mount(move || {
        let s = signal(0);
        effect(move || {
            let v = s.get();
            s.set(v + 1);
        });
        (s, text(move || s.get().to_string()))
    });
    let ks = kinds(&screen);
    assert!(
        ks.iter().any(|k| matches!(k, ErrorKind::Cycle { .. })),
        "{ks:?}"
    );
    let v = s.get_untracked();
    assert!((100..=102).contains(&v), "stopped after the limit, got {v}");
}

#[test]
fn borrowed_write_is_rejected_and_other_writes_are_immediate() {
    let (screen, (s, t)) = mount(move || {
        let s = signal(1);
        let t = signal(0);
        let v = {
            let label = ("go").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .on_press(move || {
            // Writes to other signals inside `with` are immediate.
            s.with(|x| t.set(*x + 1));
            assert_eq!(t.get(), 2, "read-after-write inside the handler");
            // Writing the borrowed signal itself is BorrowedWrite.
            s.with(|x| s.set(*x + 5));
        });
        ((s, t), v)
    });
    screen.get_by_role(Role::Button, "go").click();
    assert_eq!(t.get_untracked(), 2);
    assert_eq!(s.get_untracked(), 1, "borrowed write rejected");
    let e = screen.errors();
    let bw = e.iter().find(|e| *e.kind() == ErrorKind::BorrowedWrite);
    assert!(
        bw.is_some_and(|e| e.location().is_some_and(|l| l.file() == file!())),
        "{e:?}"
    );
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
fn equal_write_produces_no_ops_and_no_revision() {
    let (screen, s) = mount(move || {
        let s = signal(String::from("same"));
        (s, text(s))
    });
    let before = screen.stats();
    s.set("same".into());
    screen.run_until_idle();
    s.update(|_| {}); // always notifies; output equal → still no op
    screen.run_until_idle();
    let after = screen.stats();
    assert_eq!(
        after.batches, before.batches,
        "no batch, revision not advanced"
    );
    assert_eq!(after.ops, before.ops);
}

#[test]
fn panics_in_handlers_and_setup_are_isolated() {
    fn broken() -> View {
        panic!("setup exploded");
    }
    let (screen, n) = mount(move || {
        let n = signal(0);
        (
            n,
            column((
                // The nearest boundary isolates the setup panic.
                error_boundary(broken),
                {
                    let label = ("boom").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || -> kivact::Result<()> { panic!("handler exploded") }),
                {
                    let label = ("inc").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || n.update(|v| *v += 1)),
                text(move || n.get().to_string()),
            )),
        )
    });
    screen.get_by_role(Role::Button, "boom").click();
    screen.get_by_role(Role::Button, "inc").click();
    assert_eq!(n.get_untracked(), 1);
    let msgs: Vec<String> = screen.errors().iter().map(|e| e.to_string()).collect();
    assert!(
        msgs.iter().any(|m| m.contains("setup exploded")),
        "{msgs:?}"
    );
    assert!(
        msgs.iter().any(|m| m.contains("handler exploded")),
        "{msgs:?}"
    );
}

#[test]
fn setup_only_apis_outside_setup_are_located_errors() {
    let (screen, ()) = mount(move || {
        let s = signal(1);
        (
            (),
            column((
                {
                    let label = ("late").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    let _ = signal(5);
                }),
                text(move || {
                    // A memo inside a render closure would create a memo per run.
                    let _ = memo(move || s.with(|v| v + 1));
                    s.get().to_string()
                }),
            )),
        )
    });
    let setup_only: Vec<_> = screen
        .errors()
        .into_iter()
        .filter(|e| matches!(e.kind(), ErrorKind::SetupOnly { .. }))
        .collect();
    assert_eq!(setup_only.len(), 1, "map in render closure");
    screen.get_by_role(Role::Button, "late").click();
    let e = screen.errors();
    let late = e
        .iter()
        .filter(|e| matches!(e.kind(), ErrorKind::SetupOnly { api: "signal" }))
        .count();
    assert_eq!(late, 1);
    assert!(e.iter().all(|e| e.location().is_some()));
}

#[test]
fn a_missing_context_panics_into_the_boundary_and_nearest_context_wins() {
    #[derive(Clone, Copy)]
    struct Depth(u32);
    fn inner() -> impl IntoView {
        let d = context::<Depth>();
        text(format!("depth {}", d.0))
    }
    fn middle() -> impl IntoView {
        provide(Depth(2), inner)
    }
    fn needs_missing() -> impl IntoView {
        #[derive(Clone, Copy)]
        struct Nope;
        let _ = context::<Nope>();
        text("unreachable")
    }
    let (screen, ()) = mount(|| {
        (
            (),
            provide(Depth(1), || {
                column((middle(), inner(), error_boundary(needs_missing)))
            }),
        )
    });
    screen.get_by_text("depth 2");
    screen.get_by_text("depth 1");
    assert!(screen.query_by_text("unreachable").is_none());
    assert!(kinds(&screen).iter().any(|k| matches!(k, ErrorKind::Panic)));
}

#[test]
fn effect_cleanup_runs_before_rerun_and_on_dispose_child_before_parent() {
    fn child(l: Log) -> impl IntoView {
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
fn trace_subscriber_does_not_change_results() {
    struct Count(std::sync::atomic::AtomicUsize);
    impl tracing::Subscriber for Count {
        fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
            true
        }
        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }
        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
        fn event(&self, _: &tracing::Event<'_>) {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        fn enter(&self, _: &tracing::span::Id) {}
        fn exit(&self, _: &tracing::span::Id) {}
    }
    fn scenario() -> (Vec<String>, usize) {
        let (screen, s) = mount(|| {
            let s = signal(0);
            let m = memo(move || s.get() * 3);
            effect(move || {
                if m.get() == 6 {
                    s.set(3);
                }
            });
            (s, text(move || m.get().to_string()))
        });
        s.set(2);
        screen.run_until_idle();
        let texts = vec![
            screen
                .snapshot()
                .children
                .first()
                .map(|c| c.name.clone())
                .unwrap_or_default(),
        ];
        (texts, screen.stats().batches as usize)
    }
    let counter = std::sync::Arc::new(Count(std::sync::atomic::AtomicUsize::new(0)));
    let traced = tracing::subscriber::with_default(counter.clone(), scenario);
    let plain = scenario();
    assert_eq!(plain, traced);
    // Event delivery depends on tracing's process-wide callsite cache, which
    // other test threads share; only the equality of results is asserted.
    let _ = counter.0.load(std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn thousand_mount_cycles_leave_no_residue() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        (
            open,
            show(open, || {
                let s = signal(1);
                let m = memo(move || s.get() + 1);
                let v = var(0);
                let cb = callback(move |x| s.set(x));
                effect(move || v.set(m.get()));
                on_cleanup(move || cb.call(0));
                let r = node_ref();
                let _task = spawn(async move {
                    kivact::task::sleep(std::time::Duration::from_secs(3600)).await;
                });
                column((text(move || m.get().to_string()), node().node_ref(r)))
            }),
        )
    });
    let base = screen.stats();
    for _ in 0..1000 {
        open.set(true);
        screen.run_until_idle();
        open.set(false);
        screen.run_until_idle();
    }
    let end = screen.stats();
    assert_eq!(end.live_reactive, base.live_reactive);
    assert_eq!(end.live_owners, base.live_owners);
    assert_eq!(end.live_objects, base.live_objects);
    assert_eq!(end.tree_nodes, base.tree_nodes);
    assert_eq!(end.timers, base.timers);
    assert!(
        end.arena_slots <= base.arena_slots + 64,
        "{} vs {}",
        end.arena_slots,
        base.arena_slots
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_million_handles_created_and_disposed_reuse_bounded_slots() {
    let (screen, rounds) = mount(|| {
        let rounds = signal(0u32);
        (
            rounds,
            dynamic(rounds, |r| {
                let mut last = None;
                for i in 0..10_000 {
                    last = Some(signal(i + r));
                }
                let last = last.map(|s| s.get_untracked()).unwrap_or(0);
                text(last.to_string())
            }),
        )
    });
    let base = screen.stats();
    for i in 1..=100 {
        rounds.set(i);
        screen.run_until_idle();
    }
    let end = screen.stats();
    screen.get_by_text("10099");
    assert_eq!(end.live_reactive, base.live_reactive);
    assert!(end.arena_slots <= base.arena_slots + 64);
}

#[test]
fn selector_notifies_only_the_two_affected_keys() {
    let runs = Rc::new(Cell::new(0));
    let r2 = runs.clone();
    let (screen, sel) = mount(move || {
        let sel = signal(None::<u32>);
        let is = selector(sel);
        let r3 = r2.clone();
        (
            sel,
            column(for_each(
                (0..1000u32).collect::<Vec<_>>(),
                |n| *n,
                move |id, _| {
                    let r4 = r3.clone();
                    text(move || {
                        r4.set(r4.get() + 1);
                        if is.is(&id) {
                            format!("[{id}]")
                        } else {
                            id.to_string()
                        }
                    })
                },
            )),
        )
    });
    runs.set(0);
    sel.set(Some(10));
    screen.run_until_idle();
    assert_eq!(runs.get(), 1);
    sel.set(Some(20));
    screen.run_until_idle();
    assert_eq!(runs.get(), 3, "old and new selection only");
    screen.get_by_text("[20]");
    screen.get_by_text("10");
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
        let direct: Binding<i32> = base.into();
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

/// `optional` reads `Some(current)`, writes `Some(v)` through and ignores
/// `None`, for a signal and for a binding made of a value and a setter.
#[test]
fn optional_bindings_read_some_write_some_and_ignore_none() {
    let (screen, (signal_side, pair_side, base, pair_base)) = mount(|| {
        let base = signal(1i32);
        let pair_base = signal(10i32);
        let signal_side = base.optional();
        let pair_side = binding(move || pair_base.get(), move |v| pair_base.set(v)).optional();
        (
            (signal_side, pair_side, base, pair_base),
            text(move || format!("{} {}", base.get(), pair_base.get())),
        )
    });
    screen.get_by_text("1 10");
    assert_eq!(signal_side.get_untracked(), Some(1));
    assert_eq!(pair_side.get_untracked(), Some(10));
    signal_side.set(Some(2));
    pair_side.set(Some(20));
    screen.run_until_idle();
    screen.get_by_text("2 20");
    signal_side.set(None);
    pair_side.set(None);
    screen.run_until_idle();
    screen.get_by_text("2 20");
    base.set(3);
    pair_base.set(30);
    screen.run_until_idle();
    assert_eq!(signal_side.get_untracked(), Some(3));
    assert_eq!(pair_side.get_untracked(), Some(30));
}

#[derive(Clone, Copy)]
struct Ctx(Signal<i32>);

/// Lifecycle/leak check: every owner-bound registration kind — reactive nodes,
/// tasks and timers, resource/action/queue with pending work and `on_done`
/// subscriptions, frame loops, running animations and style transitions,
/// selector, context, node refs with a focus request, platform effects —
/// leaves nothing behind after 1,000 mount/unmount cycles.
#[test]
fn thousand_cycles_of_every_registration_kind_leave_no_residue() {
    use std::time::Duration;
    let (screen, open) = mount(|| {
        let open = signal(false);
        (
            open,
            show(open, || {
                let s = signal(1);
                provide(Ctx(s), move || {
                    let ctx = context::<Ctx>();
                    let sel = selector(memo(move || Some(ctx.0.get())));
                    let res = resource(
                        move || s.get(),
                        |_: i32| async move {
                            kivact::task::sleep(Duration::from_secs(3600)).await;
                            Ok(1)
                        },
                    );
                    let act = action(|x: i32| async move {
                        kivact::task::sleep(Duration::from_secs(3600)).await;
                        Ok(x)
                    });
                    act.on_done(move |_| s.set(0));
                    act.run(1);
                    let q = action(|x: i32| async move {
                        let _ = x;
                        Ok(())
                    })
                    .policy(ActionPolicy::Queue);
                    q.on_done(move |_| {});
                    q.run(1);
                    q.run(2);
                    let fl = frame_loop(|_| {});
                    fl.start(FrameRate::Hz(60.0));
                    let target = signal(0.0f32);
                    let anim = animated(target, Transition::tween(10_000, Easing::Linear));
                    target.set(1.0);
                    let r = node_ref();
                    on_mount(move || r.focus());
                    spawn(platform::copy_text("x"));
                    spawn(async move {
                        kivact::task::sleep(Duration::from_secs(3600)).await;
                    });
                    column((
                        text(move || format!("{} {:?} {}", sel.is(&1), res.latest(), anim.get())),
                        {
                            let label = ("b").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .node_ref(r)
                        .style(style().opacity(0.5).transition(
                            kivact::style::TransitionProp::Paint,
                            Transition::tween(500, Easing::Linear),
                        )),
                    ))
                })
            }),
        )
    });
    let base = screen.stats();
    for _ in 0..1000 {
        open.set(true);
        screen.run_until_idle();
        open.set(false);
        screen.run_until_idle();
    }
    let end = screen.stats();
    assert_eq!(end.live_reactive, base.live_reactive);
    assert_eq!(end.live_owners, base.live_owners);
    assert_eq!(end.live_objects, base.live_objects);
    assert_eq!(end.tree_nodes, base.tree_nodes);
    assert_eq!(end.timers, base.timers);
    assert_eq!(end.frame_loops, base.frame_loops);
    assert_eq!(end.animations, base.animations);
    assert!(
        end.arena_slots <= base.arena_slots + 64,
        "{} vs {}",
        end.arena_slots,
        base.arena_slots
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// Turn ordering: nested emits (a callback calling callbacks) and
/// re-entrant writes (an effect writing a signal another effect reads)
/// settle deterministically in one turn and one batch.
#[test]
fn nested_emits_and_reentrant_writes_settle_in_one_turn() {
    let l = log();
    let l2 = l.clone();
    let (screen, outer) = mount(move || {
        let s = signal(0);
        let doubled = signal(0);
        let inner = callback(move |x| s.set(x));
        let outer = callback(move |x| {
            inner.call(x);
            inner.call(x + 1);
            s.update(|v| *v *= 10);
        });
        effect(move || doubled.set(s.get() * 2));
        effect(move || {
            l2.borrow_mut()
                .push(format!("s={} d={}", s.get(), doubled.get()))
        });
        (
            outer,
            column((
                text(move || format!("d={}", doubled.get())),
                {
                    let label = ("go").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || outer.call(1)),
            )),
        )
    });
    take(&l);
    let before = screen.stats();
    screen.get_by_role(Role::Button, "go").click();
    let runs = take(&l);
    assert_eq!(
        runs.last().map(String::as_str),
        Some("s=20 d=40"),
        "{runs:?}"
    );
    assert_eq!(runs.len(), 1, "one settled run per turn: {runs:?}");
    screen.get_by_text("d=40");
    assert_eq!(screen.stats().batches, before.batches + 1);
    // The same sequence again gives the same result (determinism); the
    // intermediate writes notify, but the output is equal: no batch.
    let before = screen.stats();
    outer.call(1);
    screen.run_until_idle();
    assert_eq!(take(&l), vec!["s=20 d=40"]);
    assert_eq!(screen.stats().batches, before.batches);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// Regression (design survey): Selector prunes key nodes that the running computation just read.

#[test]
fn selector_keeps_reactivity_when_one_computation_reads_many_keys() {
    let (screen, sel_sig) = mount(|| {
        let selected = signal(None::<u32>);
        let sel = selector(selected);
        // One memo asks about 100 keys (e.g. "how many visible rows are selected").
        let count = memo(move || (0..100u32).filter(|k| sel.is(k)).count());
        (selected, text(move || format!("selected {}", count.get())))
    });
    screen.get_by_text("selected 0");
    sel_sig.set(Some(3));
    screen.run_until_idle();
    screen.get_by_text("selected 1");
    sel_sig.set(Some(90));
    screen.run_until_idle();
    screen.get_by_text("selected 1");
    sel_sig.set(None);
    screen.run_until_idle();
    screen.get_by_text("selected 0");
}

// ---------------------------------------------------------------------------
// Regression (design survey): re-reading a memo inside its own `with` after invalidating it panics
// with a raw RefCell error and leaves the memo stale.

#[test]
fn memo_reread_inside_with_is_a_structured_error_and_memo_recovers() {
    let (screen, (a, m)) = mount(|| {
        let a = signal(1);
        let m = memo(move || a.get() * 10);
        (
            (a, m),
            column((
                text(move || m.get().to_string()),
                {
                    let label = ("go").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    m.with(|_| {
                        a.set(2);
                        let _ = m.get();
                    });
                }),
            )),
        )
    });
    screen.get_by_role(Role::Button, "go").click();
    assert_eq!(a.get_untracked(), 2);
    assert_eq!(
        m.get_untracked(),
        20,
        "memo left stale after the failed pull"
    );
    screen.get_by_text("20");
    let ks = kinds(&screen);
    assert!(
        ks.iter().all(|k| *k != ErrorKind::Panic),
        "raw panic instead of a structured misuse error: {:?}",
        screen.errors()
    );
}

// ---------------------------------------------------------------------------
// Regression (design survey): Selector::is is stale within the turn that changed its source.

#[test]
fn selector_is_consistent_with_its_source_inside_a_handler() {
    let seen = Rc::new(Cell::new(false));
    let s2 = seen.clone();
    let (screen, ()) = mount(move || {
        let selected = signal(None::<u32>);
        let sel = selector(selected);
        (
            (),
            {
                let label = ("pick").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || {
                selected.set(Some(1));
                s2.set(sel.is(&1));
            }),
        )
    });
    screen.get_by_role(Role::Button, "pick").click();
    assert!(
        seen.get(),
        "sel.is(&1) is false right after selected.set(Some(1))"
    );
}

// ---------------------------------------------------------------------------
// Regression (design survey): the stabilisation cycle limit scales with the graph size
// (100 × live nodes), so an Internal-effect cycle in a large App spins for
// hundreds of thousands of mounts before `Cycle` is reported (intended: a
// fixed limit of 100 runs per effect).

#[test]
fn internal_effect_cycle_stops_after_about_a_hundred_runs() {
    let mounts = Rc::new(Cell::new(0u32));
    let m2 = mounts.clone();
    let (screen, go) = mount(move || {
        // a moderately large App: 2,000 live signals
        let pad: Vec<Signal<u32>> = (0..2000).map(|i| signal(i)).collect();
        let flag = signal(false);
        let go = signal(false);
        let (m3, m4) = (m2.clone(), m2.clone());
        let v = show(flag, move || {
            m3.set(m3.get() + 1);
            flag.set(false); // each branch's setup flips the condition
            text("on")
        })
        .fallback(move || {
            m4.set(m4.get() + 1);
            if go.get_untracked() {
                flag.set(true);
            }
            text("off")
        });
        drop(pad);
        ((go, flag), v)
    });
    let (go, flag) = go;
    go.set(true);
    screen.run_until_idle();
    let before = mounts.get();
    let t0 = std::time::Instant::now();
    flag.set(true);
    screen.run_until_idle();
    let runs = mounts.get() - before;
    assert!(
        kinds(&screen)
            .iter()
            .any(|k| matches!(k, ErrorKind::Cycle { .. })),
        "no Cycle reported after {runs} branch mounts"
    );
    assert!(
        runs <= 1_000,
        "cycle mounted {runs} branches ({:?}) before it was stopped",
        t0.elapsed()
    );
}
