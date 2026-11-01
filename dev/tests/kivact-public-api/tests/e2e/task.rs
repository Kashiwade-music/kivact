//! Role: Async: executor, blocking pool, resource, action, queue, time, platform.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::diagnostics::ErrorKind;
use kivact::platform;
use kivact::prelude::*;
use kivact::task::spawn;
use kivact::task::{CancelToken, sleep};
use kivact::test::Screen;
use kivact::view::error_boundary;

fn kinds(screen: &Screen) -> Vec<ErrorKind> {
    screen.errors().iter().map(|e| e.kind().clone()).collect()
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn dropping_an_app_finishes_while_a_worker_keeps_waking_its_task() {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::task::{Poll, Waker};
    use std::time::Instant;

    let waker = Arc::new(Mutex::new(None::<Waker>));
    let stored = waker.clone();
    let (screen, ()) = mount(move || {
        spawn(std::future::poll_fn(move |cx| {
            *stored.lock().unwrap() = Some(cx.waker().clone());
            Poll::<()>::Pending
        }));
        ((), text("pending worker task"))
    });
    let done = Arc::new(AtomicBool::new(false));
    let wakes = Arc::new(AtomicUsize::new(0));
    let worker_done = done.clone();
    let worker_wakes = wakes.clone();
    let worker = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let waker = waker.lock().unwrap().clone().expect("task was polled");
        while !worker_done.load(Ordering::Acquire) && Instant::now() < deadline {
            waker.wake_by_ref();
            worker_wakes.fetch_add(1, Ordering::Release);
            std::thread::yield_now();
        }
    });
    while wakes.load(Ordering::Acquire) == 0 {
        std::thread::yield_now();
    }
    let started = Instant::now();
    drop(screen);
    let elapsed = started.elapsed();
    done.store(true, Ordering::Release);
    worker.join().unwrap();
    assert!(
        elapsed < Duration::from_secs(1),
        "app drop took {elapsed:?}"
    );
}

/// `AsyncState` answers `is_ready`, `is_failed`, `value` and `error` in each
/// state, and an action and a resource read the error directly.
#[test]
fn async_state_helpers_and_direct_errors_follow_each_state() {
    let (screen, (act, res)) = mount(|| {
        let act = action(|n: u32| async move {
            sleep(ms(10)).await;
            if n == 0 {
                Err(UiError::msg("zero"))
            } else {
                Ok(n)
            }
        });
        let res = resource(
            || (),
            |()| async {
                sleep(ms(10)).await;
                Err::<u32, _>(UiError::msg("no data"))
            },
        );
        (
            (act, res),
            text(move || {
                format!(
                    "{} {}",
                    act.error().map(|e| e.to_string()).unwrap_or_default(),
                    res.error().map(|e| e.to_string()).unwrap_or_default()
                )
            }),
        )
    });
    let idle = AsyncState::<u32>::Idle;
    assert!(
        !idle.is_ready() && !idle.is_failed() && idle.value().is_none() && idle.error().is_none()
    );
    assert!(act.error().is_none(), "no error before the first run");
    assert!(res.error().is_none(), "no error while the fetch runs");
    act.run(5);
    screen.run_until_idle();
    let pending = act.state();
    assert!(pending.is_pending() && !pending.is_ready() && pending.value().is_none());
    screen.advance(ms(20));
    let ready = act.state();
    assert!(ready.is_ready() && !ready.is_failed());
    assert_eq!(ready.value(), Some(&5));
    assert!(ready.error().is_none() && act.error().is_none());
    act.run(0);
    screen.advance(ms(20));
    let failed = act.state();
    assert!(failed.is_failed() && !failed.is_ready() && failed.value().is_none());
    assert_eq!(
        failed.error().map(|e| e.to_string()),
        act.error().map(|e| e.to_string())
    );
    assert!(act.error().is_some_and(|e| e.to_string().contains("zero")));
    assert!(
        res.error()
            .is_some_and(|e| e.to_string().contains("no data"))
    );
    assert!(res.state().is_failed());
    screen.get_by_text("zero no data");
}

