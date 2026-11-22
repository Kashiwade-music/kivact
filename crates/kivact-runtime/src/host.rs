//! Role: Runtime construction and task and motion turn operations.
use crate::driver::{
    ExitState, Host, MotionEnv, MotionState, RecordBits, Runtime, TaskState, ThreadWaker,
    TurnGuard, TurnReport,
};
use crate::reactive::Extensions;
use crate::reactive::extensions::TurnWork;
use crate::reactive::host_types::TurnState;
use crate::{Cause, CauseTag, ErrorKind, Owner, Stage, UiError};
use std::panic::Location;
use std::rc::Rc;

/// How long App exit waits for queued actions to finish (ns).
const EXIT_DRAIN_NS: u64 = 3_000_000_000;

/// The work rounds (deferred handlers, timers and tasks) one turn may
/// run, shared by all its settles (12 章 ST-03).
const ASYNC_ROUNDS: u32 = 1000;
/// Host configuration at App startup.
pub struct RuntimeConfig {
    /// The host clock when the App starts.
    pub start_ns: u64,
    /// Worker count of the blocking pool; None uses the default.
    pub blocking_threads: Option<usize>,
    /// Source location of the root owner.
    pub at: &'static Location<'static>,
    /// The recorders on from the start (a test screen records its whole run).
    pub record: RecordBits,
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
        let tasks = Rc::new(TaskState::new(config.blocking_threads, waker));
        let extensions = Extensions::new()
            .with(Rc::clone(&tasks))
            .with(tasks as Rc<dyn TurnWork>)
            .with(MotionState::default());
        let rt = Runtime::register(host, config.start_ns, extensions);
        let registration = RegistrationGuard(rt.id());
        rt.set_recorder(config.record, true);
        let root = rt.create_owner(None, config.at);
        assert_eq!(
            root, rt.root,
            "invariant: the root owns the first owner slot"
        );
        let parts = {
            let _frame = rt.setup_frame(rt.root);
            build(&rt, rt.root())
        };
        MotionState::of(&rt).set_env(parts.motion_env);
        rt.app
            .get_or_init(|| crate::reactive::AppSlot { state: parts.state });
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
        MotionState::of(self).env()
    }

    /// The tasks, timers and blocking pool of this App.
    pub fn tasks(&self) -> &TaskState {
        TaskState::of(self)
    }

    /// The frame loops and animations of this App.
    pub fn motion(&self) -> &MotionState {
        MotionState::of(self)
    }
    /// Begin the exit drain, retaining the first requested deadline.
    pub fn request_exit(&self) {
        if self.exit.get() == ExitState::Running {
            self.exit.set(ExitState::Draining {
                deadline: self.now_ns().saturating_add(EXIT_DRAIN_NS),
            });
        }
    }
    /// End a host call that ran no turn (nothing to do): the App is quiet
    /// again and asks for a turn if work is queued. Every early return of
    /// the host's turn goes through it.
    pub fn end_idle(&self) -> TurnReport {
        self.state.set(TurnState::Quiet);
        let report = self.exit_report();
        if self.has_work() {
            self.request_turn();
        }
        report
    }

    /// Update exit state and return scheduling output (private: only the
    /// turn's end and `end_idle` report).
    fn exit_report(&self) -> TurnReport {
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
    /// Count one work round of this turn. Returns `false` when the budget
    /// is used up: the work that would run waits for the next turn (it stays
    /// queued, so `finish` requests one). The first round over the budget
    /// reports one `Cycle` located at `at`.
    pub(crate) fn count_round(&self, at: Option<&'static Location<'static>>) -> bool {
        let rounds = self.work_rounds.get();
        if rounds > ASYNC_ROUNDS {
            return false;
        }
        self.work_rounds.set(rounds + 1);
        if rounds == ASYNC_ROUNDS {
            self.rt.report(
                None,
                UiError::new(
                    ErrorKind::Cycle {
                        path: at.into_iter().collect(),
                    },
                    "deferred work, tasks and timers did not settle within 1000 rounds of one turn; the rest runs in the next turn",
                    at,
                ),
            );
            return false;
        }
        true
    }

    /// One round of due timers and ready tasks. Returns whether any ran.
    fn poll_async_round(&self) -> bool {
        let rt = self.rt;
        if self.work_rounds.get() >= ASYNC_ROUNDS {
            return false;
        }
        self.stage(Stage::Async);
        let wakers = rt.tasks().take_due_timers(rt.now_ns());
        let woke = !wakers.is_empty();
        if woke {
            self.cause(Cause {
                tag: CauseTag::Timer,
                event: "timer",
                node: None,
            });
        }
        for w in wakers {
            w.wake();
        }
        let polled = rt.tasks().poll_tasks(rt);
        if polled > 0 {
            self.cause(Cause {
                tag: CauseTag::Task,
                event: "task",
                node: None,
            });
        }
        let ran = woke || polled > 0;
        if ran {
            // The round already ran; at the limit it reports once.
            self.count_round(None);
        }
        ran
    }

    /// The only settle: effects, then deferred handlers, then one round of
    /// timers and tasks, until none of them has work.
    pub fn settle(&self) {
        loop {
            self.stabilize();
            let deferred = self.run_deferred();
            let polled = self.poll_async_round();
            if !deferred && !polled {
                break;
            }
        }
    }
    /// Return the motion frame deadline for the next commit.
    pub fn plan_frame(&self) -> Option<u64> {
        self.rt.motion().plan_frame(self.rt)
    }
    /// Finish recording the turn and wake for remaining work.
    pub fn finish(self, ops: usize) -> TurnReport {
        self.stage(Stage::Output);
        let report = self.rt.exit_report();
        self.rt
            .end_turn(ops, self.started.elapsed().as_nanos() as u64);
        self.rt.state.set(TurnState::Quiet);
        if self.rt.has_work() {
            self.rt.request_turn();
        }
        report
    }
    /// Seal setup notes and wake for queued effects or ready tasks.
    pub fn finish_setup(self) {
        self.rt.finish_setup_notes();
        self.rt.state.set(TurnState::Quiet);
        if self.rt.has_work() {
            self.rt.request_turn();
        }
    }
}
