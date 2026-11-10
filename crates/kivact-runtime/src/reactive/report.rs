//! Role: Error routing to handlers and diagnostic sinks.
//!
//! Error routing: a reported error goes to the nearest `on_error` handler
//! up the owner chain (an `error_boundary`), else to the App's default sink
//! (`tracing` and the diagnostics ring).
//!
//! Invariants:
//! - The handler runs in a handler frame of its boundary owner; a panic in
//!   it propagates. Only `Err` values are routed here, never panics.
//! - An error raised where no handler may run (code the engine calls while
//!   its tree is borrowed, as a primitive's measure function in its
//!   layout) is kept by [`Runtime::report_later`] and routed at the start
//!   of the next turn ([`Runtime::deliver_later`]), so a handler always
//!   runs where it may read and write the engine's tree.

use crate::reactive::arena::Key;
use crate::reactive::data::Runtime;
use crate::reactive::error::UiError;

impl Runtime {
    /// Route an error to the nearest `on_error` handler of `owner`'s chain,
    /// or to the App's default sink (tracing + diagnostics ring).
    pub(crate) fn route(&self, owner: Option<Key>, err: UiError) {
        let mut cur = owner;
        let mut handler = None;
        {
            let owners = self.owners.borrow();
            while let Some(k) = cur {
                let Some(o) = owners.get(k) else { break };
                if let Some(h) = o.extra.as_ref().and_then(|x| x.on_error.as_ref()) {
                    handler = Some(h.clone());
                    break;
                }
                cur = o.parent;
            }
        }
        match handler {
            Some(h) => {
                let _g = self.handler_frame(cur);
                h(err);
            }
            None => self.default_sink(err),
        }
    }

    /// Route `err` like [`Runtime::report`], but at the start of the next
    /// turn (one is requested): for errors raised where no handler may
    /// run. Safe to call anywhere; it runs no user code.
    pub(crate) fn route_later(&self, owner: Option<Key>, err: UiError) {
        self.later.borrow_mut().push((owner, err));
        self.request_turn();
    }

    /// Whether errors wait for [`Runtime::deliver_later`].
    pub(crate) fn has_later_reports(&self) -> bool {
        !self.later.borrow().is_empty()
    }

    /// Route the errors kept by [`Runtime::report_later`], oldest first (an
    /// owner disposed since goes to the default sink).
    pub fn deliver_later(&self) {
        let later = std::mem::take(&mut *self.later.borrow_mut());
        for (owner, err) in later {
            self.route(owner, err);
        }
    }

    /// The App's default error sink: `tracing` and the diagnostics ring.
    pub(crate) fn default_sink(&self, err: UiError) {
        tracing::error!(target: "kivact", kind = ?err.kind(), at = ?err.location(), "{}", err);
        self.diag.borrow_mut().record_error(err);
    }
}
