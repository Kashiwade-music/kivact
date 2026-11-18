//! Role: Real-window embedding contracts for showing and hiding an editor.

use super::support::{SERIAL, panel, wait};
use kivact::embed::Embedded;
use kivact::motion::FrameRate;
use kivact::prelude::*;
use kivact::test::TestParent;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

fn hidden_editor_draws_nothing_until_shown() {
    let parent = TestParent::open(320, 240).expect("parent opens");
    let frames = Arc::new(AtomicU64::new(0));
    let counter = frames.clone();
    let editor: Embedded = App::new("Hidden editor")
        .window_size(320, 240)
        .embed(parent.parent().scale(1.0).hidden(), move || {
            let counter = counter.clone();
            frame_loop(move |_| {
                counter.fetch_add(1, Ordering::SeqCst);
            })
            .start(FrameRate::Display);
            panel()
        })
        .expect("editor opens");
    assert!(!editor.is_visible(), "the editor opened hidden");
    assert!(
        !parent.editor_visible().expect("the child exists"),
        "a hidden editor's window is not visible"
    );
    parent.pump(Duration::from_millis(100));
    assert_eq!(
        frames.load(Ordering::SeqCst),
        0,
        "a hidden editor runs no frame loops"
    );
    editor.show();
    wait(&parent, || frames.load(Ordering::SeqCst) > 0);
    assert!(editor.is_visible(), "show applies on the UI thread");
    assert!(
        parent.editor_visible().expect("the child exists"),
        "show makes the child window visible"
    );
    editor.hide();
    wait(&parent, || !editor.is_visible());
    assert!(
        !parent.editor_visible().expect("the child exists"),
        "hide makes the child window invisible"
    );
    parent.pump(Duration::from_millis(50));
    let stopped = frames.load(Ordering::SeqCst);
    parent.pump(Duration::from_millis(100));
    assert_eq!(
        frames.load(Ordering::SeqCst),
        stopped,
        "a hidden editor's frame loops stop"
    );
    editor.show();
    wait(&parent, || frames.load(Ordering::SeqCst) > stopped);
    assert!(editor.is_visible());
    drop(editor);
}

#[test]
fn hidden_editors_have_no_surface_until_shown() {
    if let Some(reason) = kivact::test::native_unavailable() {
        eprintln!("SKIPPED: {reason}");
        return;
    }

    let _serial = SERIAL
        .lock()
        .expect("embed tests run one at a time; an earlier one panicked holding SERIAL");
    hidden_editor_draws_nothing_until_shown();
}
