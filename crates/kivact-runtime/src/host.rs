//! Role: Runtime construction and task and motion turn operations.
use crate::reactive::host::TurnState;
use crate::{
    DEFAULT_DRAIN_NS, ErrorKind, ExitState, Extensions, Host, MotionState, Owner, Runtime, Stage,
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
struct RegistrationGuard(crate::AppId);
impl Drop for RegistrationGuard {
    fn drop(&mut self) {
        Runtime::unregister(self.0);
    }
}
impl Runtime {
    /// Register an App, build its root setup and install its extensions.
    pub fn new(
        host: Rc<dyn Host>,
        waker: ThreadWaker,
        config: RuntimeConfig,
        build: impl FnOnce(&Rc<Runtime>, Owner) -> Extensions,
    ) -> Rc<Runtime> {
        let rt = Runtime::register(host, config.start_ns);
        let registration = RegistrationGuard(rt.id());
        let root = rt.create_owner(None, config.at);
        assert_eq!(
            root, rt.root,
            "invariant: the root owns the first owner slot"
        );
        let extensions = {
            let _frame = rt.setup_frame(rt.root);
            build(&rt, rt.root())
        };
        let installed = rt
            .ext
            .set(
                extensions
                    .with(TaskState::new(config.blocking_threads, waker))
                    .with(MotionState::default()),
            )
            .is_ok();
        assert!(
            installed,
            "invariant: extensions are installed once during construction"
        );
        rt.state.set(TurnState::Quiet);
        std::mem::forget(registration);
        rt
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
        let tasks = TaskState::of(self);
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
        TaskState::of(self.rt).check_draining(self.rt);
    }
    /// Settle tasks and due timers for at most 1000 rounds.
    pub fn settle_async(&self) {
        self.stage(Stage::Async);
        let rt = self.rt;
        for _ in 0..1000 {
            let wakers = TaskState::of(rt).take_due_timers(rt.now_ns());
            let woke = !wakers.is_empty();
            if woke {
                self.cause("timer".into());
            }
            for w in wakers {
                w.wake();
            }
            let n = TaskState::of(rt).poll_tasks(rt);
            if n > 0 {
                self.cause("task".into());
            }
            self.stabilize();
            if n == 0 && !woke {
                return;
            }
        }
        let tasks = TaskState::of(rt);
        if tasks.has_ready() || tasks.next_deadline().is_some_and(|t| t <= rt.now_ns()) {
            rt.report(None, UiError::new(ErrorKind::Cycle { path: Vec::new() },
                "tasks and timers did not settle within 1000 rounds of one turn; the rest runs in the next turn", None));
        }
    }
    /// Return the motion frame deadline for the next commit.
    pub fn plan_frame(&self) -> Option<u64> {
        MotionState::of(self.rt).plan_frame(self.rt)
    }
    /// Finish recording the turn and wake for remaining work.
    pub fn finish(self, ops: usize, client_pending: bool) -> TurnReport {
        self.stage(Stage::Output);
        let report = self.rt.report_now();
        self.rt
            .end_turn(ops, self.started.elapsed().as_nanos() as u64);
        self.rt.state.set(TurnState::Quiet);
        let pending = self.rt.pending();
        if pending.user || TaskState::of(self.rt).has_ready() || pending.later || client_pending {
            self.rt.request_turn();
        }
        report
    }
    /// Seal setup notes and wake for queued user effects or ready tasks.
    pub fn finish_setup(self) {
        self.rt.finish_setup_notes();
        self.rt.state.set(TurnState::Quiet);
        if self.rt.pending().user || TaskState::of(self.rt).has_ready() {
            self.rt.request_turn();
        }
    }
}