#[test]
fn resource_accepts_only_the_latest_request() {
    let publishes = Rc::new(Cell::new(0));
    let p2 = publishes.clone();
    let (screen, (query, res)) = mount(move || {
        let query = signal(String::from("slow"));
        let res = resource(
            move || query.get(),
            |q: String| async move {
                // "slow" takes 100ms, anything else 10ms
                let d = if q == "slow" { 100 } else { 10 };
                sleep(ms(d)).await;
                Ok(format!("result {q}"))
            },
        );
        let p3 = p2.clone();
        effect(move || {
            if res.state().is_ready() {
                p3.set(p3.get() + 1);
            }
        });
        (
            (query, res),
            column((
                show(move || res.state().is_pending(), || text("loading")),
                text(move || res.latest().unwrap_or_default()).test_id("value"),
            )),
        )
    });
    screen.get_by_text("loading");
    screen.advance(ms(5));
    query.set("fast".into()); // supersedes the slow request
    screen.run_until_idle();
    screen.advance(ms(200));
    assert_eq!(screen.get_by_test_id("value").text(), "result fast");
    assert_eq!(publishes.get(), 1, "the slow result was never published");
    assert_eq!(res.state(), AsyncState::Ready("result fast".to_string()));
    // refetch and cancel after ready
    res.refetch();
    screen.run_until_idle();
    assert!(res.state().is_pending());
    res.cancel();
    screen.advance(ms(50));
    assert!(
        res.error().is_some_and(|e| e.is_cancelled()),
        "{:?}",
        res.state()
    );
    assert_eq!(
        res.latest().as_deref(),
        Some("result fast"),
        "latest survives cancel"
    );
    res.cancel(); // no-op when nothing is in flight
}

#[test]
fn resource_errors_and_blocking_panics_become_ui_errors() {
    let (screen, res) = mount(|| {
        let n = signal(0u32);
        let res = resource(
            move || n.get(),
            |n: u32| async move {
                blocking(move || {
                    if n == 0 {
                        panic!("worker exploded");
                    }
                    n * 2
                })
                .await;
                Err::<u32, _>(std::io::Error::other("io failure").into())
            },
        );
        (
            res,
            show_some(move || res.error(), |e| text(move || e.get().to_string())),
        )
    });
    screen.find_by_text("panicked: worker exploded");
    assert!(matches!(
        res.error().map(|e| e.kind().clone()),
        Some(ErrorKind::Panic)
    ));
}

#[test]
fn action_latest_wins_retry_cancel_and_multiple_done_handlers() {
    let done = Rc::new(RefCell::new(Vec::new()));
    let d2 = done.clone();
    let (screen, (act, counter)) = mount(move || {
        let attempts = var(0u32);
        let counter = signal(0u32);
        let act = action(move |delay: u64| async move {
            sleep(ms(delay)).await;
            attempts.update(|a| *a += 1);
            if delay == 13 {
                return Err(UiError::msg("unlucky"));
            }
            Ok(delay * 2)
        });
        let (d3, d4) = (d2.clone(), d2.clone());
        act.on_done(move |r: &Result<u64, UiError>| d3.borrow_mut().push(format!("{r:?}")));
        act.on_done(move |_| {
            d4.borrow_mut().push("second".into());
            counter.update(|c| *c += 1); // writes in completion handlers publish in the same turn
        });
        (
            (act, counter),
            text(move || format!("count {}", counter.get())),
        )
    });
    act.run(100);
    screen.run_until_idle();
    assert!(act.state().is_pending());
    act.run(10); // supersedes
    screen.advance(ms(300));
    assert_eq!(act.state(), AsyncState::Ready(20));
    assert_eq!(act.input(), Some(10));
    assert_eq!(done.borrow().len(), 2, "one completion, two handlers");
    screen.get_by_text("count 1");
    act.run(13);
    screen.advance(ms(20));
    assert!(act.state().is_failed());
    assert_eq!(act.input(), Some(13));
    assert_eq!(act.latest(), Some(20), "latest survives a failure");
    act.retry();
    screen.run_until_idle();
    assert_eq!(act.state(), AsyncState::Pending);
    assert_eq!((act.input(), act.attempt()), (Some(13), 2));
    act.cancel();
    screen.run_until_idle();
    assert!(
        act.error().is_some_and(|e| e.is_cancelled()),
        "{:?}",
        act.state()
    );
    assert_eq!(act.input(), Some(13));
    screen.advance(ms(50));
    assert_eq!(counter.get_untracked(), 2, "cancelled run never completes");
}

