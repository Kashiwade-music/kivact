//! Role: Shared real-host waits and fixtures for embedding contracts.
use kivact::prelude::*;
use kivact::test::TestParent;
use kivact::view::Node;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};
static SERIAL: Mutex<()> = Mutex::new(());
/// Run the embed tests one at a time. The lock guards nothing but the order,
/// so a test that panicked while holding it does not fail the ones after it.
pub(super) fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
}
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
