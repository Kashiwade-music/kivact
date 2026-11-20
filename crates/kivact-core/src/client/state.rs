//! Role: Per-App client state for the host boundary.
//!
//! Per-App host event counters, installed as a runtime extension when the
//! App is created.

use std::cell::Cell;

use kivact_runtime::driver::Runtime;

/// Everything the client keeps per App about the host link.
#[derive(Default)]
pub(crate) struct HostState {
    /// Turns run so far.
    pub(crate) stats: Cell<HostStats>,
}

impl HostState {
    /// The host-link state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &HostState {
        &crate::app_state::AppState::of(rt).host
    }

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
