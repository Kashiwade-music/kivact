//! Role: Per-App client state for the host boundary.
//!
//! Per-App state of the client side of the host boundary (the host clock
//! and waker, close/exit, the
//! asset failures already reported and counters), installed as a runtime
//! extension when the App is created.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use kivact_types::NodeId;

use crate::client::link::HostLink;
use kivact_runtime::Runtime;

/// Where the App stands toward its exit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ExitPhase {
    /// No close was requested.
    #[default]
    Running,
    /// The host asked to close: queued actions drain until `deadline_ns`
    /// (the first request's time plus `kivact_runtime::DEFAULT_DRAIN_NS`); a
    /// later request keeps it.
    Draining { deadline_ns: u64 },
    /// The App told the host to exit.
    Exited,
}

/// Everything the client keeps per App about the host link.
#[derive(Default)]
pub(crate) struct HostState {
    /// The host clock and waker; also the runtime's driver.
    pub(super) link: Rc<HostLink>,
    /// Where the App stands toward its exit.
    pub(super) exit: Cell<ExitPhase>,
    /// Turns run so far.
    pub(super) turns: Cell<u64>,
    /// Events dropped because their node no longer exists.
    pub(super) stale_events: Cell<u64>,
    /// Asset failures reported so far, by node and message (each once);
    /// entries of removed nodes are dropped when the revision removing them
    /// is committed.
    pub(super) asset_failures: RefCell<HashSet<(NodeId, String)>>,
}

impl HostState {
    /// The host-link state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &HostState {
        rt.ext::<HostState>()
    }

    /// Move the host clock forward to `ns` (a turn, a `FrameTick`).
    pub(super) fn advance_clock(&self, ns: u64) {
        self.link.advance_clock(ns);
    }

    /// The host asked to close at `now_ns`: the drain starts. A repeated
    /// request keeps the first deadline.
    pub(super) fn request_close(&self, now_ns: u64) {
        if self.exit.get() == ExitPhase::Running {
            self.exit.set(ExitPhase::Draining {
                deadline_ns: now_ns.saturating_add(kivact_runtime::DEFAULT_DRAIN_NS),
            });
        }
    }

    /// Turns run so far.
    #[cfg(feature = "test")]
    pub(crate) fn turns(&self) -> u64 {
        self.turns.get()
    }

    /// Events dropped because their node no longer exists.
    #[cfg(feature = "test")]
    pub(crate) fn stale_events(&self) -> u64 {
        self.stale_events.get()
    }
}
