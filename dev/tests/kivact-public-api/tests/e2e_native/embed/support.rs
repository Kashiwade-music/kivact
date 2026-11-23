//! Role: Shared real-host waits and fixtures for embedding contracts.
use kivact::prelude::*;
use kivact::test::TestParent;
use kivact::view::Node;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
static SERIAL: AtomicBool = AtomicBool::new(false);
/// Held while one embed test runs; dropping it (also while a panic unwinds)
/// lets the next test start.
pub(super) struct Serial(());
impl Drop for Serial {
    fn drop(&mut self) {
        SERIAL.store(false, Ordering::Release);
    }
}
/// Run the embed tests one at a time. The lock orders the tests and guards no
/// data, so it has no poisoned state: a test that panicked while holding it
/// does not fail the ones after it.
pub(super) fn serial() -> Serial {
    while SERIAL
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        std::thread::sleep(Duration::from_millis(5));
    }
    Serial(())
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
