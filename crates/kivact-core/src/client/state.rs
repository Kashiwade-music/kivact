//! Role: Per-App client state for the host boundary.
//!
//! Per-App state of the client side of the host boundary (the host clock
//! and waker, close/exit, the
//! asset failures already reported and counters), installed as a runtime
//! extension when the App is created.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use kivact_wire::NodeId;

use crate::client::link::HostLink;
use kivact_reactive::Runtime;

/// Everything the client keeps per App about the host link.
#[derive(Default)]
pub(crate) struct HostState {
    /// The host clock and waker; also the runtime's driver.
    pub(super) link: Rc<HostLink>,
    /// The host asked to close the window.
    pub(super) close_requested: Cell<bool>,
    /// Deadline of the exit drain after a close request (ns).
    pub(super) exit_deadline: Cell<Option<u64>>,
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
