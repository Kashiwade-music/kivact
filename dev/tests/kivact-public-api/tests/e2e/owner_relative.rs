//! Role: Tests for owner-relative API lookups and context.
//! `platform::window_size`, `platform::copy_text`) use the nearest owner of the frame
//! stack, and the associated constructors of task, motion and view objects.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::Callback;
use kivact::control::Node;
use kivact::diagnostics::ErrorKind;
use kivact::motion::FrameRate;
use kivact::motion::{Easing, Transition, animated, frame_loop};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::current_theme;
use kivact::task::sleep;
use kivact::task::spawn;
use kivact::test::render;
use kivact::view::{callback, context, try_context};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn here(e: &UiError) -> bool {
    e.location()
        .is_some_and(|l| l.file().ends_with("owner_relative.rs"))
}

#[test]
fn signal_after_await_is_a_located_setup_only_error() {
    let (screen, ()) = mount(|| {
        (
            (),
            {
                let label = ("go").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(|| {
                spawn(async {
                    sleep(ms(5)).await;
                    let _ = signal(1);
                });
            }),
        )
    });
    screen.get_by_role(Role::Button, "go").click();
    screen.advance(ms(10));
    let n = screen
        .errors()
        .iter()
        .filter(|e| matches!(e.kind(), ErrorKind::SetupOnly { api: "signal" }) && here(e))
        .count();
    assert_eq!(n, 1);
}

#[test]
fn spawn_in_a_handler_belongs_to_the_target_node_owner() {
    let done = Rc::new(Cell::new(0));
    let d2 = done.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let d3 = d2.clone();
        let branch = show(open, move || {
            let d4 = d3.clone();
            {
                let label = ("start").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || {
                let d5 = d4.clone();
                spawn(async move {
                    sleep(ms(100)).await;
                    d5.set(d5.get() + 1);
                });
            })
        });
        (open, branch)
    });
    screen.get_by_role(Role::Button, "start").click();
    screen.advance(ms(150));
    assert_eq!(done.get(), 1, "the task completes while its owner lives");
    screen.get_by_role(Role::Button, "start").click();
    open.set(false); // disposes the branch that owns the handler's node
    screen.run_until_idle();
    screen.advance(ms(150));
    assert_eq!(done.get(), 1, "the task was cancelled with the branch");
    assert!(screen.errors().is_empty());
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Name(&'static str);

#[test]
fn context_in_a_handler_uses_the_target_node_owner() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let s2 = seen.clone();
    let (screen, ()) = mount(move || {
        let (s3, s4) = (s2.clone(), s2.clone());
        let view = column((
            provide(Name("branch"), move || {
                let s5 = s3.clone();
                {
                    let label = ("inner").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || s5.borrow_mut().push(try_context::<Name>()))
            }),
            {
                let label = ("outer").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || s4.borrow_mut().push(try_context::<Name>())),
        ));
        ((), view)
    });
    screen.get_by_role(Role::Button, "inner").click();
    screen.get_by_role(Role::Button, "outer").click();
    assert_eq!(*seen.borrow(), vec![Some(Name("branch")), None]);
}

#[test]
fn a_callback_runs_for_its_own_owner() {
    let done = Rc::new(Cell::new(false));
    let d2 = done.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let d3 = d2.clone();
        // Created by the root: a task spawned inside it belongs to the root.
        let start = callback(move |()| {
            let d4 = d3.clone();
            spawn(async move {
                sleep(ms(50)).await;
                d4.set(true);
            });
        });
        let branch = show(open, move || {
            {
                let label = ("child").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || start.call(()))
        });
        (open, branch)
    });
    screen.get_by_role(Role::Button, "child").click();
    open.set(false); // the child's owner goes away, the callback's owner stays
    screen.run_until_idle();
    screen.advance(ms(100));
    assert!(done.get(), "the task belongs to the callback's owner");
}

