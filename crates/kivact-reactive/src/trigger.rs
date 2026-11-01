//! Role: Value-free dependencies that computations track and writers notify.
//!
//! A [`Trigger`] is a source node without a payload: a computation that
//! calls [`Trigger::track`] re-runs after [`Trigger::notify`]. Layers use it
//! for a revision of state the graph does not hold (a retained tree, a
//! queue, a measurement), instead of a signal whose value nobody reads.
//!
//! Invariants:
//! - Notifying queues the subscribers like any write (with the same purity
//!   and disposal rules and the same trace record); it never runs them
//!   inline.
//! - [`Trigger::notify_from_render`] is the one write a render effect may
//!   make, and only for metadata the effect itself changed in retained
//!   state: it validates the handle untracked and queues the subscribers,
//!   so the notifying effect never subscribes itself.

use std::marker::PhantomData;
use std::panic::Location;

use crate::AppId;
use crate::arena::Key;
use crate::data::{Signal, UiThreadOnly};
use crate::error::{ErrorKind, misuse_at};
use crate::graph::source_cell;
use crate::registry::{runtime, try_runtime};

/// A dependency without a value, owned by the owner that created it
/// ([`crate::Owner::trigger`]). Copy; it does not keep the App alive.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Trigger {
    app: AppId,
    key: Key,
    marker: PhantomData<UiThreadOnly>,
}

impl Trigger {
    pub(crate) fn from_key(app: AppId, key: Key) -> Self {
        Self {
            app,
            key,
            marker: PhantomData,
        }
    }

    /// Subscribe the running computation (a memo or an effect) to this
    /// trigger. Outside a computation it records nothing. Nothing happens
    /// when the trigger or its App is gone.
    #[track_caller]
    pub fn track(self) {
        if let Some(rt) = try_runtime(self.app) {
            rt.read_node(self.key, true, Location::caller());
        }
    }

    /// Queue the subscribers, as a write of a signal does: rejected in a
    /// pure computation (`WriteInPureContext`), ignored and recorded as a
    /// diagnostic once the trigger is disposed.
    #[track_caller]
    pub fn notify(self) {
        Signal::<()>::from_key(self.app, self.key).notify();
    }

    /// Queue the subscribers from a render effect, after it changed retained
    /// metadata that they derive from (a scene, a visibility). It does not
    /// track the trigger, write a value or run anyone inline. A located
    /// `WrongApp` or `Disposed` for a handle of another App or a disposed
    /// trigger. Not a general write: application state uses signals, whose
    /// writes stay rejected in pure computations.
    #[track_caller]
    pub fn notify_from_render(self) {
        let at = Location::caller();
        let rt = runtime(self.app, at);
        let cell = rt.read_or_panic(self.key, false, "Trigger", at);
        if source_cell::<()>(&cell).is_none() {
            misuse_at(ErrorKind::WrongApp, "handle type mismatch", at);
        }
        rt.notify_subscribers(self.key);
    }
}

impl std::fmt::Debug for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Trigger({}:{})", self.key.slot, self.key.generation)
    }
}