#[test]
fn completion_during_unmount_is_dropped() {
    let finished = Rc::new(Cell::new(false));
    let f2 = finished.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let f3 = f2.clone();
        (
            open,
            show(open, move || {
                let f4 = f3.clone();
                let t = spawn(async move {
                    sleep(ms(50)).await;
                    f4.set(true);
                });
                let _ = t;
                text("child")
            }),
        )
    });
    screen.advance(ms(10));
    open.set(false);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert!(!finished.get(), "task cancelled with its owner");
    assert_eq!(screen.stats().timers, 0, "timer registration released");
    assert!(screen.errors().is_empty());
}

#[test]
fn queue_orders_coalesces_recovers_and_drains() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l2 = log.clone();
    let (screen, (fifo, latest)) = mount(move || {
        let (la, lb) = (l2.clone(), l2.clone());
        let fifo = action(move |n: u32| {
            let la = la.clone();
            async move {
                sleep(ms(10)).await;
                if n == 2 {
                    return Err(UiError::msg("two failed"));
                }
                la.borrow_mut().push(format!("fifo {n}"));
                Ok(())
            }
        })
        .policy(ActionPolicy::Queue);
        let latest = action(move |n: u32| {
            let lb = lb.clone();
            async move {
                sleep(ms(10)).await;
                lb.borrow_mut().push(format!("latest {n}"));
                Ok(())
            }
        })
        .policy(ActionPolicy::QueueLatest);
        let results = var(Vec::<bool>::new());
        fifo.on_done(move |r| results.update(|v| v.push(r.is_ok())));
        (
            (fifo, latest),
            text(move || format!("{:?}", fifo.state())).test_id("status"),
        )
    });
    fifo.run(1);
    latest.run(1);
    screen.run_until_idle(); // run 1 of each queue is now running
    for n in 2..=4 {
        fifo.run(n);
        latest.run(n);
    }
    screen.run_until_idle();
    assert_eq!(fifo.pending_len(), 3, "first run running, three waiting");
    assert_eq!(latest.pending_len(), 1, "waiting runs coalesced");
    assert_eq!(fifo.state(), AsyncState::Pending);
    screen.advance(ms(15));
    screen.advance(ms(10)); // t = 25: run 2 failed at 20, run 3 running
    assert!(
        matches!(fifo.state(), AsyncState::Failed(_)),
        "{:?}",
        fifo.state()
    );
    assert_eq!(fifo.input(), Some(3), "the running run's input");
    screen.advance(ms(100));
    assert_eq!(fifo.state(), AsyncState::Ready(()));
    assert_eq!(fifo.pending_len(), 0);
    let got = log.borrow().clone();
    let fifo_order: Vec<&String> = got.iter().filter(|s| s.starts_with("fifo")).collect();
    assert_eq!(
        fifo_order,
        vec!["fifo 1", "fifo 3", "fifo 4"],
        "order kept, failure skipped"
    );
    let latest_order: Vec<&String> = got.iter().filter(|s| s.starts_with("latest")).collect();
    assert_eq!(
        latest_order,
        vec!["latest 1", "latest 4"],
        "running job kept, waiting ones coalesced"
    );
}

#[test]
fn queue_drains_on_owner_dispose_and_is_abandoned_after_the_deadline() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l2 = log.clone();
    let (screen, (open, q)) = mount(move || {
        let open = signal(true);
        let holder: Rc<Cell<Option<Action<(), u64>>>> = Rc::new(Cell::new(None));
        let h2 = holder.clone();
        let l3 = l2.clone();
        let view = show(open, move || {
            let l4 = l3.clone();
            let q = action(move |d: u64| {
                let l5 = l4.clone();
                async move {
                    sleep(ms(d)).await;
                    l5.borrow_mut().push(d);
                    Ok(())
                }
            })
            .policy(ActionPolicy::Queue);
            h2.set(Some(q));
            text("owner")
        });
        ((open, holder), view)
    });
    let q = q.get().expect("queue");
    q.run(100);
    q.run(200);
    q.run(5000); // exceeds the 3 s drain deadline
    screen.run_until_idle();
    open.set(false); // owner disposed: queue keeps draining detached
    screen.run_until_idle();
    screen.advance(ms(400));
    assert_eq!(*log.borrow(), vec![100, 200]);
    screen.advance(ms(4000));
    assert_eq!(
        *log.borrow(),
        vec![100, 200],
        "5 s job abandoned at the deadline"
    );
    assert!(
        screen
            .errors()
            .iter()
            .any(|e| *e.kind() == ErrorKind::Closed)
    );
}