/// G-01: a task spawned for an owner being disposed is the runtime's to
/// release; the same disposal cancels it, so it never runs and nothing is
/// reported.
#[test]
fn spawn_on_a_disposing_owner_is_cancelled_by_the_disposal() {
    let ran = Rc::new(Cell::new(false));
    let r2 = ran.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let branch = show(open, move || {
            let r3 = r2.clone();
            on_cleanup(move || {
                let _ = spawn(async move { r3.set(true) });
            });
            text("child")
        });
        (open, branch)
    });
    let before = screen.stats();
    open.set(false);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert!(!ran.get(), "a task of a disposed owner ran");
    assert!(screen.stats().live_objects <= before.live_objects);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn spawn_outside_every_owner_is_a_misuse() {
    let r = std::panic::catch_unwind(|| spawn(async {}));
    let msg = match r {
        Ok(_) => String::from("no panic"),
        Err(p) => p
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_else(|| String::from("non-string panic")),
    };
    assert!(msg.contains("`spawn` needs an owner"), "{msg}");
}

#[test]
fn spawn_in_a_user_effect_belongs_to_the_effect_owner() {
    let got = Rc::new(Cell::new(0));
    let g2 = got.clone();
    let (screen, n) = mount(move || {
        let n = signal(0);
        effect(move || {
            let v = n.get();
            let g3 = g2.clone();
            spawn(async move { g3.set(v) });
        });
        (n, ())
    });
    n.set(5);
    screen.run_until_idle();
    assert_eq!(got.get(), 5);
}

#[test]
fn task_constructors_work_without_a_scope() {
    let (screen, (query, res, act, queue, log)) = mount(|| {
        let query = signal(1u32);
        let res = resource(
            move || query.get(),
            |q| async move {
                sleep(ms(10)).await;
                Ok(q * 10)
            },
        );
        let act = action(|n: u32| async move { Ok(n + 1) });
        let log = signal(Vec::<u32>::new());
        let queue = action(move |n: u32| async move {
            log.update(|l| l.push(n));
            Ok(())
        })
        .policy(ActionPolicy::Queue);
        ((query, res, act, queue, log), ())
    });
    screen.advance(ms(20));
    assert_eq!(res.state(), AsyncState::Ready(10));
    query.set(2);
    screen.run_until_idle();
    screen.advance(ms(20));
    assert_eq!(res.state(), AsyncState::Ready(20));
    act.run(4);
    screen.run_until_idle();
    assert_eq!(act.state(), AsyncState::Ready(5));
    queue.run(1);
    queue.run(2);
    screen.run_until_idle();
    assert_eq!(log.get_untracked(), vec![1, 2]);
}

#[test]
fn motion_and_view_constructors_work_without_a_scope() {
    let (screen, (fl, ticks, x, target, r)) = mount(|| {
        let ticks = signal(0u32);
        let fl = frame_loop(move |_| ticks.update(|t| *t += 1));
        let target = signal(0.0f32);
        let x = animated(target, Transition::tween(100.0, Easing::Linear));
        let r = node_ref();
        let view = {
            let label = ("anchor").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .node_ref(r);
        ((fl, ticks, x, target, r), view)
    });
    fl.start(FrameRate::Hz(10.0));
    screen.advance(Duration::from_secs(1));
    assert_eq!(ticks.get_untracked(), 10);
    target.set(100.0);
    screen.run_until_idle();
    screen.advance(ms(200));
    assert_eq!(x.get_untracked(), 100.0);
    r.focus();
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Button, "anchor").is_focused());
}

#[test]
fn the_theme_and_platform_services_follow_the_nearest_owner() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let s2 = seen.clone();
    let (screen, ()) = mount(move || {
        let in_setup = current_theme().colors().text;
        let s3 = s2.clone();
        let view = {
            let label = ("copy").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .on_press(move || {
            // In a handler: the button's owner.
            s3.borrow_mut()
                .push(current_theme().colors().text == in_setup);
            spawn(platform::copy_text("from handler"));
        });
        ((), view)
    });
    screen.get_by_role(Role::Button, "copy").click();
    assert_eq!(*seen.borrow(), vec![true]);
    assert_eq!(screen.clipboard().as_deref(), Some("from handler"));
}

