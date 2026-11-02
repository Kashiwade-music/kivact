//! Role: `on_mount` callback scheduling and deferred execution.
//!
//! `on_mount` callbacks, deferred until the driver of the runtime (the
//! client) acknowledges what the setup created. The runtime does not know
//! what is acknowledged or by whom: the driver hands it opaque tokens that
//! only grow (the client uses the revision of the batch that carries the
//! new nodes, acknowledged when the host applied it).
//!
//! Invariants:
//! - A queued callback waits without a token until the driver stamps the
//!   waiting callbacks with the next token ([`Runtime::defer_mounts_until`]);
//!   it runs once a token at least as large is acknowledged
//!   ([`Runtime::run_mounts_acked`]), in queue order.
//! - A callback whose owner was disposed meanwhile never runs; it is
//!   dropped when its token is acknowledged (or with the runtime).
//! - Each callback runs in a handler frame of its owner; a panic propagates.

use crate::arena::Key;
use crate::data::{PendingMount, Runtime};

impl Runtime {
    /// Queue `f` to run for `owner` once the next token the driver stamps is
    /// acknowledged.
    pub(crate) fn push_mount(&self, owner: Key, f: Box<dyn FnOnce()>) {
        self.mounts.borrow_mut().push(PendingMount {
            owner,
            token: None,
            run: f,
        });
    }

    /// Make every queued callback without a token wait for `token`; returns
    /// whether any did.
    pub(crate) fn defer_mounts_until(&self, token: u64) -> bool {
        let mut stamped = false;
        for m in self.mounts.borrow_mut().iter_mut() {
            match m.token {
                None => {
                    m.token = Some(token);
                    stamped = true;
                }
                Some(t) => assert!(t <= token, "invariant: mount tokens only grow"),
            }
        }
        stamped
    }

    /// The driver acknowledged every token up to `token`: run the callbacks
    /// waiting for one of them, in queue order, for owners that are still
    /// alive.
    pub(crate) fn run_mounts_acked(&self, token: u64) {
        let due: Vec<(Key, Box<dyn FnOnce()>)> = {
            let mut mounts = self.mounts.borrow_mut();
            let all = std::mem::take(&mut *mounts);
            let (due, keep): (Vec<_>, Vec<_>) = all
                .into_iter()
                .partition(|m| m.token.is_some_and(|t| t <= token));
            *mounts = keep;
            due.into_iter().map(|m| (m.owner, m.run)).collect()
        };
        for (owner, f) in due {
            if self.owner_alive(owner) {
                let _g = self.handler_frame(Some(owner));
                f();
            }
        }
    }

    /// Whether a queued callback waits for a token up to `token` (it runs
    /// in the next [`Runtime::run_mounts_acked`] with `token`).
    pub(crate) fn mounts_due(&self, token: u64) -> bool {
        self.mounts
            .borrow()
            .iter()
            .any(|m| m.token.is_some_and(|t| t <= token))
    }
}