#[test]
fn app_close_waits_for_queues_with_a_deadline() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l2 = log.clone();
    let (screen, q) = mount(move || {
        let q = action(move |d: u64| {
            let l3 = l2.clone();
            async move {
                sleep(ms(d)).await;
                l3.borrow_mut().push(d);
                Ok(())
            }
        })
        .policy(ActionPolicy::Queue);
        (q, text("app"))
    });
    q.run(500);
    screen.run_until_idle();
    assert!(screen.request_close());
    assert_eq!(*log.borrow(), vec![500], "drained before exit");
}

#[test]
fn queue_close_future_reports_abandoned_items() {
    let outcome = Rc::new(RefCell::new(None));
    let o2 = outcome.clone();
    let (screen, ()) = mount(move || {
        let q = action(move |d: u64| async move {
            sleep(ms(d)).await;
            Ok(())
        })
        .policy(ActionPolicy::Queue);
        q.run(50);
        q.run(10_000);
        let o3 = o2.clone();
        spawn(async move {
            let r = q.close(Duration::from_secs(1)).await;
            *o3.borrow_mut() = Some(r.map_err(|e| e.to_string()));
        });
        ((), text("q"))
    });
    screen.advance(ms(1500));
    let r = outcome.borrow().clone();
    assert!(
        matches!(r, Some(Err(ref m)) if m.contains("abandoned")),
        "{r:?}"
    );
}

#[test]
fn cancelling_a_queued_action_drops_its_waiting_runs() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l2 = log.clone();
    let (screen, q) = mount(move || {
        let q = action(move |d: u64| {
            let l3 = l2.clone();
            async move {
                sleep(ms(d)).await;
                l3.borrow_mut().push(d);
                Ok(())
            }
        })
        .policy(ActionPolicy::Queue);
        (q, text("q"))
    });
    q.run(50);
    q.run(60);
    screen.run_until_idle();
    assert_eq!(q.pending_len(), 1);
    q.cancel();
    screen.run_until_idle();
    assert!(
        q.error().is_some_and(|e| e.is_cancelled()),
        "{:?}",
        q.state()
    );
    assert_eq!(q.pending_len(), 0);
    screen.advance(ms(200));
    assert!(log.borrow().is_empty(), "cancelled runs never finish");
    q.run(10);
    screen.run_until_idle();
    assert_eq!(q.state(), AsyncState::Pending, "a cancellation is not kept");
    screen.advance(ms(20));
    assert_eq!(q.state(), AsyncState::Ready(()));
    assert_eq!(*log.borrow(), vec![10]);
}

#[test]
fn timeout_and_sleep_use_the_virtual_clock() {
    let res = Rc::new(RefCell::new(Vec::new()));
    let r2 = res.clone();
    let (screen, ()) = mount(move || {
        let r3 = r2.clone();
        spawn(async move {
            let quick = kivact::task::timeout(ms(50), sleep(ms(10))).await;
            let slow = kivact::task::timeout(ms(50), sleep(ms(100))).await;
            r3.borrow_mut().push(format!(
                "{} {}",
                quick.is_ok(),
                slow.map_err(|e| e.kind().clone()).is_err()
            ));
        });
        ((), text("t"))
    });
    screen.advance(ms(40));
    assert!(res.borrow().is_empty());
    screen.advance(ms(40));
    assert_eq!(*res.borrow(), vec!["true true"]);
    assert!(screen.now() >= ms(80));
}

#[test]
fn blocking_runs_on_the_pool_and_returns_on_the_ui_thread() {
    let (screen, res) = mount(|| {
        let res = resource(
            || 21u64,
            |n: u64| async move {
                let ui_thread = std::thread::current().id();
                let (v, worker) = blocking(move || (n * 2, std::thread::current().id())).await;
                assert_ne!(worker, ui_thread, "ran on a worker");
                assert_eq!(
                    std::thread::current().id(),
                    ui_thread,
                    "resumed on the UI thread"
                );
                // The other form takes the cancellation token; nothing has
                // dropped this future, so it is not cancelled.
                let cancelled = blocking(|cancel: CancelToken| cancel.is_cancelled()).await;
                assert!(!cancelled);
                Ok(v)
            },
        );
        (
            res,
            text(move || (res.state().value().map(ToString::to_string)).unwrap_or_default()),
        )
    });
    screen.find_by_text("42");
    assert_eq!(res.state(), AsyncState::Ready(42));
}

