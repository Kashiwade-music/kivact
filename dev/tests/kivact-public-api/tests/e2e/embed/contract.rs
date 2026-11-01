//! Role: Twelve OS-independent real-window embedding contracts.
use super::support::{SERIAL, painted, panel, reopen, wait};
mod lifetime;
use kivact::{
    App,
    diagnostics::ErrorKind,
    embed::{Embedded, Parent},
    interaction::EditPhase,
    platform,
    prelude::*,
    style::Color,
    test::TestParent,
    view::{KeyCode, error_boundary},
};
use lifetime::{
    callback_can_drop_its_own_embedded_handle_without_panicking,
    external_drop_joins_cleanup_after_parent_destruction, external_drop_waits_for_cleanup,
    parent_cleanup_can_drop_its_own_embedded_handle,
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
#[derive(Clone, Debug, PartialEq)]
enum Event {
    Begin,
    Set(f64),
    End(EditPhase),
}
struct Wrapper {
    editor: Embedded,
    logical: (u32, u32),
}
impl Wrapper {
    fn scale(&self, scale: f64) {
        self.editor.set_scale(scale);
        self.editor.set_size(
            (f64::from(self.logical.0) * scale).round() as u32,
            (f64::from(self.logical.1) * scale).round() as u32,
        );
    }
}

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

fn null_parent_is_a_platform_error() {
    let result = App::new("Embedded").embed(Parent::win32(std::ptr::null_mut()), || text("Editor"));
    assert_eq!(
        *result.err().expect("null parent rejected").kind(),
        ErrorKind::Platform
    );
}

fn invalid_scale_is_a_message_error() {
    for scale in [
        f64::NAN,
        f64::INFINITY,
        0.0,
        -1.0,
        f64::MAX,
        f64::MIN_POSITIVE,
    ] {
        let result = App::new("Embedded").embed(Parent::x11(1).scale(scale), || text("Editor"));
        assert_eq!(
            *result.err().expect("invalid scale must fail").kind(),
            ErrorKind::Message
        );
    }
}

fn unhandled_root_panic_returns_original_diagnostic() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let error = App::new("Root panic")
        .embed(parent.parent(), || -> Node { panic!("root diagnostic") })
        .err()
        .expect("unhandled root panic must fail open");
    assert_eq!(*error.kind(), ErrorKind::Panic);
    assert!(error.message().contains("root diagnostic"));
    let survivor = App::new("Survivor")
        .embed(parent.parent(), panel)
        .expect("editor opens");
    assert!(survivor.error().is_none());
}

fn owner_boundary_recovers_root_setup_panic() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let app = App::new("Recovered root")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), || {
            error_boundary(|| -> Node { panic!("recovered child") }).fallback(|_, _| panel())
        })
        .expect("owner error boundary must recover");
    wait(&parent, || {
        parent
            .capture()
            .is_ok_and(|s| s.pixel(20, 20) == Some(0xffffff))
    });
    assert!(app.error().is_none());
}

fn size_scale_and_cross_thread_setters_are_applied() {
    let parent = TestParent::open(640, 480).expect("parent opens");
    let app = App::new("Size")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), panel)
        .expect("editor opens");
    painted(&parent);
    app.set_scale(2.0);
    wait(&parent, || app.scale() == 2.0);
    assert_eq!(app.scale(), 2.0);
    assert_eq!(app.size(), (160, 100));
    app.set_size(320, 200);
    wait(&parent, || app.size() == (320, 200));
    assert_eq!(app.size(), (320, 200));
    let app = std::thread::spawn(move || {
        app.set_scale(1.5);
        app.set_size(300, 180);
        app
    })
    .join()
    .expect("setter thread joined");
    wait(&parent, || app.scale() == 1.5 && app.size() == (300, 180));
    assert!(app.error().is_none());
    let dropping = std::thread::spawn(move || drop(app));
    wait(&parent, || dropping.is_finished());
    dropping.join().expect("drop thread joined");
    assert!(
        parent.capture().is_err(),
        "drop acknowledged native destruction"
    );
}

