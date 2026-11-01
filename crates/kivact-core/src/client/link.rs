//! Role: Host clock and waker driving the reactive runtime.
//!
//! The client as the driver of its runtime (`kivact_reactive::Driver`): it
//! owns the host clock and the host waker, and lends the runtime the
//! clock reading and the turn request.
//!
//! Invariants:
//! - The clock starts at the host's `now_ns` of `HostInfo` and only moves
//!   forward, to the time of a turn or of a `FrameTick` (virtual in tests).
//!   It reads 0 before the host starts.
//! - A turn request before the host starts wakes nobody (the host runs the
//!   first turn anyway), and neither does one during a turn: the turn
//!   itself settles what was requested, and the client asks again at its
//!   end if work is left (`AppClient::turn`). A wake from inside the turn
//!   would cost the host one more, empty, turn. The waker runs with no
//!   borrow of the link held, so a waker that reaches the link again is
//!   fine.

use std::cell::{Cell, RefCell};

use kivact_engine::bridge::HostWaker;
use kivact_reactive::Driver;

/// The host clock and waker of one App.
#[derive(Default)]
pub(crate) struct HostLink {
    /// The host clock (ns).
    now_ns: Cell<u64>,
    /// Wakes the host for a turn (from the host's `start`).
    waker: RefCell<Option<HostWaker>>,
    /// A turn is running.
    in_turn: Cell<bool>,
}

impl HostLink {
    /// The host started: its clock reads `now_ns`, and `waker` wakes it.
    pub(crate) fn start(&self, now_ns: u64, waker: HostWaker) {
        self.now_ns.set(now_ns);
        *self.waker.borrow_mut() = Some(waker);
    }

    /// A turn starts (`true`) or ends.
    pub(crate) fn set_in_turn(&self, on: bool) {
        self.in_turn.set(on);
    }

    /// Move the clock forward to `ns`; it never goes back.
    pub(crate) fn advance_clock(&self, ns: u64) {
        self.now_ns.set(self.now_ns.get().max(ns));
    }
}

impl Driver for HostLink {
    fn now_ns(&self) -> u64 {
        self.now_ns.get()
    }

    fn request_turn(&self) {
        if self.in_turn.get() {
            return;
        }
        // Wake outside the borrow: a waker is the host's code and may reach
        // this link again (a host that runs the turn in place, a test).
        let waker = self.waker.borrow().clone();
        if let Some(w) = waker {
            w.wake();
        }
    }
}
