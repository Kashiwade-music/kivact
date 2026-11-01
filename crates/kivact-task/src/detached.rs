//! Role: Track worker detach facts below execution and shutdown orchestration.
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) static DETACHED: AtomicBool = AtomicBool::new(false);

/// Whether this crate has ever detached a thread that may still execute its code.
pub fn detached_threads() -> bool {
    DETACHED.load(Ordering::Acquire)
}
