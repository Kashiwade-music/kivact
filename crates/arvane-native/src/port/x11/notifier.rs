//! Role: Own X11 notification identity and the UI thread join handle.
use std::os::fd::OwnedFd;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread::JoinHandle;
/// Largest physical dimension an X11 window accepts (16-bit extents).
pub(super) const MAX_DIMENSION: u32 = u16::MAX as u32;
pub(super) struct Notifier {
    pub(super) wake: OwnedFd,
    pub(super) reasons: AtomicU8,
    pub(super) thread: Mutex<Option<JoinHandle<()>>>,
}
impl Notifier {
    pub(super) fn request(&self, reasons: arvane_shell::WakeReasons) {
        let bits = crate::wake::encode(reasons);
        if bits != 0 && self.reasons.fetch_or(bits, Ordering::AcqRel) == 0 {
            super::poll::signal(&self.wake);
        }
    }
    pub(super) fn take(&self) -> arvane_shell::WakeReasons {
        crate::wake::decode(self.reasons.swap(0, Ordering::AcqRel))
    }
}
