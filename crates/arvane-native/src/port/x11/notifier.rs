//! Role: Own X11 notification identity and the UI thread's lifetime.
use std::os::fd::OwnedFd;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread::JoinHandle;
/// Largest physical dimension an X11 window accepts (16-bit extents).
pub(super) const MAX_DIMENSION: u32 = u16::MAX as u32;
pub(super) struct Notifier {
    pub(super) wake: OwnedFd,
    // xtask-allow(state-fields): independent: bits OR-ed in by `request` on any thread and swapped to zero by `take` on the UI thread; a request in any UiThread state only sets bits, which wait until the UI loop takes them
    pub(super) reasons: AtomicU8,
    /// The UI thread's lifetime: spawned once, and joined once at close.
    pub(super) thread: Mutex<UiThread>,
}
/// The UI thread's lifetime.
pub(super) enum UiThread {
    /// Not spawned yet.
    Unspawned,
    /// Spawned; the handle is held until the thread is joined or detached.
    Running(JoinHandle<()>),
    /// The handle was taken once (joined, or detached because its own callback closed).
    Joined,
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
