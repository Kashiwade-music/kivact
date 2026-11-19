//! Role: Host wake boundary, turn stages and reactive turn operations.

use std::panic::Location;
use std::rc::Rc;

use crate::reactive::data::{NodeInfo, Runtime, TurnGuard};
use crate::reactive::detail::CauseTag;
use crate::reactive::error::UiError;
use crate::reactive::host_types::{Stage, TurnState};
use crate::reactive::{AppId, NodeId, Owner, OwnerId};

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
    /// Whether work waits that a turn would run: queued render or user
    /// effects, deferred handlers, deferred error reports, or ready tasks.
    /// Mount callbacks are asked separately (`mounts_due`), with the
    /// revision the host acknowledged.
    pub fn has_work(&self) -> bool {
        self.has_render_effects()
            || self.has_user_effects()
            || !self.deferred.borrow().is_empty()
            || self.has_later_reports()
            || self.tasks.has_ready()
    }

    /// Whether a turn was requested of the host and has not started.
    pub fn turn_requested(&self) -> bool {
        self.state.get() == TurnState::WakeSent
    }

    /// Run `f` for `owner` once the running handler has returned (in the
    /// same turn): a handler frame of `owner`, skipped when the owner is
    /// gone. `at` locates a `Cycle` when deferred work exceeds the turn's
    /// budget. `NodeRef::activate` uses it.
    pub fn defer(&self, owner: Owner, at: &'static Location<'static>, f: Box<dyn FnOnce()>) {
        self.deferred.borrow_mut().push_back((owner.key, at, f));
        self.request_turn();
    }

    #[track_caller]
    fn begin(&self, stage: Stage) -> TurnGuard<'_> {
        assert!(
            !matches!(self.state.get(), TurnState::Running(_)),
            "kivact misuse: invariant: turns cannot be nested at {}",
            Location::caller()
        );
        self.state.set(TurnState::Running(stage));
        let turn = TurnGuard {
            rt: self,
            started: std::time::Instant::now(),
            work_rounds: std::cell::Cell::new(0),
        };
        self.record_stage(stage, turn.started);
        turn
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
    pub fn abandon_user_effects(&self) -> Vec<NodeId> {
        self.abandon_user_queue().into_iter().map(NodeId).collect()
    }
}

impl TurnGuard<'_> {
    /// Record the active turn stage.
    pub fn stage(&self, stage: Stage) {
        self.rt.state.set(TurnState::Running(stage));
        self.rt.record_stage(stage, self.started);
    }
    /// Record a cause of the given kind in the turn trace.
    #[doc(hidden)]
    pub fn cause_with(&self, tag: CauseTag, label: String) {
        self.rt.record_cause_with(tag, label);
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
    /// Run the deferred handlers queued so far (those they defer wait for
    /// the next call). The batch counts as one round of the turn's budget;
    /// over the budget nothing runs and the queue waits for the next turn.
    /// Returns whether any ran.
    pub fn run_deferred(&self) -> bool {
        let first = self.rt.deferred.borrow().front().map(|(_, at, _)| *at);
        let Some(first) = first else { return false };
        if !self.count_round(Some(first)) {
            return false;
        }
        let batch = std::mem::take(&mut *self.rt.deferred.borrow_mut());
        for (owner, _, f) in batch {
            if self.rt.owner_alive(owner) {
                let _frame = self.rt.handler_frame(Some(owner));
                f();
            }
        }
        true
    }
}