#[test]
fn multiple_file_picker_preserves_paths_cancellation_and_platform_errors() {
    let (screen, result) = mount(|| {
        let result = signal(None);
        (
            result,
            {
                let label = ("Pick attachments").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || {
                spawn(async move {
                    result.set(Some(platform::pick_files(Default::default()).await));
                });
            }),
        )
    });
    let button = screen.get_by_role(Role::Button, "Pick attachments");
    let paths = vec!["C:/fixtures/a.pdf".into(), "C:/fixtures/日本語.png".into()];
    screen.respond_files_dialog(Some(paths.clone()));
    button.click();
    assert_eq!(result.get(), Some(Ok(Some(paths.clone()))));
    screen.respond_files_dialog(None);
    button.click();
    assert_eq!(result.get(), Some(Ok(None)));
    screen.respond_files_dialog(Some(paths.clone()));
    screen.fail_next_platform_request("picker unavailable");
    button.click();
    let error = result
        .get()
        .expect("completed picker")
        .expect_err("platform failed");
    assert_eq!(*error.kind(), ErrorKind::Platform);
    assert!(error.to_string().contains("picker unavailable"));
    button.click();
    assert_eq!(
        result.get(),
        Some(Ok(Some(paths))),
        "failure keeps the queued selection"
    );
    assert!(screen.errors().is_empty());
}

#[test]
fn platform_requests_are_futures_and_spawned_failures_reach_on_error() {
    let got = Rc::new(RefCell::new(Vec::new()));
    let g2 = got.clone();
    let (screen, ()) = mount(move || {
        let g3 = g2.clone();
        (
            (),
            column((
                {
                    let label = ("copy").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    spawn(platform::copy_text("copied!"));
                }),
                {
                    let label = ("paste").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .on_press(move || {
                    let g4 = g3.clone();
                    spawn(async move {
                        let t = platform::read_text().await;
                        let f = platform::pick_file(kivact::platform::FileDialog::default()).await;
                        g4.borrow_mut().push(format!("{t:?} {f:?}"));
                    });
                }),
            )),
        )
    });
    screen.get_by_role(Role::Button, "copy").click();
    assert_eq!(screen.clipboard().as_deref(), Some("copied!"));
    screen.respond_file_dialog(Some("C:/data/a.txt".into()));
    screen.get_by_role(Role::Button, "paste").click();
    assert_eq!(
        *got.borrow(),
        vec![r#"Ok("copied!") Ok(Some("C:/data/a.txt"))"#]
    );
    screen.fail_next_platform_request("clipboard busy");
    screen.get_by_role(Role::Button, "copy").click();
    assert!(
        screen
            .errors()
            .iter()
            .any(|e| *e.kind() == ErrorKind::Platform && e.to_string().contains("busy"))
    );
}

#[test]
fn a_spawned_task_error_reaches_on_error() {
    let caught: Rc<RefCell<Vec<UiError>>> = Rc::default();
    let seen = caught.clone();
    let line = line!() + 5;
    let (screen, ()) = mount(move || {
        let view = error_boundary(|| {
            spawn(async {
                sleep(ms(5)).await;
                Err::<(), UiError>(UiError::msg("task failed"))
            });
            text("child")
        })
        .on_error(move |e| seen.borrow_mut().push(e));
        ((), view)
    });
    assert!(caught.borrow().is_empty());
    screen.advance(ms(10));
    let errors = caught.borrow().clone();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].message(), "task failed");
    assert_eq!(errors[0].location().map(|l| l.line()), Some(line));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn on_done_handlers_are_released_with_their_owner() {
    let token = Rc::new(());
    let t2 = token.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let t3 = t2.clone();
        (
            open,
            show(open, move || {
                let act = action(|n: u32| async move { Ok(n) });
                let (a, b) = (t3.clone(), t3.clone());
                act.on_done(move |_| drop(a.clone()));
                act.on_done(move |_| drop(b.clone()));
                let q = action(|_: u32| async { Ok(()) }).policy(ActionPolicy::Queue);
                let c = t3.clone();
                q.on_done(move |_| drop(c.clone()));
                text("owner")
            }),
        )
    });
    assert_eq!(Rc::strong_count(&token), 5);
    open.set(false);
    screen.run_until_idle();
    assert_eq!(
        Rc::strong_count(&token),
        2,
        "only the show closure keeps its copy"
    );
}

