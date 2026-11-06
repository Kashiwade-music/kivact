//! Role: OS-independent real-window embedding contracts.

use super::support::{SERIAL, painted, panel, wait};
use kivact::embed::Embedded;
use kivact::prelude::*;
use kivact::style::Color;
use kivact::test::TestParent;
use kivact::view::KeyCode;
use kivact::{App, platform};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn native_click_key_and_text_reach_the_editor() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let clicks = Arc::new(AtomicUsize::new(0));
    let observed = Arc::new(Mutex::new(String::new()));
    let (count, output) = (clicks.clone(), observed.clone());
    let roots = Arc::new(AtomicUsize::new(0));
    let root_calls = roots.clone();
    let app = App::new("Native input")
        .window_size(320, 240)
        .embed(parent.parent().scale(1.0), move || {
            root_calls.fetch_add(1, Ordering::SeqCst);
            let value = signal(String::new());
            effect(move || *output.lock().expect("test state lock") = value.get());
            column((
                panel().focusable(true).on_press(move || {
                    count.fetch_add(1, Ordering::SeqCst);
                }),
                node()
                    .style(style().size(160, 60))
                    .part(kivact::interaction::use_text_edit(value).control()),
            ))
        })
        .expect("editor opens");
    painted(&parent);
    parent.click(20.0, 20.0).expect("click delivered");
    let click_deadline = Instant::now() + Duration::from_secs(15);
    wait(&parent, || {
        let value = clicks.load(Ordering::SeqCst);
        assert!(
            value == 1 || Instant::now() < click_deadline,
            "click did not arrive once: count={value}, scale={}, error={:?}",
            app.scale(),
            app.error()
        );
        value == 1
    });
    parent.click(20.0, 120.0).expect("click delivered");
    parent.text("hello").expect("text delivered");
    let typing_deadline = Instant::now() + Duration::from_secs(15);
    wait(&parent, || {
        let value = observed.lock().expect("test state lock").clone();
        assert!(
            value == "hello" || Instant::now() < typing_deadline,
            "typed text did not arrive: value={value:?}, scale={}, error={:?}",
            app.scale(),
            app.error()
        );
        value == "hello"
    });
    parent.key(KeyCode::Backspace).expect("key delivered");
    wait(&parent, || {
        *observed.lock().expect("test state lock") == "hell"
    });
    assert_eq!(roots.load(Ordering::SeqCst), 1);
    assert!(app.error().is_none());
}

fn accepted_startup_resize_rounds_and_changes_capture_then_host_size_wins() {
    let parent = TestParent::open(640, 480).expect("parent opens");
    let opening = Arc::new(AtomicBool::new(false));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let result = Arc::new(Mutex::new(None));
    let (inside, calls, output) = (opening.clone(), requests.clone(), result.clone());
    let built = opening.clone();
    let app = App::new("Resize")
        .window_size(120, 80)
        .embed(
            parent.parent().scale(2.0).on_resize_request(move |w, h| {
                assert!(
                    inside.load(Ordering::SeqCst),
                    "callback ran before root finished"
                );
                calls.lock().expect("test state lock").push((w, h));
                true
            }),
            move || {
                let future = platform::request_window_size(150.25, 90.25);
                spawn(async move {
                    *output.lock().expect("test state lock") =
                        Some(future.await.map_err(|e| e.to_string()));
                    Ok::<(), kivact::UiError>(())
                });
                let root = node().style(style().size(500, 500).background(Color::WHITE));
                built.store(true, Ordering::SeqCst);
                root
            },
        )
        .expect("editor opens");
    wait(&parent, || {
        result.lock().expect("test state lock").is_some()
    });
    assert_eq!(*result.lock().expect("test state lock"), Some(Ok(())));
    assert_eq!(*requests.lock().expect("test state lock"), vec![(301, 181)]);
    wait(&parent, || {
        parent
            .capture()
            .is_ok_and(|s| (s.width, s.height) == (301, 181))
    });
    app.set_size(220, 140);
    wait(&parent, || app.size() == (220, 140));
    wait(&parent, || {
        parent
            .capture()
            .is_ok_and(|s| (s.width, s.height) == (220, 140))
    });
    assert!(app.error().is_none());
}