#[test]
fn expecting_a_missing_context_in_setup_reports_a_panic_at_the_caller() {
    let screen = render(|| {
        let _ = context::<Name>();
        text("x")
    });
    assert!(
        screen
            .errors()
            .iter()
            .any(|e| matches!(e.kind(), ErrorKind::Panic) && here(e))
    );
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Lane(u32);

fn lane_label() -> impl IntoView {
    // A plain-function component reading the context in setup and in a handler.
    let lane = context::<Lane>();
    {
        let label = (format!("lane {}", lane.0)).into_value();
        node()
            .role(Role::Button)
            .focusable(true)
            .name(label)
            .child(text(label))
    }
    .on_press(move || {
        assert_eq!(try_context::<Lane>(), Some(lane));
    })
}

#[test]
fn provide_reaches_the_child_but_not_its_siblings() {
    let sibling = Rc::new(Cell::new(None));
    let s2 = sibling.clone();
    let (screen, ()) = mount(move || {
        let view = column((
            provide(Lane(3), lane_label),
            provide(Lane(4), || (lane_label(), text("same owner"))),
            // Set up after the providers were mounted: they must not leak here.
            show(
                move || true,
                move || {
                    s2.set(Some(try_context::<Lane>()));
                    text("sibling")
                },
            ),
        ));
        ((), view)
    });
    screen.get_by_role(Role::Button, "lane 3").click();
    screen.get_by_role(Role::Button, "lane 4").click();
    assert_eq!(sibling.get(), Some(None));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn provide_isolates_a_setup_panic_of_its_child() {
    let (screen, ()) = mount(|| {
        let view = column((
            provide(Lane(1), || -> Node { panic!("child setup failed") }),
            text("after"),
        ));
        ((), view)
    });
    screen.get_by_text("after");
    assert!(
        screen
            .errors()
            .iter()
            .any(|e| *e.kind() == ErrorKind::Panic && e.to_string().contains("child setup failed"))
    );
}

#[test]
fn provide_theme_applies_to_the_child_and_switches_without_remount() {
    use kivact::view::provide_theme;
    let surface = |id: &'static str| {
        text(move || format!("{:?}", current_theme().colors().surface)).test_id(id)
    };
    let (screen, theme) = mount(move || {
        let theme = signal(Theme::dark());
        let view = column((
            provide_theme(theme, move || surface("inner")),
            surface("outer"),
        ));
        (theme, view)
    });
    let dark = format!("{:?}", Theme::dark().colors().surface);
    let light = format!("{:?}", Theme::default().colors().surface);
    assert_ne!(dark, light);
    assert_eq!(screen.get_by_test_id("inner").text(), dark);
    assert_eq!(screen.get_by_test_id("outer").text(), light);
    let creates = screen.stats().creates;
    theme.set(Theme::default());
    screen.run_until_idle();
    assert_eq!(screen.get_by_test_id("inner").text(), light);
    assert_eq!(screen.stats().creates, creates, "no remount");
    // A constant theme works too.
    let screen = render(move || provide_theme(Theme::dark(), move || surface("c")));
    assert_eq!(screen.get_by_test_id("c").text(), dark);
}

#[test]
fn frame_callbacks_and_tasks_run_for_their_owner() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let s2 = seen.clone();
    let slot = Rc::new(Cell::new(None));
    let slot2 = slot.clone();
    let (screen, ()) = mount(move || {
        let s3 = s2.clone();
        let view = provide(Lane(9), move || {
            let (s4, s5) = (s3.clone(), s3.clone());
            // The frame loop and the task belong to the provider's owner.
            slot2.set(Some(frame_loop(move |_| {
                s4.borrow_mut().push(("frame", try_context::<Lane>()));
            })));
            spawn(async move {
                sleep(ms(5)).await;
                s5.borrow_mut().push(("task", try_context::<Lane>()));
            });
            text("provider")
        });
        ((), view)
    });
    // The provider's child is set up when the view is mounted.
    let fl = slot.take().expect("the provider's setup ran");
    fl.start(FrameRate::Hz(10.0));
    screen.advance(ms(150));
    fl.stop();
    let seen = seen.borrow();
    assert!(seen.contains(&("task", Some(Lane(9)))), "{seen:?}");
    assert!(seen.contains(&("frame", Some(Lane(9)))), "{seen:?}");
}