/// `cancel` after `Ready` is a no-op — the value
/// stays, nothing is published and completion handlers do not run again.
#[test]
fn cancel_after_ready_keeps_the_result_and_publishes_nothing() {
    let (screen, (res, act, done)) = mount(move || {
        let res = resource(
            || (),
            |_: ()| async move {
                sleep(ms(10)).await;
                Ok(7u32)
            },
        );
        let act = action(|x: u32| async move { Ok(x + 1) });
        let done = signal(0u32);
        act.on_done(move |_| done.update(|d| *d += 1));
        (
            (res, act, done),
            text(move || format!("{:?} {:?}", res.state(), act.state())),
        )
    });
    screen.advance(ms(20));
    act.run(1);
    screen.run_until_idle();
    assert_eq!(res.state(), AsyncState::Ready(7));
    assert_eq!(act.state(), AsyncState::Ready(2));
    assert_eq!(done.get_untracked(), 1);
    let before = screen.stats();
    res.cancel();
    act.cancel();
    screen.advance(ms(50));
    assert_eq!(res.state(), AsyncState::Ready(7), "Ready survives cancel");
    assert_eq!(act.state(), AsyncState::Ready(2));
    assert_eq!(done.get_untracked(), 1, "no second completion");
    assert_eq!(screen.stats().batches, before.batches, "nothing published");
}

// ---------------------------------------------------------------------------
// Regression (design survey): registrations made by effect cleanups during
// disposal leak. The disposal releases them.

#[test]
fn spawn_in_an_effect_cleanup_during_unmount_does_not_leak() {
    let ticks = Rc::new(Cell::new(0u32));
    let t2 = ticks.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        let v = show(open, move || {
            let t3 = t2.clone();
            effect(move || {
                let t4 = t3.clone();
                on_cleanup(move || {
                    let t5 = t4.clone();
                    spawn::<()>(async move {
                        loop {
                            kivact::task::sleep(Duration::from_millis(10)).await;
                            t5.set(t5.get() + 1);
                        }
                    });
                });
            });
            text("child")
        });
        (open, v)
    });
    let before = screen.stats();
    open.set(false);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    let after = screen.stats();
    assert_eq!(
        ticks.get(),
        0,
        "a task spawned while disposing keeps running"
    );
    // (The survey version asserted `<`; the child owns no object, so the
    // leak check is "no object left behind".)
    assert!(
        after.live_objects <= before.live_objects,
        "objects {} -> {}",
        before.live_objects,
        after.live_objects
    );
    // The disposal releases the task (G-01): nothing to report.
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------------------
// Regression (design survey): side-effecting methods called from a pure context change internal
// state before the write is rejected (stuck states).

#[test]
fn action_cancel_in_a_memo_does_not_leave_the_action_stuck() {
    let (screen, (act, arm)) = mount(|| {
        let act = action(|v: u32| async move {
            kivact::task::sleep(Duration::from_millis(50)).await;
            Ok(v)
        });
        let arm = signal(false);
        let m = memo(move || {
            if arm.get() {
                act.cancel();
            }
            0
        });
        (
            (act, arm),
            text(move || format!("{} {}", m.get(), act.state().is_pending())),
        )
    });
    act.run(1);
    screen.run_until_idle();
    assert!(act.state().is_pending());
    arm.set(true);
    screen.run_until_idle();
    assert!(kinds(&screen).contains(&ErrorKind::WriteInPureContext));
    screen.advance(Duration::from_millis(200));
    // Either the cancel is rejected entirely (run completes) or it fully applies.
    assert!(
        !act.state().is_pending(),
        "state stuck at Pending: {:?}",
        act.state()
    );
}

#[test]
fn resource_cancel_in_a_memo_does_not_leave_the_resource_loading() {
    let (screen, (res, arm)) = mount(|| {
        let res = resource(
            || (),
            |()| async move {
                kivact::task::sleep(Duration::from_millis(50)).await;
                Ok(7)
            },
        );
        let arm = signal(false);
        let m = memo(move || {
            if arm.get() {
                res.cancel();
            }
            0
        });
        (
            (res, arm),
            text(move || format!("{} {}", m.get(), res.state().is_pending())),
        )
    });
    assert!(res.state().is_pending());
    arm.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(200));
    assert!(!res.state().is_pending(), "stuck: {:?}", res.state());
}

// ---------------------------------------------------------------------------
// Regression (design survey): on_done registered by a child on a parent's action is not released
// when the child is disposed.

