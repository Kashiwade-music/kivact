//! Role: Own X11 notification identity and the UI thread join handle.
use std::os::fd::OwnedFd;
use std::sync::Mutex;
use std::thread::JoinHandle;
/// Largest physical dimension an X11 window accepts (16-bit extents).
pub(super) const MAX_DIMENSION: u32 = u16::MAX as u32;
pub(super) struct Notifier {
    pub(super) wake: OwnedFd,
    pub(super) thread: Mutex<Option<JoinHandle<()>>>,
}