/// Records `try_context::<Lane>()` and whether `spawn` and `platform::window_size`
/// were accepted, in the context the closure runs in.
fn probe_owner(seen: &Rc<RefCell<Vec<Option<Lane>>>>) {
    seen.borrow_mut().push(try_context::<Lane>());
    let _ = spawn(async {});
    let _ = platform::window_size();
}

#[test]
fn queue_jobs_run_for_the_queue_owner() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let s2 = seen.clone();
    let (screen, ()) = mount(move || {
        let view = provide(Lane(5), move || {
            let (s3, s4) = (s2.clone(), s2.clone());
            let q = action(move || {
                probe_owner(&s3); // the run's synchronous part
                let s5 = s4.clone();
                async move {
                    sleep(ms(5)).await;
                    probe_owner(&s5); // after an await
                    Ok(())
                }
            })
            .policy(ActionPolicy::Queue);
            q.run(());
            text("queue")
        });
        ((), view)
    });
    screen.advance(ms(20));
    assert_eq!(*seen.borrow(), vec![Some(Lane(5)), Some(Lane(5))]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_queue_job_draining_after_its_owner_was_disposed_reports_disposed() {
    let go = Rc::new(Cell::new(false));
    let g2 = go.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let g3 = g2.clone();
        let branch = show(open, move || {
            let g4 = g3.clone();
            let q = action(move || {
                let g5 = g4.clone();
                async move {
                    while !g5.get() {
                        sleep(ms(5)).await;
                    }
                    let _ = try_context::<Lane>();
                    Ok(())
                }
            })
            .policy(ActionPolicy::Queue);
            q.run(());
            text("queue")
        });
        (open, branch)
    });
    open.set(false); // the queue is detached and drains
    screen.run_until_idle();
    go.set(true);
    screen.advance(ms(20));
    let kinds: Vec<ErrorKind> = screen.errors().iter().map(|e| e.kind().clone()).collect();
    assert!(
        screen
            .errors()
            .iter()
            .any(|e| matches!(e.kind(), ErrorKind::Disposed { .. }) && here(e)),
        "{kinds:?}"
    );
    assert!(
        !kinds
            .iter()
            .any(|k| matches!(k, ErrorKind::SetupOnly { .. })),
        "{kinds:?}"
    );
}