#[test]
fn on_done_registered_by_a_disposed_child_is_released() {
    let calls = Rc::new(Cell::new(0));
    let c2 = calls.clone();
    let (screen, (act, open)) = mount(move || {
        let act = action(|v: u32| async move { Ok(v) });
        let open = signal(true);
        let c3 = c2.clone();
        let v = show(open, move || {
            let mine = signal(0);
            let c4 = c3.clone();
            act.on_done(move |_| {
                c4.set(c4.get() + 1);
                mine.update(|m| *m += 1); // child's own handle
            });
            text("child")
        });
        ((act, open), v)
    });
    act.run(1);
    screen.run_until_idle();
    assert_eq!(calls.get(), 1);
    open.set(false);
    screen.run_until_idle();
    act.run(2);
    screen.run_until_idle();
    assert_eq!(
        calls.get(),
        1,
        "the child's handler still runs after the child is gone"
    );
    assert!(
        !kinds(&screen)
            .iter()
            .any(|k| matches!(k, ErrorKind::Disposed { .. })),
        "{:?}",
        kinds(&screen)
    );
}

#[test]
fn on_done_registered_in_a_handler_does_not_accumulate() {
    let calls = Rc::new(Cell::new(0));
    let c2 = calls.clone();
    let (screen, _act) = mount(move || {
        let act = action(|v: u32| async move { Ok(v) });
        let c3 = c2.clone();
        (
            act,
            {
                let label = ("go").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || {
                let c4 = c3.clone();
                act.on_done(move |_| c4.set(c4.get() + 1));
                act.run(1);
            }),
        )
    });
    for _ in 0..3 {
        screen.get_by_role(Role::Button, "go").click();
    }
    // 1 + 2 + 3 handler runs if registrations accumulate; the registration
    // API has no removal path (every registration pairs with a release).
    assert!(calls.get() <= 3, "handler runs: {}", calls.get());
}

#[test]
fn platform_request_is_rejected_in_a_memo() {
    let (screen, arm) = mount(|| {
        let arm = signal(false);
        let m = memo(move || {
            if arm.get() {
                drop(platform::copy_text("from a memo"));
            }
            0
        });
        (arm, text(move || m.get().to_string()))
    });
    arm.set(true);
    screen.run_until_idle();
    assert_eq!(screen.clipboard(), None, "a memo wrote the OS clipboard");
    assert!(kinds(&screen).contains(&ErrorKind::WriteInPureContext));
}

// ---------------------------------------------------------------------------
// Regression (design survey): a synchronous panic in an action / resource closure leaves the state
// Pending / Loading forever.

#[test]
fn action_closure_panic_does_not_leave_it_pending() {
    let (screen, act) = mount(|| {
        let act = action(|v: u32| {
            if v == 1 {
                panic!("sync failure");
            }
            async move { Ok(v) }
        });
        (
            act,
            {
                let label = ("go").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || act.run(1)),
        )
    });
    screen.get_by_role(Role::Button, "go").click();
    screen.advance(Duration::from_millis(100));
    assert!(!act.state().is_pending(), "stuck: {:?}", act.state());
}

#[test]
fn resource_fetcher_panic_does_not_leave_it_loading() {
    let (screen, res) = mount(|| {
        let res = resource(
            || 1u32,
            |v: u32| {
                if v == 1 {
                    panic!("sync failure");
                }
                async move { Ok(v) }
            },
        );
        (res, text(move || res.state().is_pending().to_string()))
    });
    screen.advance(Duration::from_millis(100));
    assert!(!res.state().is_pending(), "stuck: {:?}", res.state());
}

// ---------------------------------------------------------------------------
// Regression (design survey): a synchronous panic in a queue closure escapes the turn (it
// could cross a host's platform FFI boundary and abort the process).

#[test]
fn queue_closure_panic_is_caught_at_a_boundary() {
    let (screen, q) = mount(|| {
        let q = action(|v: u32| {
            if v == 1 {
                panic!("sync failure");
            }
            async move { Ok(()) }
        })
        .policy(ActionPolicy::Queue);
        (
            q,
            {
                let label = ("go").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(move || q.run(1)),
        )
    });
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        screen.get_by_role(Role::Button, "go").click();
    }));
    assert!(r.is_ok(), "the panic escaped EngineClient::turn");
    assert!(
        matches!(q.state(), AsyncState::Failed(_)),
        "{:?}",
        q.state()
    );
}
