//! Role: Runtime construction and task and motion turn operations.
use crate::reactive::host_types::TurnState;
use crate::{
    CauseTag, DEFAULT_DRAIN_NS, ErrorKind, ExitState, Host, MotionEnv, Owner, Runtime, Stage,
    TaskState, ThreadWaker, TurnGuard, TurnReport, UiError,
};
use std::panic::Location;
use std::rc::Rc;
/// Host configuration at App startup.
pub struct RuntimeConfig {
    /// The host clock when the App starts.
    pub start_ns: u64,
    /// Worker count of the blocking pool; None uses the default.
    pub blocking_threads: Option<usize>,
    /// Source location of the root owner.
    pub at: &'static Location<'static>,
}
/// What the host layer builds during `Runtime::new` (in the root's setup):
/// its own per-App state, read back with [`Runtime::app_state`], and the
/// environment values the motion system reads.
pub struct AppParts {
    /// The host layer's per-App state (one value; it downcasts it).
    pub state: Box<dyn std::any::Any>,
    /// Reduced motion, drawability and the frame period.
    pub motion_env: MotionEnv,
}

struct RegistrationGuard(crate::AppId);
impl Drop for RegistrationGuard {
    fn drop(&mut self) {
        Runtime::unregister(self.0);
    }
}
impl Runtime {
    /// Register an App, build its root setup and install its app parts.
    pub fn new(
        host: Rc<dyn Host>,
        waker: ThreadWaker,
        config: RuntimeConfig,
        build: impl FnOnce(&Rc<Runtime>, Owner) -> AppParts,
    ) -> Rc<Runtime> {
        let tasks = TaskState::new(config.blocking_threads, waker);
        let rt = Runtime::register(host, config.start_ns, tasks);
        let registration = RegistrationGuard(rt.id());
        let root = rt.create_owner(None, config.at);
        assert_eq!(
            root, rt.root,
            "invariant: the root owns the first owner slot"
        );
        let parts = {
            let _frame = rt.setup_frame(rt.root);
            build(&rt, rt.root())
        };
        rt.app.get_or_init(|| crate::reactive::AppSlot {
            motion_env: parts.motion_env,
            state: parts.state,
        });
        rt.state.set(TurnState::Quiet);
        std::mem::forget(registration);
        rt
    }

    /// The app slot. Design 6.2 names `Runtime.app` as the one exception to
    /// "no object filled later": it is set once inside `Runtime::new`, before
    /// any user code runs (12 章 ST-02). Reading it while the App is still
    /// being constructed is a bug of the host layer.
    fn app_slot(&self) -> &crate::reactive::AppSlot {
        debug_assert!(
            self.app.get().is_some(),
            "invariant: the app parts are read while the App is being constructed (Runtime::new has not installed them)"
        );
        self.app
            .get()
            .expect("invariant: Runtime::new installs the app parts before it returns")
    }

    /// The host layer's per-App state given by `AppParts::state`.
    pub fn app_state(&self) -> &dyn std::any::Any {
        self.app_slot().state.as_ref()
    }

    /// The motion environment given by `AppParts::motion_env`.
    pub(crate) fn motion_env(&self) -> &MotionEnv {
        &self.app_slot().motion_env
    }

    /// The tasks, timers and blocking pool of this App.
    pub fn tasks(&self) -> &TaskState {
        &self.tasks
    }

    /// The frame loops and animations of this App.
    pub fn motion(&self) -> &crate::motion::MotionState {
        &self.motion
    }
    /// Begin the exit drain, retaining the first requested deadline.
    pub fn request_exit(&self) {
        if self.exit.get() == ExitState::Running {
            self.exit.set(ExitState::Draining {
                deadline: self.now_ns().saturating_add(DEFAULT_DRAIN_NS),
            });
        }
    }
    /// Update exit state and return scheduling output without a turn.
    pub fn report_now(&self) -> TurnReport {
        if matches!(self.state.get(), TurnState::Running(_)) {
            self.state.set(TurnState::Running(Stage::Output));
        }
        let tasks = self.tasks();
        let exit = match self.exit.get() {
            ExitState::Draining { .. } if !tasks.queues_busy() => ExitState::Exited,
            ExitState::Draining { deadline } if self.now_ns() >= deadline => {
                let n = tasks.abandon_all_queues();
                self.report(
                    None,
                    UiError::new(
                        ErrorKind::Closed,
                        format!("exit drain deadline passed; {n} queued run(s) abandoned"),
                        None,
                    ),
                );
                ExitState::Exited
            }
            phase => phase,
        };
        self.exit.set(exit);
        let next_deadline = match exit {
            ExitState::Draining { deadline } => {
                Some(tasks.next_deadline().map_or(deadline, |t| t.min(deadline)))
            }
            _ => tasks.next_deadline(),
        };
        TurnReport {
            next_deadline,
            exit,
        }
    }
}
impl TurnGuard<'_> {
    /// Release drained actions and abandon expired drains.
    pub fn check_draining(&self) {
        self.stage(Stage::Async);
        self.rt.tasks().check_draining(self.rt);
    }
    /// Settle tasks and due timers for at most 1000 rounds.
    pub fn settle_async(&self) {
        self.stage(Stage::Async);
        let rt = self.rt;
        for _ in 0..1000 {
            let wakers = rt.tasks().take_due_timers(rt.now_ns());
            let woke = !wakers.is_empty();
            if woke {
                self.cause_with(CauseTag::Timer, "timer".into());
            }
            for w in wakers {
                w.wake();
            }
            let n = rt.tasks().poll_tasks(rt);
            if n > 0 {
                self.cause_with(CauseTag::Task, "task".into());
            }
            self.stabilize();
            if n == 0 && !woke {
                return;
            }
        }
        let tasks = rt.tasks();
        if tasks.has_ready() || tasks.next_deadline().is_some_and(|t| t <= rt.now_ns()) {
            rt.report(None, UiError::new(ErrorKind::Cycle { path: Vec::new() },
                "tasks and timers did not settle within 1000 rounds of one turn; the rest runs in the next turn", None));
        }
    }
    /// Return the motion frame deadline for the next commit.
    pub fn plan_frame(&self) -> Option<u64> {
        self.rt.motion().plan_frame(self.rt)
    }
    /// Finish recording the turn and wake for remaining work.
    pub fn finish(self, ops: usize, client_pending: bool) -> TurnReport {
        self.stage(Stage::Output);
        let report = self.rt.report_now();
        self.rt
            .end_turn(ops, self.started.elapsed().as_nanos() as u64);
        self.rt.state.set(TurnState::Quiet);
        let pending = self.rt.pending();
        if pending.user
            || pending.render
            || self.rt.tasks().has_ready()
            || pending.later
            || client_pending
        {
            self.rt.request_turn();
        }
        report
    }
    /// Seal setup notes and wake for queued effects or ready tasks.
    pub fn finish_setup(self) {
        self.rt.finish_setup_notes();
        self.rt.state.set(TurnState::Quiet);
        let pending = self.rt.pending();
        if pending.user || pending.render || self.rt.tasks().has_ready() {
            self.rt.request_turn();
        }
    }
}