fn refusal_and_missing_callback_complete_with_their_errors() {
    for callback in [false, true] {
        let parent = TestParent::open(640, 480).expect("parent opens");
        let result = Arc::new(Mutex::new(None));
        let output = result.clone();
        let mut options = parent.parent().scale(1.0);
        if callback {
            options = options.on_resize_request(|_, _| false);
        }
        let app = App::new("Refused resize")
            .window_size(120, 80)
            .embed(options, move || {
                let future = platform::request_window_size(200, 160);
                spawn(async move {
                    *output.lock().expect("test state lock") = Some(
                        future
                            .await
                            .map_err(|e| (format!("{:?}", e.kind()), e.to_string())),
                    );
                    Ok::<(), kivact::UiError>(())
                });
                text("Resize")
            })
            .expect("editor opens");
        wait(&parent, || {
            result.lock().expect("test state lock").is_some()
        });
        let error = result
            .lock()
            .expect("test state lock")
            .clone()
            .expect("resize result")
            .expect_err("resize rejected");
        assert_eq!(error.0, if callback { "Platform" } else { "Unsupported" });
        if callback {
            assert!(error.1.contains("the host refused the size"));
        }
        assert_eq!(app.size(), (120, 80));
        assert!(app.error().is_none());
    }
}

fn correction_inside_accepting_resize_callback_remains_authoritative() {
    let parent = TestParent::open(640, 480).expect("parent opens");
    let held = Arc::new(Mutex::new(None::<kivact::embed::Embedded>));
    let host = held.clone();
    let result = Arc::new(Mutex::new(None));
    let output = result.clone();
    let app = App::new("Host correction")
        .window_size(120, 80)
        .embed(
            parent.parent().scale(1.0).on_resize_request(move |w, h| {
                assert_eq!((w, h), (300, 200));
                host.lock()
                    .expect("test state lock")
                    .as_ref()
                    .expect("editor stored")
                    .set_size(220, 140);
                true
            }),
            move || {
                node()
                    .style(style().size(500, 500).background(Color::WHITE))
                    .on_press(move || {
                        let future = platform::request_window_size(300, 200);
                        let output = output.clone();
                        spawn(async move {
                            *output.lock().expect("test state lock") =
                                Some(future.await.map_err(|e| e.to_string()));
                            Ok::<(), kivact::UiError>(())
                        });
                    })
            },
        )
        .expect("editor opens");
    *held.lock().expect("test state lock") = Some(app);
    wait(&parent, || {
        parent
            .capture()
            .is_ok_and(|s| s.pixel(20, 20) == Some(0xffffff))
    });
    parent.click(20.0, 20.0).expect("click delivered");
    wait(&parent, || {
        result.lock().expect("test state lock").is_some()
    });
    assert_eq!(*result.lock().expect("test state lock"), Some(Ok(())));
    wait(&parent, || {
        held.lock()
            .expect("test state lock")
            .as_ref()
            .expect("editor stored")
            .size()
            == (220, 140)
    });
    wait(&parent, || {
        parent
            .capture()
            .is_ok_and(|s| (s.width, s.height) == (220, 140))
    });
    assert!(
        held.lock()
            .expect("test state lock")
            .as_ref()
            .expect("editor stored")
            .error()
            .is_none()
    );
    // Break the host callback's shared handle slot before the next editor.
    drop(held.lock().expect("test state lock").take());
    wait(&parent, || parent.capture().is_err());

    close_during_resize_callback(&parent);
}

fn clipboard_requests_complete_through_the_host_pump() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let result = Arc::new(Mutex::new(None));
    let output = result.clone();
    let app = App::new("Embedded clipboard")
        .window_size(200, 120)
        .embed(parent.parent().scale(1.0), move || {
            spawn(async move {
                let result = async {
                    platform::copy_text("embedded 日本語 🦀").await?;
                    platform::read_text().await
                }
                .await;
                *output.lock().expect("test state lock") = Some(result);
            });
            text("Clipboard")
        })
        .expect("editor opens");
    wait(&parent, || {
        result.lock().expect("test state lock").is_some()
    });
    let result = result
        .lock()
        .expect("test state lock")
        .take()
        .expect("clipboard result");
    assert_eq!(
        result.expect("clipboard read succeeds"),
        "embedded 日本語 🦀"
    );
    assert!(app.error().is_none());
}

#[test]
fn opening_runs_root_once_and_delivers_click_key_and_text() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

    let _serial = SERIAL
        .lock()
        .expect("embed tests run one at a time; an earlier one panicked holding SERIAL");
    native_click_key_and_text_reach_the_editor();
}

#[test]
fn resize_negotiation_handles_acceptance_refusal_missing_callback_and_host_correction() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

    let _serial = SERIAL
        .lock()
        .expect("embed tests run one at a time; an earlier one panicked holding SERIAL");
    accepted_startup_resize_rounds_and_changes_capture_then_host_size_wins();
    refusal_and_missing_callback_complete_with_their_errors();
    correction_inside_accepting_resize_callback_remains_authoritative();
}