fn invalid_host_size_and_scale_are_ignored_without_failing_the_editor() {
    let parent = TestParent::open(400, 300).expect("parent opens");
    let app = App::new("Invalid host values")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), || {
            node().style(
                style()
                    .size(500, 500)
                    .background(kivact::style::Color::WHITE),
            )
        })
        .expect("editor opens");
    let deadline = Instant::now() + Duration::from_secs(10);
    let white = |parent: &TestParent, size: (u32, u32)| {
        parent.capture().is_ok_and(|shot| {
            (shot.width, shot.height) == size && shot.pixel(20, 20) == Some(0xffffff)
        })
    };
    while !white(&parent, (160, 100)) {
        assert!(Instant::now() < deadline, "initial frame timed out");
        parent.pump(Duration::from_millis(5));
    }
    app.set_size(0, 0);
    app.set_size(u32::MAX, 10);
    app.set_size(10, u32::MAX);
    app.set_size(10, 0);
    app.set_size(0, 10);
    app.set_scale(0.0);
    app.set_scale(f64::NAN);
    app.set_scale(f64::INFINITY);
    app.set_scale(f64::NEG_INFINITY);
    app.set_scale(-1.0);
    app.set_scale(f64::MAX);
    app.set_scale(f64::MIN_POSITIVE);
    parent.pump(Duration::from_millis(100));
    assert!(
        app.error().is_none(),
        "invalid host values must not fail the editor"
    );
    assert_eq!(app.size(), (160, 100));
    assert_eq!(app.scale(), 1.0);
    app.set_size(220, 140);
    while !white(&parent, (220, 140)) {
        assert!(Instant::now() < deadline, "editor stopped rendering");
        parent.pump(Duration::from_millis(5));
    }
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
    wait(&parent, || {
        parent
            .capture()
            .is_ok_and(|shot| shot.pixel(20, 20) == Some(0xffffff))
    });
    parent.click(20.0, 20.0).expect("click delivered");
    wait(&parent, || {
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

fn unhandled_event_panic_is_observable() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let later_effect = Arc::new(AtomicBool::new(false));
    let handled = Arc::new(AtomicUsize::new(0));
    let effect_ran = later_effect.clone();
    let events = handled.clone();
    let cleaned = Arc::new(AtomicBool::new(false));
    let cleanup = cleaned.clone();
    let started = Arc::new(AtomicBool::new(false));
    let worker_started = started.clone();
    let cancelled = Arc::new(AtomicBool::new(false));
    let worker_cancelled = cancelled.clone();
    let app = App::new("Event panic")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), move || {
            on_cleanup(move || cleanup.store(true, Ordering::Release));
            spawn(async move {
                kivact::task::blocking(move |cancel: kivact::task::CancelToken| {
                    worker_started.store(true, Ordering::Release);
                    while !cancel.is_cancelled() {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    worker_cancelled.store(true, Ordering::Release);
                })
                .await;
            });
            let pressed = signal(false);
            effect(move || {
                if pressed.get() {
                    effect_ran.store(true, Ordering::Release);
                }
            });
            panel().on_press(move || -> () {
                events.fetch_add(1, Ordering::AcqRel);
                pressed.set(true);
                panic!("event diagnostic")
            })
        })
        .expect("editor opens");
    wait(&parent, || {
        parent
            .capture()
            .is_ok_and(|s| s.pixel(20, 20) == Some(0xffffff))
    });
    wait(&parent, || started.load(Ordering::Acquire));
    parent.click(20.0, 20.0).expect("click delivered");
    wait(&parent, || app.error().is_some());
    wait(&parent, || {
        cleaned.load(Ordering::Acquire)
            && cancelled.load(Ordering::Acquire)
            && parent.capture().is_err()
    });
    let error = app.error().expect("panic diagnostic");
    assert_eq!(*error.kind(), ErrorKind::Panic);
    assert!(error.message().contains("event diagnostic"));
    assert!(
        later_effect.load(Ordering::Acquire),
        "the failing turn finishes its effects"
    );
    assert_eq!(handled.load(Ordering::Acquire), 1);
    // The child has stopped; input in a later turn must never reach its handler.
    let _ = parent.click(20.0, 20.0);
    parent.pump(Duration::from_millis(50));
    assert_eq!(handled.load(Ordering::Acquire), 1);
    assert_eq!(
        app.error().expect("panic diagnostic").message(),
        error.message()
    );
    let stopped_size = app.size();
    let stopped_scale = app.scale();
    app.set_size(220, 140);
    app.set_scale(2.0);
    parent.pump(Duration::from_millis(50));
    assert_eq!(app.size(), stopped_size);
    assert_eq!(app.scale(), stopped_scale);
    // Keep the failed handle alive through all cleanup and child-window checks.
    assert!(parent.capture().is_err());
    drop(app);
    reopen();
}