#[test]
fn queue_on_done_runs_for_the_registering_owner() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let s2 = seen.clone();
    let (screen, q) = mount(move || {
        // The queue belongs to the root; the handler to the provider's child.
        let q = action(|| async { Ok(()) }).policy(ActionPolicy::Queue);
        let view = provide(Lane(7), move || {
            let s3 = s2.clone();
            q.on_done(move |_| probe_owner(&s3));
            text("child")
        });
        (q, view)
    });
    q.run(());
    screen.run_until_idle();
    assert_eq!(*seen.borrow(), vec![Some(Lane(7))]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn action_on_done_runs_for_the_registering_owner() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let late = Rc::new(Cell::new(false));
    let (s2, l2) = (seen.clone(), late.clone());
    let (screen, (act, open)) = mount(move || {
        // The action belongs to the root; the handler to the provider's child.
        let act = action(|n: u32| async move { Ok(n) });
        let open = signal(true);
        let view = show(open, move || {
            let (s3, l3) = (s2.clone(), l2.clone());
            provide(Lane(2), move || {
                let (s4, l4) = (s3.clone(), l3.clone());
                act.on_done(move |_| {
                    s4.borrow_mut().push(try_context::<Lane>());
                    let l5 = l4.clone();
                    spawn(async move {
                        sleep(ms(50)).await;
                        l5.set(true);
                    });
                });
                text("child")
            })
        });
        ((act, open), view)
    });
    act.run(1);
    screen.run_until_idle();
    assert_eq!(*seen.borrow(), vec![Some(Lane(2))]);
    open.set(false); // the registering child goes away before the task ends
    screen.run_until_idle();
    screen.advance(ms(100));
    assert!(!late.get(), "the task belonged to the registering child");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_callback_uses_its_owner_when_called_from_setup_an_effect_or_a_handler() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let late = Rc::new(Cell::new(0));
    let (s2, l2) = (seen.clone(), late.clone());
    let slot: Rc<Cell<Option<Callback<&'static str>>>> = Rc::new(Cell::new(None));
    let (screen, (n, open)) = mount(move || {
        let n = signal(0);
        let open = signal(true);
        let slot2 = slot.clone();
        let view = column((
            // The callback's owner provides `Lane(1)`.
            provide(Lane(1), move || {
                let (s3, l3) = (s2.clone(), l2.clone());
                slot.set(Some(callback(move |from| {
                    let _ = n.get(); // untracked: a calling effect does not subscribe
                    s3.borrow_mut().push((from, try_context::<Lane>()));
                    let l4 = l3.clone();
                    spawn(async move {
                        sleep(ms(50)).await;
                        l4.set(l4.get() + 1);
                    });
                })));
                text("owner")
            }),
            // The caller is set up after the provider and sees no `Lane`.
            show(open, move || {
                let cb = slot2.get().expect("the provider was set up first");
                cb.call("setup");
                effect(move || cb.call("effect"));
                {
                    let label = ("call").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || cb.call("handler"))
            }),
        ));
        ((n, open), view)
    });
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "call").click();
    n.set(1); // the effect does not track what the callback read
    screen.run_until_idle();
    assert_eq!(
        *seen.borrow(),
        vec![
            ("setup", Some(Lane(1))),
            ("effect", Some(Lane(1))),
            ("handler", Some(Lane(1))),
        ]
    );
    open.set(false); // the caller goes away; the callback's owner stays
    screen.run_until_idle();
    screen.advance(ms(100));
    assert_eq!(late.get(), 3, "every task belonged to the callback's owner");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_read_in_a_callback_does_not_subscribe_the_calling_effect_or_memo() {
    let effect_runs = Rc::new(Cell::new(0));
    let memo_runs = Rc::new(Cell::new(0));
    let (e2, m2) = (effect_runs.clone(), memo_runs.clone());
    let (screen, n) = mount(move || {
        let n = signal(0);
        let read = callback(move |()| {
            let _ = n.get();
        });
        effect(move || {
            e2.set(e2.get() + 1);
            read.call(());
        });
        let m = memo(move || {
            m2.set(m2.get() + 1);
            read.call(());
            1
        });
        (n, text(move || m.get().to_string()))
    });
    screen.run_until_idle();
    assert_eq!((effect_runs.get(), memo_runs.get()), (1, 1));
    n.set(1);
    screen.run_until_idle();
    assert_eq!(
        (effect_runs.get(), memo_runs.get()),
        (1, 1),
        "a read inside `Callback::call` is not a dependency of the caller"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_read_in_an_action_run_closure_does_not_subscribe_the_calling_effect() {
    let runs = Rc::new(Cell::new(0));
    let r2 = runs.clone();
    let (screen, n) = mount(move || {
        let n = signal(0);
        let act = action(move || {
            let _ = n.get(); // the synchronous part of the run closure
            async { Ok::<(), UiError>(()) }
        });
        effect(move || {
            r2.set(r2.get() + 1);
            act.run(());
        });
        (n, text("action"))
    });
    screen.run_until_idle();
    assert_eq!(runs.get(), 1);
    n.set(1);
    screen.run_until_idle();
    assert_eq!(runs.get(), 1, "the run closure is untracked");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn creation_inside_a_callback_called_during_setup_belongs_to_the_caller() {
    let made: Rc<Cell<Option<Signal<u32>>>> = Rc::new(Cell::new(None));
    let m2 = made.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let make = callback(move |()| m2.set(Some(signal(5))));
        let branch = show(open, move || {
            make.call(());
            text("caller")
        });
        (open, branch)
    });
    let s = made
        .get()
        .expect("the callback ran during the caller's setup");
    assert_eq!(s.try_get(), Some(5));
    open.set(false); // disposes the caller, not the callback's owner
    screen.run_until_idle();
    assert_eq!(s.try_get(), None, "the signal belonged to the caller");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
