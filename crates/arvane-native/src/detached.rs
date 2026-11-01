//! Role: Records whether this crate ever detached a thread that may still run its code.

pub(crate) static DETACHED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Whether this crate has ever detached a thread that may still execute its code.
pub fn detached_threads() -> bool {
    DETACHED.load(std::sync::atomic::Ordering::Acquire)
}
