//! Role: Embedded cleanup and callback-contained drop contract helpers.
use super::super::support::{reopen, wait};
use kivact::{App, embed::Embedded, prelude::*, test::TestParent};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

pub(super) fn external_drop_waits_for_cleanup() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let cleaned = Arc::new(AtomicUsize::new(0));
    let cleanup = cleaned.clone();
    let app = App::new("External close")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), move || {
            on_cleanup(move || {
                cleanup.fetch_add(1, Ordering::SeqCst);
            });
            node().style(style().size(160, 100))
        })
        .expect("editor opens");
    wait(&parent, || parent.capture().is_ok());
    assert!(app.error().is_none());
    drop(app);
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    assert!(parent.capture().is_err());
}

pub(super) fn callback_can_drop_its_own_embedded_handle_without_panicking() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let slot = Arc::new(Mutex::new(None::<Embedded>));
    let cleaned = Arc::new(AtomicUsize::new(0));
    let returned = Arc::new(AtomicBool::new(false));
    let (handle, cleanup, after_drop) = (slot.clone(), cleaned.clone(), returned.clone());
    let callback_returned = returned.clone();
    let deliveries = Arc::new(AtomicUsize::new(0));
    let delivered = deliveries.clone();
    let effect_runs = Arc::new(AtomicUsize::new(0));
    let effect_count = effect_runs.clone();
    let app = App::new("Callback close")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), move || {
            on_cleanup(move || {
                assert!(
                    callback_returned.load(Ordering::SeqCst),
                    "cleanup reentered the press callback"
                );
                cleanup.fetch_add(1, Ordering::SeqCst);
            });
            let changed = signal(false);
            effect(move || {
                if changed.get() {
                    effect_count.fetch_add(1, Ordering::SeqCst);
                }
            });
            node()
                .style(
                    style()
                        .size(160, 100)
                        .background(kivact::style::Color::WHITE),
                )
                .on_press(move || {
                    if delivered.fetch_add(1, Ordering::SeqCst) != 0 {
                        return;
                    }
                    let app = handle
                        .lock()
                        .expect("test state lock")
                        .take()
                        .expect("editor stored");
                    assert!(app.error().is_none());
                    drop(app);
                    // The turn in progress finishes; cleanup follows it.
                    changed.set(true);
                    // Cleanup alone also happens after a contained callback panic.
                    // This marker proves Drop returned normally to the callback.
                    after_drop.store(true, Ordering::SeqCst);
                })
        })
        .expect("editor opens");
    *slot.lock().expect("test state lock") = Some(app);
    wait(&parent, || {
        parent
            .capture()
            .is_ok_and(|shot| shot.pixel(20, 20) == Some(0xffffff))
    });
    parent.click(20., 20.).expect("click delivered");
    // Queue a second action before the host pump. The X11 UI thread may already
    // have destroyed its child; otherwise this pending action must be discarded.
    if parent.click(20., 20.).is_err() {
        assert!(
            parent.capture().is_err(),
            "input failed while the child was still present"
        );
    }
    wait(&parent, || {
        returned.load(Ordering::SeqCst)
            && cleaned.load(Ordering::SeqCst) == 1
            && parent.capture().is_err()
    });
    parent.pump(Duration::from_millis(50));
    assert_eq!(
        deliveries.load(Ordering::SeqCst),
        1,
        "pending actions ran after callback Drop"
    );
    assert!(slot.lock().expect("test state lock").is_none());
    assert_eq!(effect_runs.load(Ordering::SeqCst), 1);
}

pub(super) fn parent_cleanup_can_drop_its_own_embedded_handle() {
    let parent = TestParent::open(200, 120).expect("parent opens");
    let slot = Arc::new(Mutex::new(None::<Embedded>));
    let cleaned = Arc::new(AtomicUsize::new(0));
    let (held, count) = (slot.clone(), cleaned.clone());
    let app = App::new("Cleanup drop")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), move || {
            on_cleanup(move || {
                let app = held.lock().expect("test state lock").take();
                drop(app);
                count.fetch_add(1, Ordering::SeqCst);
            });
            text("Cleanup drop")
        })
        .expect("editor opens");
    *slot.lock().expect("test state lock") = Some(app);
    parent.close();
    assert!(parent.is_closed());
    wait(&parent, || cleaned.load(Ordering::SeqCst) == 1);
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    assert!(slot.lock().expect("test state lock").is_none());
    reopen();
}

pub(super) fn external_drop_joins_cleanup_after_parent_destruction() {
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