fn binding_writes_are_between_begin_and_end_and_cancel_restores_first() {
    let parent = TestParent::open(640, 480).expect("parent opens");
    let events = Arc::new(Mutex::new(Vec::new()));
    let output = events.clone();
    let editor = App::new("Wrapper")
        .window_size(320, 100)
        .embed(parent.parent().scale(1.0), move || {
            let value = signal(0.5);
            let writes = output.clone();
            let bound = binding(
                move || value.get(),
                move |next: f64| {
                    value.set(next);
                    writes
                        .lock()
                        .expect("test state lock")
                        .push(Event::Set(next));
                },
            );
            slider("Gain", bound)
                .range(0.0..=1.0)
                .on_edit(move |phase| {
                    output.lock().expect("test state lock").push(match phase {
                        EditPhase::Begin => Event::Begin,
                        end => Event::End(end),
                    });
                })
                .style(style().width(300).height(40))
        })
        .expect("editor opens");
    wait(&parent, || parent.capture().is_ok());
    parent.pump(Duration::from_millis(100));
    parent.press(150.0, 20.0).expect("press delivered");
    parent.move_to(250.0, 20.0).expect("pointer moved");
    parent.release(250.0, 20.0).expect("release delivered");
    wait(&parent, || {
        events.lock().expect("test state lock").last() == Some(&Event::End(EditPhase::Commit))
    });
    let committed = events.lock().expect("test state lock").clone();
    assert_eq!(committed.first(), Some(&Event::Begin));
    assert!(
        committed[1..committed.len() - 1]
            .iter()
            .all(|e| matches!(e, Event::Set(_)))
    );
    assert!(committed.len() >= 3);
    let Event::Set(before) = committed[committed.len() - 2] else {
        panic!("missing write")
    };
    events.lock().expect("test state lock").clear();
    parent.press(250.0, 20.0).expect("press delivered");
    parent.move_to(80.0, 20.0).expect("pointer moved");
    parent
        .key(kivact::view::KeyCode::Escape)
        .expect("key delivered");
    wait(&parent, || {
        events.lock().expect("test state lock").last() == Some(&Event::End(EditPhase::Cancel))
    });
    let canceled = events.lock().expect("test state lock").clone();
    assert_eq!(canceled.first(), Some(&Event::Begin));
    assert_eq!(canceled[canceled.len() - 2], Event::Set(before));
    parent.release(80.0, 20.0).expect("release delivered");
    let wrapper = Wrapper {
        editor,
        logical: (320, 100),
    };
    wrapper.scale(1.5);
    wait(&parent, || wrapper.editor.size() == (480, 150));
    assert_eq!(wrapper.editor.scale(), 1.5);
    wait(&parent, || {
        parent
            .capture()
            .is_ok_and(|s| (s.width, s.height) == (480, 150))
    });
    assert!(wrapper.editor.error().is_none());
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

fn callback_setter_does_not_reenter_app_code() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let slot = Arc::new(Mutex::new(None::<Embedded>));
    let held = slot.clone();
    let inside = Arc::new(AtomicBool::new(false));
    let pressing = inside.clone();
    let observing = inside.clone();
    let applied = Arc::new(AtomicBool::new(false));
    let observed = applied.clone();
    let after = applied.clone();
    let returned = Arc::new(AtomicBool::new(false));
    let done = returned.clone();
    let app = App::new("Callback setter")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), move || {
            effect(move || {
                if platform::window_size().width == 220.0 {
                    assert!(
                        !observing.load(Ordering::SeqCst),
                        "setter reentered an App callback"
                    );
                    observed.store(true, Ordering::SeqCst);
                }
            });
            panel().on_press(move || {
                pressing.store(true, Ordering::SeqCst);
                held.lock()
                    .expect("test state lock")
                    .as_ref()
                    .expect("editor stored")
                    .set_size(220, 140);
                assert!(!after.load(Ordering::SeqCst));
                pressing.store(false, Ordering::SeqCst);
                done.store(true, Ordering::SeqCst);
            })
        })
        .expect("editor opens");
    *slot.lock().expect("test state lock") = Some(app);
    painted(&parent);
    parent.click(20.0, 20.0).expect("click delivered");
    wait(&parent, || {
        applied.load(Ordering::SeqCst) && returned.load(Ordering::SeqCst)
    });
    assert_eq!(
        slot.lock()
            .expect("test state lock")
            .as_ref()
            .expect("editor stored")
            .size(),
        (220, 140)
    );
    assert!(
        slot.lock()
            .expect("test state lock")
            .as_ref()
            .expect("editor stored")
            .error()
            .is_none()
    );
    drop(slot.lock().expect("test state lock").take());
    assert!(parent.capture().is_err());
}
fn two_apps_share_one_parent_without_sharing_state() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let first = Arc::new(AtomicUsize::new(0));
    let second = Arc::new(AtomicUsize::new(0));
    let a_count = first.clone();
    let a = App::new("First sibling")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), move || {
            panel().on_press(move || {
                a_count.fetch_add(1, Ordering::SeqCst);
            })
        })
        .expect("editor opens");
    let b_count = second.clone();
    let b = App::new("Second sibling")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), move || {
            panel().on_press(move || {
                b_count.fetch_add(1, Ordering::SeqCst);
            })
        })
        .expect("editor opens");
    painted(&parent);
    parent.click(20.0, 20.0).expect("click delivered");
    wait(&parent, || {
        first.load(Ordering::SeqCst) + second.load(Ordering::SeqCst) == 1
    });
    // Native drivers may enumerate siblings in either stacking order.
    let first_before = first.load(Ordering::SeqCst);
    let second_before = second.load(Ordering::SeqCst);
    assert!(matches!((first_before, second_before), (1, 0) | (0, 1)));
    a.set_size(200, 120);
    wait(&parent, || a.size() == (200, 120));
    assert_eq!(b.size(), (160, 100));
    b.set_scale(1.5);
    wait(&parent, || b.scale() == 1.5);
    assert_eq!(a.scale(), 1.0);
    assert_eq!(a.size(), (200, 120));
    assert!(a.error().is_none());
    assert!(b.error().is_none());
    drop(b);
    painted(&parent);
    parent.click(20.0, 20.0).expect("click delivered");
    wait(&parent, || first.load(Ordering::SeqCst) == first_before + 1);
    assert_eq!(second.load(Ordering::SeqCst), second_before);
    assert!(a.error().is_none());
    drop(a);
    assert!(parent.capture().is_err());
}

