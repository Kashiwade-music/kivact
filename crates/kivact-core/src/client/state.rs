//! Role: Per-App client state for the host boundary.
//!
//! Per-App host event counters, installed as a runtime extension when the
//! App is created.

use std::cell::Cell;

/// Everything the client keeps per App about the host link.
#[derive(Default)]
pub(crate) struct HostState {
    /// Turns run so far.
    pub(crate) stats: Cell<HostStats>,
}

impl HostState {
    /// Turns run so far.
    pub(crate) fn turns(&self) -> u64 {
        self.stats.get().turns
    }

    /// Events dropped because their node no longer exists.
    pub(crate) fn stale_events(&self) -> u64 {
        self.stats.get().stale_events
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct HostStats {
    pub(crate) turns: u64,
    pub(crate) stale_events: u64,
}
