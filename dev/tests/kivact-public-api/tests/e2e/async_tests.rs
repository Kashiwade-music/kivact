//! Role: Tests for async state and resource handling.
//!
//! Async state against a model: Action request → cancel → retry and
//! Resource source change / refetch / cancel, driven by random probes (from
//! the adversarial design survey) and each checked against a model.

#![allow(
    clippy::type_complexity,
    reason = "the probes stash handles of their setup in shared cells"
)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use kivact::prelude::*;
use kivact::test::render;

use crate::support::random::{Rng, residue, seeds};

// ---------------------------------------------------------------------------
// Request → cancel → retry (Action), against a model.

#[derive(Clone, Debug, PartialEq)]
enum Model {
    Idle,
    Pending(u32, u64),
    Done(u32),
    Cancelled(u32),
}

#[test]
fn action_request_cancel_retry_matches_the_model() {
    for seed in seeds() {
        let mut rng = Rng::new(seed);
        let done_log = Rc::new(RefCell::new(Vec::<u32>::new()));
        let d2 = done_log.clone();
        let slot: Rc<Cell<Option<Action<u32, u32>>>> = Rc::new(Cell::new(None));
        let s2 = slot.clone();
        let screen = render(move || {
            let act = action(ActionMode::Latest, |i: u32| async move {
                kivact::task::sleep(Duration::from_millis(10 * (i as u64 % 5 + 1))).await;
                if i % 7 == 3 {
                    Err(UiError::msg("odd failure"))
                } else {
                    Ok(i * 10)
                }
            });
            let act = act.on_complete(move |_| {
                if let Some(input) = act.input() {
                    d2.borrow_mut().push(input);
                }
            });
            s2.set(Some(act));
            text(move || format!("{:?}", act.state()))
        });
        let act = slot.get().expect("setup");
        let baseline = residue(&screen);
        let mut model = Model::Idle;
        let mut last_input: Option<u32> = None;
        let mut expected_done = Vec::new();
        let mut now = 0u64; // ms since start of the sequence
        for step in 0..50 {
            let ctx = format!("seed {seed} step {step}");
            match rng.below(4) {
                0 => {
                    let i = rng.below(20) as u32;
                    act.run(i);
                    last_input = Some(i);
                    model = Model::Pending(i, now + 10 * (i as u64 % 5 + 1));
                }
                1 => {
                    act.cancel();
                    if let Model::Pending(i, _) = model {
                        model = Model::Cancelled(i);
                    }
                }
                2 => {
                    act.retry();
                    if let Some(i) = last_input {
                        model = Model::Pending(i, now + 10 * (i as u64 % 5 + 1));
                    }
                }
                _ => {
                    let dt = rng.below(40) as u64;
                    now += dt;
                    screen.advance(Duration::from_millis(dt));
                    if let Model::Pending(i, at) = model
                        && at <= now
                    {
                        model = Model::Done(i);
                        expected_done.push(i);
                    }
                }
            }
            screen.run_until_idle();
            let input = act.input().unwrap_or(u32::MAX);
            let got = match act.state() {
                AsyncState::Idle => Model::Idle,
                AsyncState::Pending => match model {
                    Model::Pending(_, at) => Model::Pending(input, at),
                    _ => Model::Pending(input, 0),
                },
                AsyncState::Failed(e) if e.is_cancelled() => Model::Cancelled(input),
                AsyncState::Ready(_) | AsyncState::Failed(_) => Model::Done(input),
            };
            assert_eq!(got, model, "{ctx}");
            assert_eq!(
                act.state().is_pending(),
                matches!(model, Model::Pending(..)),
                "{ctx}"
            );
            assert_eq!(act.input(), last_input, "{ctx}: input");
            assert_eq!(
                *done_log.borrow(),
                expected_done,
                "{ctx}: on_done publishes"
            );
        }
        act.cancel();
        screen.advance(Duration::from_millis(500));
        assert_eq!(
            residue(&screen),
            baseline,
            "seed {seed}: in-flight task or timer left"
        );
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RModel {
    Loading(u32, u64),
    Ready(u32),
    Failed,
    Cancelled,
}

#[test]
fn p2b_resource_source_change_refetch_cancel_matches_the_model() {
    for seed in seeds() {
        let mut rng = Rng::new(seed);
        let slot: Rc<Cell<Option<(Signal<u32>, Resource<u32>)>>> = Rc::new(Cell::new(None));
        let s2 = slot.clone();
        let screen = render(move || {
            let q = signal(1u32);
            let res = resource(
                move || q.get(),
                |v: u32| async move {
                    kivact::task::sleep(Duration::from_millis(10 * (v as u64 % 4 + 1))).await;
                    if v % 5 == 4 {
                        Err(UiError::msg("bad"))
                    } else {
                        Ok(v * 10)
                    }
                },
            );
            s2.set(Some((q, res)));
            text(move || format!("{:?} {:?}", res.state(), res.latest()))
        });
        let (q, res) = slot.get().expect("setup");
        let delay = |v: u32| 10 * (v as u64 % 4 + 1);
        let mut now = 0u64;
        let mut model = RModel::Loading(1, delay(1));
        let mut cur_q = 1u32;
        let mut last_ok: Option<u32> = None;
        for step in 0..50 {
            let ctx = format!("seed {seed} step {step}");
            match rng.below(4) {
                0 => {
                    let v = rng.below(10) as u32;
                    q.set(v);
                    if v != cur_q {
                        cur_q = v;
                        model = RModel::Loading(v, now + delay(v));
                    }
                }
                1 => {
                    res.refetch();
                    model = RModel::Loading(cur_q, now + delay(cur_q));
                }
                2 => {
                    res.cancel();
                    if matches!(model, RModel::Loading(..)) {
                        model = RModel::Cancelled;
                    }
                }
                _ => {
                    let dt = rng.below(40) as u64;
                    now += dt;
                    screen.advance(Duration::from_millis(dt));
                    if let RModel::Loading(v, at) = model
                        && at <= now
                    {
                        if v % 5 == 4 {
                            model = RModel::Failed;
                        } else {
                            model = RModel::Ready(v * 10);
                            last_ok = Some(v * 10);
                        }
                    }
                }
            }
            screen.run_until_idle();
            let got = match res.state() {
                AsyncState::Pending => match model {
                    RModel::Loading(v, at) => RModel::Loading(v, at),
                    _ => RModel::Loading(u32::MAX, 0),
                },
                AsyncState::Ready(v) => RModel::Ready(v),
                AsyncState::Failed(e) if e.is_cancelled() => RModel::Cancelled,
                AsyncState::Failed(_) => RModel::Failed,
                AsyncState::Idle => RModel::Loading(u32::MAX, 1),
            };
            assert_eq!(got, model, "{ctx}");
            assert_eq!(res.latest(), last_ok, "{ctx}: latest");
            assert_eq!(
                res.state().is_pending(),
                matches!(model, RModel::Loading(..)),
                "{ctx}"
            );
        }
    }
}

#[test]
fn async_view_shows_loading_then_ready() {
    #[derive(Clone)]
    struct Record(String);
    let screen = render(|| {
        let value = resource((), |_| async {
            kivact::task::sleep(Duration::from_millis(10)).await;
            Ok(Record(String::from("Loaded")))
        });
        async_view(value, |record| text(record.0)).loading(|| text("Please wait"))
    });
    screen.get_by_text("Please wait");
    screen.advance(Duration::from_millis(10));
    screen.get_by_text("Loaded");
}

#[test]
fn async_view_fallback_retries() {
    let screen = render(|| {
        let attempts = Rc::new(Cell::new(0));
        let load = action(ActionMode::Latest, move || {
            let n = attempts.get();
            attempts.set(n + 1);
            async move {
                if n == 0 {
                    Err(UiError::msg("Offline"))
                } else {
                    Ok("Recovered")
                }
            }
        });
        column((
            button("Load").on_press(move || load.run(())),
            async_view(load, text),
        ))
    });
    assert!(
        screen.query_by_text("Loading…").is_none(),
        "an idle action has no loading view"
    );
    screen.get_by_role(Role::Button, "Load").click();
    screen.get_by_text("Error: Offline");
    screen.get_by_role(Role::Button, "Retry").click();
    screen.get_by_text("Recovered");
}

#[cfg(feature = "tokio")]
#[test]
fn on_tokio_future_resolves_and_wakes_the_ui() {
    let (send, receive) = async_channel::bounded(1);
    let receive = Rc::new(RefCell::new(Some(receive)));
    let screen = render(move || {
        let receiver = receive.borrow_mut().take().expect("setup runs once");
        let value = resource((), move |_| {
            let receiver = receiver.clone();
            async move {
                Ok(kivact::task::on_tokio(
                    async move { receiver.recv().await.expect("sender lives") },
                )
                .await)
            }
        });
        async_view(value, text)
    });
    screen.get_by_text("Loading…");
    send.send_blocking(String::from("Worker result"))
        .expect("receiver lives");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        screen.run_until_idle();
        if screen.query_by_text("Worker result").is_some() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Tokio completion must wake the UI"
        );
        std::thread::yield_now();
    }
}

#[cfg(feature = "tokio")]
#[test]
fn on_tokio_task_is_aborted_with_its_owner() {
    struct Dropped(std::sync::mpsc::Sender<()>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            let _ = self.0.send(());
        }
    }
    let (started, observed_start) = std::sync::mpsc::channel();
    let (dropped, observed_drop) = std::sync::mpsc::channel();
    let screen = render(move || {
        let visible = signal(true);
        column((
            button("Remove").on_press(move || visible.set(false)),
            show(visible, move || {
                let started = started.clone();
                let dropped = dropped.clone();
                spawn(async move {
                    kivact::task::on_tokio(async move {
                        let _guard = Dropped(dropped);
                        started.send(()).expect("test waits for start");
                        std::future::pending::<()>().await;
                    })
                    .await;
                });
                text("Owner")
            }),
        ))
    });
    observed_start
        .recv_timeout(Duration::from_secs(5))
        .expect("task starts");
    screen.get_by_role(Role::Button, "Remove").click();
    observed_drop
        .recv_timeout(Duration::from_secs(5))
        .expect("owner disposal aborts task");
    assert!(screen.query_by_text("Owner").is_none());
}