#[test]
fn opening_runs_root_once_and_delivers_click_key_and_text() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    native_click_key_and_text_reach_the_editor();
}
#[test]
fn opening_rejects_null_parent_and_invalid_scale_with_their_error_kinds() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    null_parent_is_a_platform_error();
    invalid_scale_is_a_message_error();
}
#[test]
fn root_panic_preserves_its_diagnostic_and_owner_boundary_recovers() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    unhandled_root_panic_returns_original_diagnostic();
    owner_boundary_recovers_root_setup_panic();
}
#[test]
fn queued_setters_apply_across_threads_and_do_not_reenter_callbacks() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    size_scale_and_cross_thread_setters_are_applied();
    callback_setter_does_not_reenter_app_code();
}
#[test]
fn invalid_host_values_are_ignored_and_the_editor_keeps_rendering() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    invalid_host_size_and_scale_are_ignored_without_failing_the_editor();
}
#[test]
fn resize_negotiation_handles_acceptance_refusal_missing_callback_and_host_correction() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    accepted_startup_resize_rounds_and_changes_capture_then_host_size_wins();
    refusal_and_missing_callback_complete_with_their_errors();
    correction_inside_accepting_resize_callback_remains_authoritative();
}
#[test]
fn drop_waits_outside_callbacks_and_defers_inside_press_and_cleanup() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    external_drop_waits_for_cleanup();
    callback_can_drop_its_own_embedded_handle_without_panicking();
    parent_cleanup_can_drop_its_own_embedded_handle();
}
#[test]
fn parent_destruction_stops_the_app_without_an_error() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    external_drop_joins_cleanup_after_parent_destruction();
}
#[test]
fn event_panic_finishes_its_turn_then_releases_resources_and_stops_input() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    unhandled_event_panic_is_observable();
}
#[test]
fn two_editors_in_the_same_parent_remain_independent() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    two_apps_share_one_parent_without_sharing_state();
}
#[test]
fn wrapper_writes_stay_between_begin_and_commit_or_cancel() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    binding_writes_are_between_begin_and_end_and_cancel_restores_first();
}
#[test]
fn clipboard_text_round_trips_through_the_host_pump() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    clipboard_requests_complete_through_the_host_pump();
}
