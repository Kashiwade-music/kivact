//! Role: Isolated real-window checks of App-owned blocking thread cleanup.
//!
//! Run with `--test-threads=1`; the lock also protects default parallel runs.

#[path = "embed_resources/support.rs"]
mod embed_resources;

use std::sync::{Mutex, mpsc};
use std::time::{Duration, Instant};

use kivact::prelude::*;
use kivact::task::{CancelToken, blocking};
use kivact::test::{RenderOptions, WindowMode, render_with_handle};
use kivact::view::callback;

static RESOURCE_TEST: Mutex<()> = Mutex::new(());

fn open() -> (kivact::test::Screen, kivact::Callback<()>) {
    render_with_handle(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::default()
        },
        || (callback(|()| {}), text("resource warmup")),
    )
}

#[test]
fn closing_joins_blocking_workers() {
    let _serial = RESOURCE_TEST.lock().unwrap();
    drop(open());
    let screen = kivact::test::render_with(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::default()
        },
        || {
            kivact::task::spawn(async {
                blocking(|| 42).await;
            });
            text("blocking completes")
        },
    );
    assert!(
        !embed_resources::blocking_threads().is_empty(),
        "pool was created"
    );
    drop(screen);
    assert!(embed_resources::blocking_threads().is_empty());
}

