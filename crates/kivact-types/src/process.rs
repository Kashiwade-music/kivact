//! Role: Remember process-wide thread detachment for safe native module retention.

use std::sync::atomic::{AtomicUsize, Ordering};

static DETACHED: AtomicUsize = AtomicUsize::new(0);

/// Process-wide sticky detach fact, the named statistics exception to S-9.
///
/// Once code has detached a worker, that worker may still execute its code.
/// Completion is not observed here, so the count never decrements or resets.
/// Native module retention must remain conservative for the process lifetime.
pub struct DetachedThreads;

impl DetachedThreads {
    /// Record one detached thread, saturating rather than losing the sticky fact.
    pub fn note() {
        let _ = DETACHED.fetch_update(Ordering::Release, Ordering::Relaxed, |count| {
            Some(count.saturating_add(1))
        });
    }

    /// Whether any thread has ever been detached in this process.
    pub fn any() -> bool {
        DETACHED.load(Ordering::Acquire) != 0
    }
}