#[test]
fn parent_destruction_stops_the_app_without_an_error() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

    let _serial = SERIAL
        .lock()
        .expect("embed tests run one at a time; an earlier one panicked holding SERIAL");
    external_drop_joins_cleanup_after_parent_destruction();
}

#[test]
fn clipboard_text_round_trips_through_the_host_pump() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

    let _serial = SERIAL
        .lock()
        .expect("embed tests run one at a time; an earlier one panicked holding SERIAL");
    clipboard_requests_complete_through_the_host_pump();
}

#[cfg(windows)]
fn explode() {
    panic!("editor exploded");
}

/// A panic in the editor stops it without running more App code; the wrapper's
/// next call on `Embedded` resumes it on the calling thread.
#[cfg(windows)]
#[test]
#[should_panic(expected = "editor exploded")]
fn a_panic_in_the_editor_resumes_in_the_wrappers_next_call() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

    let serial = SERIAL
        .lock()
        .expect("embed tests run one at a time; an earlier one panicked holding SERIAL");
    let parent = TestParent::open(320, 240).expect("parent opens");
    let app = App::new("Panicking editor")
        .window_size(320, 240)
        .embed(parent.parent().scale(1.0), || {
            panel().focusable(true).on_press(explode)
        })
        .expect("editor opens");
    painted(&parent);
    parent.click(20.0, 20.0).expect("click delivered");
    wait(&parent, || parent.capture().is_err());
    // The panic must not poison the other embed tests.
    drop(serial);
    app.size();
}

fn external_drop_joins_cleanup_after_parent_destruction() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let cleaned = Arc::new(AtomicUsize::new(0));
    let cleanup = cleaned.clone();
    let app = App::new("Parent closes first")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), move || {
            on_cleanup(move || {
                cleanup.fetch_add(1, Ordering::SeqCst);
            });
            node().style(style().size(160, 100))
        })
        .expect("editor opens");
    wait(&parent, || parent.capture().is_ok());
    parent.close();
    wait(&parent, || cleaned.load(Ordering::SeqCst) == 1);
    assert!(app.error().is_none());
    drop(app);
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    assert!(parent.capture().is_err());
}

fn close_during_resize_callback(parent: &TestParent) {
    // A resize callback may close its editor. Its platform result must not
    // start another reactive turn and resume the task that requested it.
    let slot = Arc::new(Mutex::new(None::<Embedded>));
    let callback_slot = slot.clone();
    let returned = Arc::new(AtomicBool::new(false));
    let callback_returned = returned.clone();
    let resumed = Arc::new(AtomicBool::new(false));
    let continuation = resumed.clone();
    let cleaned = Arc::new(AtomicBool::new(false));
    let cleanup = cleaned.clone();
    let app = App::new("Resize callback close")
        .window_size(160, 100)
        .embed(
            parent.parent().scale(1.0).on_resize_request(move |_, _| {
                let app = callback_slot
                    .lock()
                    .expect("test state lock")
                    .take()
                    .expect("editor stored");
                assert!(app.error().is_none());
                drop(app);
                callback_returned.store(true, Ordering::SeqCst);
                true
            }),
            move || {
                on_cleanup(move || {
                    cleanup.store(true, Ordering::SeqCst);
                });
                node()
                    .style(style().size(160, 100).background(Color::WHITE))
                    .on_press(move || {
                        let future = platform::request_window_size(200, 120);
                        let continuation = continuation.clone();
                        spawn(async move {
                            let _ = future.await;
                            continuation.store(true, Ordering::SeqCst);
                        });
                    })
            },
        )
        .expect("editor opens");
    // Start the request only after the public handle has been stored.
    *slot.lock().expect("test state lock") = Some(app);
    wait(parent, || {
        parent
            .capture()
            .is_ok_and(|shot| shot.pixel(20, 20) == Some(0xffffff))
    });
    parent.click(20.0, 20.0).expect("click delivered");
    wait(parent, || {
        returned.load(Ordering::SeqCst)
            && cleaned.load(Ordering::SeqCst)
            && parent.capture().is_err()
    });
    parent.pump(Duration::from_millis(50));
    assert!(
        !resumed.load(Ordering::SeqCst),
        "a new task turn ran after the resize callback closed its editor"
    );
    assert!(slot.lock().expect("test state lock").is_none());
}
