//! Role: Async: executor, blocking pool, resource, action, queue, time, platform.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::prelude::*;
use kivact::task::spawn;
use kivact::task::{CancelToken, sleep};
use kivact::view::error_boundary;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// `AsyncState` answers `is_ready`, `is_failed`, `value` and `error` in each
/// state, and an action and a resource read the error directly.
#[test]
fn async_state_helpers_and_direct_errors_follow_each_state() {
    let (screen, (act, res)) = mount(|| {
        let act = action(ActionPolicy::Latest, |n: u32| async move {
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
        let act = action(ActionPolicy::Latest, move |delay: u64| async move {
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
fn queue_orders_coalesces_recovers_and_drains() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let l2 = log.clone();
    let (screen, (fifo, latest)) = mount(move || {
        let (la, lb) = (l2.clone(), l2.clone());
        let fifo = action(ActionPolicy::Queue, move |n: u32| {
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
        let latest = action(ActionPolicy::QueueLatest, move |n: u32| {
            let lb = lb.clone();
            async move {
                sleep(ms(10)).await;
                lb.borrow_mut().push(format!("latest {n}"));
                Ok(())
            }
        });
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

// ---------------------------------------------------------------------------
// Regression (design survey): registrations made by effect cleanups during
// disposal leak. The disposal releases them.

// ---------------------------------------------------------------------------
// Regression (design survey): side-effecting methods called from a pure context change internal
// state before the write is rejected (stuck states).

// ---------------------------------------------------------------------------
// Regression (design survey): on_done registered by a child on a parent's action is not released
// when the child is disposed.

// ---------------------------------------------------------------------------
// Regression (design survey): a synchronous panic in an action / resource closure leaves the state
// Pending / Loading forever.

// ---------------------------------------------------------------------------
// Regression (design survey): a synchronous panic in a queue closure escapes the turn (it
// could cross a host's platform FFI boundary and abort the process).
