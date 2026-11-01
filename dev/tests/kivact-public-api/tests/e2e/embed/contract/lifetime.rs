//! Role: Embedded cleanup and callback-contained drop contract helpers.
use super::super::support::wait;
use kivact::{App, prelude::*, test::TestParent};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

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
