//! Role: Async: executor, pool, resource, action, queue, time, platform.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use kivact::prelude::*;
use kivact::task::{CancelToken, Remote, clock, queued_action, remote, remote_trigger, sleep};
use kivact_public_api_tests::support::mount;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
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
#[should_panic(expected = "worker exploded")]
fn a_blocking_panic_resumes_on_the_ui_thread() {
    let (screen, _res) = mount(|| {
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
    screen.run_until_idle();
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
        let act = act.on_complete(move |r| {
            let result = r;
            d3.borrow_mut().push(format!("{result:?}"));
        });
        let act = act.on_complete(move |_| {
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
fn queue_orders_recovers_and_drains() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l2 = log.clone();
    let (screen, fifo) = mount(move || {
        let la = l2.clone();
        let fifo = queued_action(move |n: u32| {
            let la = la.clone();
            async move {
                sleep(ms(10)).await;
                if n == 2 {
                    return Err(UiError::msg("two failed"));
                }
                la.borrow_mut().push(format!("fifo {n}"));
                Ok(())
            }
        });
        let results = var(Vec::<bool>::new());
        let fifo = fifo.on_complete(move |r| results.update(|v| v.push(r.is_ok())));
        (
            fifo,
            text(move || format!("{:?}", fifo.state())).test_id("status"),
        )
    });
    fifo.run(1);
    screen.run_until_idle(); // run 1 is now running
    for n in 2..=4 {
        fifo.run(n);
    }
    screen.run_until_idle();
    assert_eq!(fifo.pending_len(), 3, "first run running, three waiting");
    assert_eq!(fifo.state(), AsyncState::Pending);
    screen.advance(ms(15));
    screen.advance(ms(10)); // t = 25: run 2 failed at 20, run 3 running
    assert_eq!(
        fifo.state(),
        AsyncState::Pending,
        "queued work remains pending after a failed run"
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

/// Writes from another thread wake the App and arrive merged: the signal
/// is written once, with the latest value.
#[test]
fn remote_writes_from_another_thread_coalesce_into_one_update() {
    let (screen, (sender, runs)) = mount(|| {
        let (value, sender) = remote(0u32);
        let runs = var(0u32);
        effect(move || {
            value.get();
            runs.update(|r| *r += 1);
        });
        ((sender, runs), text(move || format!("v {}", value.get())))
    });
    screen.find_by_text("v 0");
    let before = runs.get();
    let worker = sender.clone();
    std::thread::spawn(move || {
        for n in 1..=100 {
            worker.set(n);
        }
    })
    .join()
    .expect("the writing thread ends");
    screen.find_by_text("v 100");
    assert_eq!(
        runs.get(),
        before + 1,
        "a hundred writes arrive as one update"
    );
}

/// After the component of a remote is removed, values set from another
/// thread are dropped at once, not kept for a delivery that never comes.
#[test]
fn a_remote_of_a_removed_component_drops_later_values() {
    let slot: Rc<RefCell<Option<Remote<Arc<u32>>>>> = Rc::new(RefCell::new(None));
    let child_slot = slot.clone();
    let (screen, shown) = mount(move || {
        let shown = signal(true);
        let child_slot = child_slot.clone();
        (
            shown,
            show(shown, move || {
                let (value, sender) = remote(Arc::new(0u32));
                *child_slot.borrow_mut() = Some(sender);
                text(move || format!("v {}", value.get()))
            }),
        )
    });
    screen.find_by_text("v 0");
    let sender = slot.borrow_mut().take().expect("the child was set up");
    shown.set(false);
    screen.run_until_idle();
    let probe = Arc::new(7u32);
    let sent = Arc::clone(&probe);
    std::thread::spawn(move || sender.set(sent))
        .join()
        .expect("the writing thread ends");
    assert_eq!(
        Arc::strong_count(&probe),
        1,
        "the closed mailbox kept no value"
    );
    screen.run_until_idle();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A producer that writes while the App turns does not flood it: each
/// turn writes the signal at most once, and the last value arrives.
#[test]
fn remote_delivers_at_most_once_per_turn_under_a_continuous_producer() {
    const WRITES: u64 = 200_000;
    let (screen, (sender, runs)) = mount(|| {
        let (value, sender) = remote(0u64);
        let runs = var(0u64);
        effect(move || {
            value.get();
            runs.update(|r| *r += 1);
        });
        ((sender, runs), text(move || format!("v {}", value.get())))
    });
    screen.run_until_idle();
    let (runs_before, turns_before) = (runs.get(), screen.stats().turns);
    let producer = std::thread::spawn(move || {
        for n in 1..=WRITES {
            sender.set(n);
        }
    });
    while !producer.is_finished() {
        screen.run_until_idle();
    }
    producer.join().expect("the producer ends");
    screen.find_by_text(&format!("v {WRITES}"));
    let runs = runs.get() - runs_before;
    let turns = screen.stats().turns - turns_before;
    assert!(runs > 0, "values arrived");
    assert!(
        runs <= turns,
        "at most one delivery per turn: {runs} deliveries in {turns} turns"
    );
    assert!(runs < WRITES, "writes between turns were merged");
}

/// A trigger notified from another thread wakes the App; notifications
/// before a turn are one.
#[test]
fn remote_trigger_notifies_from_another_thread() {
    let (screen, (sender, count)) = mount(|| {
        let (changed, sender) = remote_trigger();
        let count = signal(0u32);
        effect(move || {
            changed.track();
            count.update(|c| *c += 1);
        });
        ((sender, count), text(move || format!("n {}", count.get())))
    });
    screen.find_by_text("n 1");
    let worker = sender.clone();
    std::thread::spawn(move || {
        worker.notify();
        worker.notify();
    })
    .join()
    .expect("the notifying thread ends");
    screen.find_by_text("n 2");
    screen.run_until_idle();
    assert_eq!(count.get_untracked(), 2);
}

/// Clocks of one period share one value and one timer, which ticks once per period.
#[test]
fn clock_shares_one_timer_per_period() {
    let (screen, (a, b, c, ticks)) = mount(|| {
        let a = clock(ms(1000));
        let b = clock(ms(1000));
        let c = clock(ms(500));
        let ticks = var(0u32);
        effect(move || {
            a.get();
            ticks.update(|t| *t += 1);
        });
        ((a, b, c, ticks), ())
    });
    screen.run_until_idle();
    assert_eq!(a, b, "one value for one period");
    assert_ne!(a, c);
    let before = ticks.get();
    for _ in 0..3 {
        screen.advance(ms(1000));
    }
    assert_eq!(ticks.get(), before + 3);
}

/// Removing the component of a queued action cancels its runs, the
/// waiting ones included: none of them finishes later.
#[test]
fn disposing_the_owner_cancels_its_queued_runs() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let slot: Rc<Cell<Option<Action<(), u32>>>> = Rc::new(Cell::new(None));
    let (l2, s2) = (log.clone(), slot.clone());
    let (screen, shown) = mount(move || {
        let shown = signal(true);
        let (l2, s2) = (l2.clone(), s2.clone());
        (
            shown,
            show(shown, move || {
                let l3 = l2.clone();
                let queue = queued_action(move |n: u32| {
                    let l4 = l3.clone();
                    async move {
                        sleep(ms(10)).await;
                        l4.borrow_mut().push(n);
                        Ok::<(), UiError>(())
                    }
                });
                s2.set(Some(queue));
                text("queue")
            }),
        )
    });
    let queue = slot.get().expect("the branch was set up");
    for n in 1..=3 {
        queue.run(n);
    }
    screen.run_until_idle();
    assert_eq!(queue.pending_len(), 2, "one run running, two waiting");
    shown.set(false);
    screen.run_until_idle();
    screen.advance(ms(100));
    assert!(
        log.borrow().is_empty(),
        "no run finished after its owner was removed: {:?}",
        log.borrow()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
