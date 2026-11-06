//! Role: Share an embedded host closure request across callback threads.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A closure request observed at host callback boundaries.
#[derive(Clone, Default)]
pub struct CloseSignal(Arc<AtomicBool>);

impl CloseSignal {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn close(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_set(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
