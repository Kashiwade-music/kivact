//! Role: Shared real-host waits and fixtures for embedding contracts.
use kivact::{prelude::*, style::Color, test::TestParent};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
pub(super) static SERIAL: Mutex<()> = Mutex::new(());
#[track_caller]
pub(super) fn wait(parent: &TestParent, mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !ready() {
        assert!(Instant::now() < deadline, "embedded operation timed out");
        parent.pump(Duration::from_millis(5));
    }
}
#[track_caller]
pub(super) fn painted(parent: &TestParent) {
    wait(parent, || {
        parent
            .capture()
            .is_ok_and(|shot| shot.pixel(20, 20) == Some(0xffffff))
    });
}
pub(super) fn panel() -> Node {
    node().style(style().size(160, 100).background(Color::WHITE))
}
pub(super) fn reopen() {
    let parent = TestParent::open(200, 120).expect("open another real parent");
    let app = App::new("Reopened")
        .window_size(160, 100)
        .embed(parent.parent().scale(1.0), panel)
        .expect("reopen the embedded editor");
    painted(&parent);
    assert!(app.error().is_none());
    drop(app);
    assert!(parent.capture().is_err());
}
