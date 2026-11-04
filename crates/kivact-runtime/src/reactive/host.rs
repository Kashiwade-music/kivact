//! Role: Host wake boundary, turn stages and reactive turn operations.

use std::panic::Location;
use std::rc::Rc;

use crate::reactive::data::{NodeInfo, Runtime};
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

/// The host wake boundary of an App.
pub trait Host: 'static {
    /// Request a turn soon on the Quiet to WakeSent edge.
    fn wake(&self);
}
/// A wake usable from any thread.
#[derive(Clone)]
pub struct ThreadWaker(std::sync::Arc<dyn Fn() + Send + Sync>);
impl ThreadWaker {
    /// Create a wake callback usable from any thread.
    pub fn new(f: impl Fn() + Send + Sync + 'static) -> Self {
        Self(std::sync::Arc::new(f))
    }
    /// Invoke the wake callback.
    pub fn wake(&self) {
        (self.0)();
    }
}
/// The operation currently performed in a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// App setup.
    Setup,
    /// Turn entry.
    Begin,
    /// Deferred error reports.
    Reports,
    /// Host event delivery.
    Dispatch,
    /// Tasks, timers and drains.
    Async,
    /// Engine write commit.
    Commit,
    /// Acknowledged mount callbacks.
    Mounts,
    /// User effect execution.
    UserEffects,
    /// Turn output and scheduling.
    Output,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TurnState {
    Quiet,
    WakeSent,
    Running(Stage),
}
/// The App's exit drain status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitState {
    /// No exit was requested.
    Running,
    /// Queued actions drain until the deadline in nanoseconds.
    Draining {
        /// Drain deadline in nanoseconds.
        deadline: u64,
    },
    /// The host may exit.
    Exited,
}
/// Scheduling result of a completed turn.
pub struct TurnReport {
    /// Earliest timer or drain deadline in nanoseconds.
    pub next_deadline: Option<u64>,
    /// Current exit status.
    pub exit: ExitState,
}
/// A running turn and its trace record.
#[must_use]
pub struct TurnGuard<'a> {
    pub(crate) rt: &'a Runtime,
    pub(crate) started: std::time::Instant,
}
impl Runtime {
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

    /// The handle of owner `id` of this App (an identity
    /// [`Owner::id`] gave out, carried where a handle cannot go, such as the
    /// metadata of a task queued across threads). Nothing is validated until
    /// the handle is used.
    pub fn owner(&self, id: OwnerId) -> Owner {
        Owner::from_key(self.id, id.0)
    }

    /// The App's root owner.
    pub fn root(&self) -> Owner {
        Owner::from_key(self.id, self.root)
    }
    /// Snapshot of queued reactive work.
    pub fn pending(&self) -> Pending {
        Pending {
            user: self.has_user_effects(),
            render: self.has_render_effects(),
            later: self.has_later_reports(),
            turn_requested: self.state.get() == TurnState::WakeSent,
        }
    }
    #[track_caller]
    fn begin(&self, stage: Stage) -> TurnGuard<'_> {
        assert!(
            !matches!(self.state.get(), TurnState::Running(_)),
            "kivact misuse: invariant: turns cannot be nested at {}",
            Location::caller()
        );
        self.state.set(TurnState::Running(stage));
        TurnGuard {
            rt: self,
            started: std::time::Instant::now(),
        }
    }
    #[track_caller]
    /// Begin the setup turn; turns cannot be nested.
    pub fn begin_setup(&self) -> TurnGuard<'_> {
        self.begin(Stage::Setup)
    }
    #[track_caller]
    /// Begin a turn containing the given number of host events.
    pub fn begin_turn(&self, events: usize) -> TurnGuard<'_> {
        let turn = self.begin(Stage::Begin);
        self.begin_turn_record(events);
        turn
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
    /// Abandon queued user effects after a cycle.
    pub fn abandon_user_effects(&self) -> Vec<NodeId> {
        self.abandon_user_queue().into_iter().map(NodeId).collect()
    }
}

impl TurnGuard<'_> {
    /// Record the active turn stage.
    pub fn stage(&self, stage: Stage) {
        self.rt.state.set(TurnState::Running(stage));
    }
    /// Record a cause in the turn trace.
    pub fn cause(&self, cause: String) {
        self.rt.record_cause(cause);
    }
    /// Run render and internal effects until stable.
    pub fn stabilize(&self) {
        self.rt.stabilize();
    }
    /// Deliver the initial snapshot of deferred reports.
    pub fn deliver_reports(&self) {
        self.stage(Stage::Reports);
        self.rt.deliver_later();
    }
    /// Run the initial snapshot of acknowledged mount callbacks.
    pub fn run_mounts(&self, acked: kivact_types::Revision) {
        self.stage(Stage::Mounts);
        self.rt.run_mounts_acked(acked);
    }
    #[track_caller]
    /// Stamp pending mount callbacks during Commit.
    pub fn stamp_mounts(&self, revision: kivact_types::Revision) {
        assert_eq!(
            self.rt.state.get(),
            TurnState::Running(Stage::Commit),
            "kivact misuse: invariant: mount stamps require Commit at {}",
            Location::caller()
        );
        self.rt.stamp_mounts(revision);
    }
    /// Run one round of queued user effects.
    pub fn run_user_effects(&self) -> bool {
        self.stage(Stage::UserEffects);
        self.rt.run_user_effects()
    }
    /// Abandon queued user effects after a cycle.
    pub fn abandon_user_effects(&self) -> Vec<NodeId> {
        self.rt.abandon_user_effects()
    }
}
