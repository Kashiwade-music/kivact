//! Role: Per-App client state for the host boundary.
//!
//! Per-App asset failures already reported and host event counters, installed as a runtime
//! extension when the App is created.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;

use kivact_types::NodeId;

use kivact_runtime::Runtime;

/// Everything the client keeps per App about the host link.
#[derive(Default)]
pub(crate) struct HostState {
    /// Turns run so far.
    pub(super) stats: Cell<HostStats>,
    /// Asset failures reported so far, by node and message (each once);
    /// entries of removed nodes are dropped when the revision removing them
    /// is committed.
    pub(crate) removed: RefCell<Vec<NodeId>>,
    pub(super) asset_failures: RefCell<HashSet<(NodeId, String)>>,
}

impl HostState {
    /// The host-link state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &HostState {
        rt.ext::<HostState>()
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
pub(super) struct HostStats {
    pub(super) turns: u64,
    pub(super) stale_events: u64,
}
