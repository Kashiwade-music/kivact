//! Role: Host clock and waker driving the reactive runtime.
//!
//! The client as the driver of its runtime (`kivact_runtime::Driver`): it
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
//!   would cost the host one more, empty, turn. The waker is read from a
//!   `OnceCell`, so a waker that reaches the link again finds no borrow
//!   held.

use std::cell::{Cell, OnceCell};

use kivact_engine::api::HostWaker;
use kivact_runtime::Driver;

/// The host clock and waker of one App.
#[derive(Default)]
pub(crate) struct HostLink {
    /// The host clock (ns).
    now_ns: Cell<u64>,
    /// Wakes the host for a turn; set once by the host's `start`.
    waker: OnceCell<HostWaker>,
    /// A turn is running ([`HostLink::enter_turn`]).
    in_turn: Cell<bool>,
}

impl HostLink {
    /// The host started: its clock reads `now_ns`, and `waker` wakes it.
    pub(crate) fn start(&self, now_ns: u64, waker: HostWaker) {
        self.now_ns.set(now_ns);
        self.waker
            .set(waker)
            .expect("invariant: the host starts an App once");
    }

    /// A turn runs until the returned scope drops; turn requests meanwhile
    /// wake nobody.
    pub(crate) fn enter_turn(&self) -> TurnScope<'_> {
        debug_assert!(!self.in_turn.get(), "invariant: turns do not nest");
        self.in_turn.set(true);
        TurnScope(self)
    }

    /// Move the clock forward to `ns`; it never goes back.
    pub(crate) fn advance_clock(&self, ns: u64) {
        self.now_ns.set(self.now_ns.get().max(ns));
    }
}

/// A running turn ([`HostLink::enter_turn`]); dropping it ends the turn.
#[must_use = "the turn ends when the scope drops"]
pub(crate) struct TurnScope<'a>(&'a HostLink);

impl Drop for TurnScope<'_> {
    fn drop(&mut self) {
        self.0.in_turn.set(false);
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
        // The waker is the host's code and may reach this link again (a
        // host that runs the turn in place, a test).
        if let Some(waker) = self.waker.get() {
            waker.wake();
        }
    }
}
