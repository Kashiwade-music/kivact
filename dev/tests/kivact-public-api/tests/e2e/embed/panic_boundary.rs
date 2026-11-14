//! Role: Real-window checks that a panic in an embedded App stops it without
//! aborting the host process.

use super::support::{SERIAL, painted, panel, wait};
use kivact::App;
use kivact::embed::EmbedNotice;
use kivact::embed::EmbedStatus;
use kivact::prelude::*;
use kivact::test::TestParent;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Set in the child process; names the scenario it runs.
const CHILD_ENV: &str = "KIVACT_EMBED_PANIC_CHILD";
const CHILD_TEST: &str = "embed::panic_boundary::child_process_scenarios_do_not_abort";

/// Run every panic scenario in a child process of this test binary: an
/// escaping panic would abort it (the process behind a C ABI), which the
/// parent sees as a failed child.
#[test]
fn child_process_scenarios_do_not_abort() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }
    if let Ok(scenario) = std::env::var(CHILD_ENV) {
        run_scenario(&scenario);
        return;
    }
    let _serial = SERIAL
        .lock()
        .expect("embed tests run one at a time; an earlier one panicked holding SERIAL");
    for scenario in ["root", "event", "cleanup", "payload"] {
        let status = std::process::Command::new(
            std::env::current_exe().expect("the test binary has a path"),
        )
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env(CHILD_ENV, scenario)
        .status()
        .expect("the child test process starts");
        assert!(
            status.success(),
            "the child aborted or failed for the {scenario} panic: {status:?}"
        );
    }
}

fn run_scenario(scenario: &str) {
    match scenario {
        "root" => root_panic(),
        "event" => event_panic(),
        "cleanup" => cleanup_panic(),
        "payload" => payload_drop(),
        other => panic!("unknown panic scenario {other}"),
    }
}

/// A panic while the root is built: `embed` returns `ErrorKind::Panicked` and
/// leaves no window.
fn root_panic() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let result = App::new("Root panic").window_size(320, 240).embed(
        parent.parent().scale(1.0),
        || -> Node {
            panic!("root exploded");
        },
    );
    let error = result.err().expect("the root panic is an error");
    assert_eq!(error.kind(), &kivact::ErrorKind::Panicked);
    assert!(parent.capture().is_err(), "a failed open leaves no window");
}

/// A panic in an event handler: reported as a value, the window closes, and
/// no user code runs afterwards.
fn event_panic() {
    fn explode() {
        panic!("editor exploded");
    }
    let parent = TestParent::open(320, 240).expect("parent opens");
    let presses = Arc::new(AtomicUsize::new(0));
    let counter = presses.clone();
    let app = App::new("Event panic")
        .window_size(320, 240)
        .embed(parent.parent().scale(1.0), move || {
            panel().focusable(true).on_press(move || {
                counter.fetch_add(1, Ordering::SeqCst);
                explode();
            })
        })
        .expect("editor opens");
    painted(&parent);
    parent.click(20.0, 20.0).expect("click delivered");
    wait(&parent, || matches!(app.status(), EmbedStatus::Panicked(_)));
    let EmbedStatus::Panicked(error) = app.status() else {
        panic!("status reports the panic");
    };
    assert_eq!(error.to_string(), "editor exploded");
    wait(&parent, || parent.capture().is_err());
    // A click after the window closed is refused by the test parent.
    let _ = parent.click(20.0, 20.0);
    parent.pump(Duration::from_millis(50));
    assert_eq!(
        presses.load(Ordering::SeqCst),
        1,
        "no user code runs after the panic"
    );
    drop(app);
}

/// A panic in an App cleanup while the host closes the editor: the drop
/// returns normally and the host is told that the App stopped.
fn cleanup_panic() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let stopped = Arc::new(AtomicUsize::new(0));
    let noticed = stopped.clone();
    let app = App::new("Cleanup panic")
        .window_size(160, 100)
        .embed(
            parent.parent().scale(1.0).on_notice(move |notice| {
                if notice == EmbedNotice::Stopped {
                    noticed.fetch_add(1, Ordering::SeqCst);
                }
            }),
            move || {
                on_cleanup(|| panic!("cleanup exploded"));
                panel()
            },
        )
        .expect("editor opens");
    painted(&parent);
    drop(app);
    assert_eq!(
        stopped.load(Ordering::SeqCst),
        1,
        "the host is told that the App stopped"
    );
}

struct Explosive;

impl Drop for Explosive {
    fn drop(&mut self) {
        panic!("the panic payload was dropped");
    }
}

/// A panic payload whose `Drop` panics: the payload is forgotten, so its
/// `Drop` never runs and the message is the fixed sentence.
fn payload_drop() {
    fn explode_payload() {
        std::panic::panic_any(Explosive);
    }
    let parent = TestParent::open(320, 240).expect("parent opens");
    let app = App::new("Explosive payload")
        .window_size(320, 240)
        .embed(parent.parent().scale(1.0), || {
            panel().focusable(true).on_press(explode_payload)
        })
        .expect("editor opens");
    painted(&parent);
    parent.click(20.0, 20.0).expect("click delivered");
    wait(&parent, || matches!(app.status(), EmbedStatus::Panicked(_)));
    let EmbedStatus::Panicked(error) = app.status() else {
        panic!("status reports the panic");
    };
    assert_eq!(
        error.to_string(),
        "the embedded App panicked with a non-string payload"
    );
    wait(&parent, || parent.capture().is_err());
    drop(app);
}