#[test]
fn uncancellable_job_detaches_after_the_shutdown_deadline_with_warning() {
    let _serial = RESOURCE_TEST.lock().unwrap();
    #[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
    {
        const CHILD_ENV: &str = "KIVACT_EMBED_RESOURCE_STARTUP_FAILURE_CHILD";
        const EXACT_CASE: &str =
            "uncancellable_job_detaches_after_the_shutdown_deadline_with_warning";
        if std::env::var(CHILD_ENV).as_deref() == Ok("root-panic") {
            // Isolate process-wide retention from normal cycles and the other
            // timeout scenario. This mode returns without spawning another child.
            let warnings = embed_resources::warnings();
            let parent = kivact::test::TestParent::open(320, 240).unwrap();
            // Native libraries may retain an executor for the process lifetime.
            // Compare with that warmed state instead of assuming no threads;
            // an unnamed Linux executor can inherit the UI thread's native name.
            let warmup = kivact::App::new("Startup cleanup warmup")
                .embed(parent.parent(), || text("warmup"))
                .unwrap();
            drop(warmup);
            let baseline_threads = embed_resources::owned_threads();
            let (started_tx, started_rx) = mpsc::channel();
            let (finished_tx, finished_rx) = mpsc::channel();
            let error = kivact::App::new("Startup failure retention")
                .embed(parent.parent(), move || -> Node {
                    drop(blocking(move |_: CancelToken| {
                        started_tx.send(()).unwrap();
                        std::thread::sleep(Duration::from_secs(3));
                        finished_tx.send(()).unwrap();
                    }));
                    // Only wait for our worker, never for the host GUI thread.
                    started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
                    panic!("startup failure original diagnostic")
                })
                .err()
                .expect("root panic must return its original error");
            assert_eq!(*error.kind(), kivact::diagnostics::ErrorKind::Panic);
            assert!(
                error
                    .message()
                    .contains("startup failure original diagnostic")
            );
            // Err may precede the end of bounded native cleanup. Observe the
            // eventual result rather than requiring immediate disappearance.
            finished_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let detached = warnings.lock().unwrap().iter().any(|message| {
                    message == "blocking jobs still running after the App closed; their threads are detached"
                });
                if detached
                    && embed_resources::owned_threads() == baseline_threads
                    && parent.capture().is_err()
                {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "startup cleanup did not finish or diagnose its detached worker: baseline={baseline_threads:?}, threads={:?}, warnings={:?}",
                    embed_resources::owned_threads(),
                    warnings.lock().unwrap()
                );
                parent.pump(Duration::from_millis(5));
            }
            assert!(embed_resources::blocking_threads().is_empty());
            // This observes retention (including permanent executable mapping),
            // not an actual DLL unload/reload experiment.
            assert_eq!(
                embed_resources::retention_warnings(&warnings.lock().unwrap()),
                1
            );
            return;
        }
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", EXACT_CASE, "--nocapture", "--test-threads=1"])
            .env(CHILD_ENV, "root-panic")
            // Avoid a driver initialization wait masking the blocking deadline.
            .env("ARVANE_RENDERER", "cpu")
            .status()
            .expect("startup cleanup subprocess runs");
        assert!(
            status.success(),
            "startup cleanup subprocess failed: {status}"
        );
    }
    drop(open());
    let warnings = embed_resources::warnings();
    let before = warnings.lock().unwrap().len();
    let (started_tx, started_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let launch = move || {
        let started_tx = started_tx.clone();
        let finished_tx = finished_tx.clone();
        // Creating blocking starts the job immediately. Deliberately ignore
        // cancellation, including dropping its returned future.
        drop(blocking(move |_: CancelToken| {
            started_tx.send(()).unwrap();
            std::thread::sleep(Duration::from_secs(3));
            finished_tx.send(()).unwrap();
        }));
    };
    #[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
    let parent = kivact::test::TestParent::open(320, 240).unwrap();
    #[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
    let screen = kivact::App::new("Retain detached worker")
        .embed(parent.parent(), move || {
            launch();
            text("uncancellable job")
        })
        .unwrap();
    #[cfg(not(all(feature = "embed", any(windows, target_os = "linux"))))]
    let (screen, start) = render_with_handle(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::default()
        },
        move || (callback(move |()| launch()), text("uncancellable job")),
    );
    // Mounting a Screen settles setup work. Start afterward so Drop, rather
    // than startup settlement, owns the wait for the unfinished job.
    #[cfg(not(all(feature = "embed", any(windows, target_os = "linux"))))]
    start.try_call(()).unwrap();
    started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let close = Instant::now();
    drop(screen);
    let elapsed = close.elapsed();
    assert!(elapsed >= Duration::from_millis(1900), "{elapsed:?}");
    assert!(elapsed < Duration::from_millis(2800), "{elapsed:?}");
    // X11's outer UI-thread deadline can expire just before the worker's
    // own shutdown deadline. Its cleanup and warning then finish asynchronously.
    let warning_deadline = Instant::now() + Duration::from_secs(1);
    let blocking_warning =
        "blocking jobs still running after the App closed; their threads are detached";
    while !warnings.lock().unwrap()[before..]
        .iter()
        .any(|message| message == blocking_warning)
    {
        assert!(
            Instant::now() < warning_deadline,
            "blocking detach was not diagnosed"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    let recorded = warnings.lock().unwrap()[before..].to_vec();
    assert_eq!(
        recorded
            .iter()
            .filter(|message| message.as_str() == blocking_warning)
            .count(),
        1
    );
    assert!(
        recorded.iter().all(|message| message == blocking_warning
            || message.contains("module code kept mapped until process exit")
            || message == "embedded X11 UI thread did not stop within two seconds; detaching"),
        "unexpected shutdown warnings: {recorded:?}"
    );
    #[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
    {
        assert_eq!(
            embed_resources::retention_warnings(&warnings.lock().unwrap()),
            1,
            "a detached worker keeps its module mapped (or a native library already retained it)"
        );
        let again = kivact::App::new("Retention is idempotent")
            .embed(parent.parent(), || text("second close"))
            .unwrap();
        drop(again);
        assert_eq!(
            embed_resources::retention_warnings(&warnings.lock().unwrap()),
            1,
            "a second retention request emits no additional warning"
        );
    }
    finished_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    while !embed_resources::blocking_threads().is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(embed_resources::blocking_threads().is_empty());
}

#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
#[path = "embed_resources/cycles.rs"]
mod cycles;
