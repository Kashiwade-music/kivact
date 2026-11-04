//! Role: The host's view of a runtime: creation, turns, mounts and reports.
//!
//! A host (the client of one App) creates the runtime with its driver,
//! runs the turns ([`Runtime::stabilize`], [`Runtime::run_user_effects`]),
//! acknowledges what the setup created ([`Mounts`]), records the turn trace
//! ([`TurnRecorder`]) and shuts the runtime down. Everything here is
//! host-neutral: the runtime knows no window, engine or clock source.

use std::panic::Location;
use std::rc::Rc;

use crate::reactive::data::{Driver, Extensions, NodeInfo, Runtime};
use crate::reactive::error::UiError;
use crate::reactive::{AppId, NodeId, Owner, OwnerId};

/// What a runtime has queued, as one snapshot ([`Runtime::pending`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pending {
    /// User effects wait for [`Runtime::run_user_effects`].
    pub user: bool,
    /// Render or Internal effects wait for [`Runtime::stabilize`].
    pub render: bool,
    /// Errors wait for [`Runtime::deliver_later`].
    pub later: bool,
    /// A turn was requested of the driver and has not started.
    pub turn_requested: bool,
}

/// The `on_mount` callbacks of a runtime ([`Runtime::mounts`]). The host
/// hands out opaque tokens that only grow (the client uses the revision of
/// the batch that carries the new nodes) and acknowledges them once the
/// nodes are applied.
pub struct Mounts<'a> {
    rt: &'a Runtime,
}

/// Records one turn of the trace ring, from [`Runtime::turn`] until it is
/// finished ([`TurnRecorder::finish`]) or dropped; its duration is the wall
/// time in between. What is recorded between turns goes to the next turn.
#[must_use = "the turn is recorded when its TurnRecorder is finished or dropped"]
pub struct TurnRecorder<'a> {
    rt: &'a Runtime,
    started: std::time::Instant,
    ops: usize,
}

impl Runtime {
    /// Create and register the runtime of a new App with the per-App
    /// `extensions` of the layers above and the `driver` that runs its
    /// turns. App identities are never reused.
    pub fn new(extensions: Extensions, driver: Rc<dyn Driver>) -> Rc<Runtime> {
        let rt = Runtime::register(extensions);
        rt.set_driver(driver);
        rt
    }

    /// Unregister the App: every handle of it becomes `WrongApp` (or
    /// `Disposed`), and the runtime is dropped when the last `Rc` goes.
    pub fn shutdown(&self) {
        Runtime::unregister(self.id);
    }

    /// The runtime whose code is executing on this thread; `None` outside
    /// every App frame, even when Apps are registered.
    pub fn current() -> Option<Rc<Runtime>> {
        crate::reactive::registry::current_runtime()
    }

    /// The registered runtime of `app`, if it still exists. It does not
    /// check which App is running: handle operations do that themselves.
    pub fn get(app: AppId) -> Option<Rc<Runtime>> {
        crate::reactive::registry::try_runtime(app)
    }

    /// The App of this runtime (the App of every handle it made).
    #[inline]
    pub fn id(&self) -> AppId {
        self.id
    }

    /// Create the App's root owner, created at `at`, and record it
    /// ([`Runtime::root`]).
    pub fn create_root(&self, at: &'static Location<'static>) -> Owner {
        let key = self.create_owner(None, at);
        self.root_owner.set(Some(key));
        Owner::from_key(self.id, key)
    }

    /// The handle of owner `id` of this App (an identity
    /// [`Owner::id`] gave out, carried where a handle cannot go, such as the
    /// metadata of a task queued across threads). Nothing is validated until
    /// the handle is used.
    pub fn owner(&self, id: OwnerId) -> Owner {
        Owner::from_key(self.id, id.0)
    }

    /// The App's root owner, once created.
    pub fn root(&self) -> Option<Owner> {
        self.root_owner
            .get()
            .map(|key| Owner::from_key(self.id, key))
    }

    /// What is queued now.
    pub fn pending(&self) -> Pending {
        Pending {
            user: self.has_user_effects(),
            render: self.has_render_effects(),
            later: self.has_later_reports(),
            turn_requested: self.turn_requested(),
        }
    }

    /// The `on_mount` callbacks.
    pub fn mounts(&self) -> Mounts<'_> {
        Mounts { rt: self }
    }

    /// Start recording a turn that delivers `events` host events.
    pub fn turn(&self, events: usize) -> TurnRecorder<'_> {
        self.begin_turn(events);
        TurnRecorder {
            rt: self,
            started: std::time::Instant::now(),
            ops: 0,
        }
    }

    /// Route `err` to the nearest error boundary of `owner`, or, without an
    /// owner or a boundary, to the App's default sink (`tracing` and
    /// [`Runtime::errors`]).
    pub fn report(&self, owner: Option<Owner>, err: UiError) {
        self.route(owner.map(|o| o.key), err);
    }

    /// Route `err` like [`Runtime::report`], at the start of the next turn
    /// (one is requested): for errors raised where no handler may run, such
    /// as code the engine calls while its tree is borrowed. It runs no user
    /// code.
    pub fn report_later(&self, owner: Option<Owner>, err: UiError) {
        self.route_later(owner.map(|o| o.key), err);
    }

    /// Run `f` as host code of this App outside every owner: untracked, not
    /// pure, not setup, with no owner for the owner-relative lookups. The
    /// previous context is restored also on unwind.
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _host = self.handler_frame(None);
        f()
    }

    /// Snapshot of graph node `node` (kind, type name, locations, what it
    /// read), or `None` once it is gone. Values are never exposed.
    pub fn node_info(&self, node: NodeId) -> Option<NodeInfo> {
        self.node_snapshot(node.0)
    }

    /// The Render or Internal effect whose run is executing, if any: the
    /// origin of the properties it sets (diagnostics).
    #[inline]
    pub fn running_effect(&self) -> Option<NodeId> {
        self.current_writer().map(NodeId)
    }

    /// Drop the queued user effects (after a cycle was detected) and return
    /// them.
    pub fn abandon_user_effects(&self) -> Vec<NodeId> {
        self.abandon_user_queue().into_iter().map(NodeId).collect()
    }
}

impl Mounts<'_> {
    /// Make every queued callback without a token wait for `token`; returns
    /// whether any did.
    pub fn defer_until(&self, token: u64) -> bool {
        self.rt.defer_mounts_until(token)
    }

    /// The host acknowledged every token up to `token`: run the callbacks
    /// waiting for one of them, in queue order, for owners that are still
    /// there. Each runs in a handler of its owner; a panic propagates.
    pub fn run_acked(&self, token: u64) {
        self.rt.run_mounts_acked(token);
    }

    /// Whether a callback waits for a token up to `token`.
    pub fn due(&self, token: u64) -> bool {
        self.rt.mounts_due(token)
    }
}

impl TurnRecorder<'_> {
    /// Record what started work in this turn (`"timer"`, an event, ...).
    pub fn cause(&self, cause: String) {
        self.rt.record_cause(cause);
    }

    /// Finish the record with the number of writes committed in the turn and
    /// push it into the ring ([`Runtime::turns`]).
    pub fn finish(mut self, ops: usize) {
        self.ops = ops;
    }
}

impl Drop for TurnRecorder<'_> {
    fn drop(&mut self) {
        let duration_ns = self.started.elapsed().as_nanos() as u64;
        self.rt.end_turn(self.ops, duration_ns);
    }
}
