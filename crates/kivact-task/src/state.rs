//! Role: Task state tracking and completion.
//!
//! Per-App state of the task subsystem (executor, timers, blocking pool,
//! finished tasks, queued actions and their drains), installed as a runtime
//! extension when the App is created. [`TaskState`] is the driver API of the
//! host that runs the App's turns.

use std::cell::{OnceCell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::Waker;

use super::executor::Executor;
use crate::worker_pool::{self, Pool};
use kivact_reactive::{ErrorKind, UiError};
use kivact_reactive::{Owned, Runtime};

/// The pending host-clock timers of an App, by deadline and arrival.
#[derive(Default)]
pub(crate) struct TimeState {
    pub(crate) timers: BTreeMap<(u64, u64), Waker>,
    pub(crate) next_id: u64,
}

impl TimeState {
    pub(crate) fn next_deadline(&self) -> Option<u64> {
        self.timers.keys().next().map(|k| k.0)
    }

    pub(crate) fn len(&self) -> usize {
        self.timers.len()
    }

    /// Remove and return wakers of timers due at `now` (ns).
    pub(crate) fn take_due(&mut self, now: u64) -> Vec<Waker> {
        let due: Vec<(u64, u64)> = self.timers.range(..(now + 1, 0)).map(|(k, _)| *k).collect();
        due.into_iter()
            .filter_map(|k| self.timers.remove(&k))
            .collect()
    }
}

/// What a spawned task stores under its owner: its executor task (removing
/// the object cancels the future). The object is removed after the poll that
/// completes the future, by `cancel`, or with its owner.
pub(crate) struct TaskCell {
    /// Set right after the spawn, before the first poll.
    pub(super) task: OnceCell<super::executor::RawTask>,
}

/// Everything the task subsystem keeps per App, driven by the host that runs
/// the App's turns: it installs the state (`new`), wakes on `set_waker`,
/// polls ready futures and fires due timers each turn, and drains and shuts
/// down at exit. Fields drop in declaration order (executor first, the pool
/// last).
pub struct TaskState {
    /// The UI-thread executor of spawned futures and action queue workers.
    pub(super) exec: Executor,
    /// Pending host-clock timers.
    pub(super) time: RefCell<TimeState>,
    /// Worker count of the blocking pool (`None`: the default).
    pub(super) blocking_threads: Option<usize>,
    /// Owned tasks whose future completed; their objects are removed after
    /// the poll that finished them.
    pub(super) finished: RefCell<Vec<Owned<TaskCell>>>,
    /// Live actions with a queue policy (for the exit drain).
    pub(super) queues: RefCell<Vec<Weak<dyn Drain>>>,
    /// Queued actions of disposed owners still draining, with their deadline (ns).
    pub(super) draining: RefCell<Vec<(Rc<dyn Drain>, u64)>>,
    /// The blocking pool, created on first use.
    fallback_threads: RefCell<Vec<std::thread::JoinHandle<()>>>,
    pub(super) pool: RefCell<Option<Arc<Pool>>>,
}

impl TaskState {
    /// Empty state to install as a runtime extension; `blocking_threads` sizes
    /// the blocking pool when it is created (`None`: `min(4, CPUs)`).
    pub fn new(blocking_threads: Option<usize>) -> Self {
        TaskState {
            exec: Executor::new(),
            time: RefCell::new(TimeState::default()),
            blocking_threads,
            finished: RefCell::new(Vec::new()),
            queues: RefCell::new(Vec::new()),
            draining: RefCell::new(Vec::new()),
            pool: RefCell::new(None),
            fallback_threads: RefCell::new(Vec::new()),
        }
    }

    /// The task state installed in `rt`'s App.
    #[inline]
    pub fn of(rt: &Runtime) -> &TaskState {
        rt.ext::<TaskState>()
    }

    /// Call `waker` (from any thread) when a future becomes ready, so the host
    /// runs a turn. The waker should only schedule that turn.
    pub fn set_waker(&self, waker: Arc<dyn Fn() + Send + Sync>) {
        self.exec.set_waker(waker);
    }

    /// Whether a spawned future is ready to be polled (the next turn has
    /// tasks to run).
    pub fn has_ready(&self) -> bool {
        self.exec.has_ready()
    }

    /// Remove and return the wakers of the timers due at `now` (ns). The host
    /// wakes them, then polls the tasks they ready.
    pub fn take_due_timers(&self, now: u64) -> Vec<Waker> {
        self.time.borrow_mut().take_due(now)
    }

    /// When the host must run a turn next for the task subsystem (ns): the
    /// earlier of the next pending timer and the next drain deadline of a
    /// draining action. `None` when neither exists.
    pub fn next_deadline(&self) -> Option<u64> {
        let timer = self.time.borrow().next_deadline();
        let drain = self.draining.borrow().iter().map(|(_, d)| *d).min();
        [timer, drain].into_iter().flatten().min()
    }

    /// Counts of what the subsystem holds now. Reading them creates nothing
    /// (not the blocking pool).
    pub fn stats(&self) -> TaskStats {
        TaskStats {
            timers: self.time.borrow().len(),
            outstanding_blocking: self
                .pool
                .borrow()
                .as_ref()
                .map(|p| p.outstanding.load(Ordering::SeqCst))
                .unwrap_or(0),
        }
    }

    /// Stop the blocking pool, waiting up to two seconds per worker. A later
    /// blocking job creates a new pool.
    pub fn shutdown(&self) {
        let pool = self.pool.borrow_mut().take();
        if let Some(p) = pool {
            p.shutdown();
        }
        worker_pool::wait_for_threads(self.fallback_threads.take());
    }
}

/// Type-erased view of an action for drain bookkeeping.
pub(crate) trait Drain {
    /// A run is in flight or waiting.
    fn busy(&self) -> bool;
    /// Drop the running run and the waiting ones; returns how many were dropped.
    fn abandon(&self) -> usize;
}

impl TaskState {
    /// Drop drained actions and abandon those past their deadline (reported
    /// to the App's default sink). The host calls it each turn.
    pub fn check_draining(&self, rt: &Runtime) {
        let now = rt.now_ns();
        let list = std::mem::take(&mut *self.draining.borrow_mut());
        let mut keep = Vec::new();
        for (a, deadline) in list {
            if !a.busy() {
                continue;
            }
            if now >= deadline {
                let n = a.abandon();
                rt.report(
                    None,
                    UiError::new(
                        ErrorKind::Closed,
                        format!("action drain deadline passed; {n} run(s) abandoned"),
                        None,
                    ),
                );
                continue;
            }
            keep.push((a, deadline));
        }
        self.draining.borrow_mut().extend(keep);
    }

    /// Any queued action (live or draining) still has work: the host waits for
    /// this to turn false before it exits.
    pub fn queues_busy(&self) -> bool {
        self.draining.borrow().iter().any(|(a, _)| a.busy())
            || self
                .queues
                .borrow()
                .iter()
                .filter_map(|w| w.upgrade())
                .any(|a| a.busy())
    }

    /// Abandon every live and draining queued action (the exit deadline
    /// passed); returns how many runs were dropped.
    pub fn abandon_all_queues(&self) -> usize {
        let all: Vec<Rc<dyn Drain>> = {
            let mut v: Vec<Rc<dyn Drain>> = self
                .draining
                .borrow_mut()
                .drain(..)
                .map(|(a, _)| a)
                .collect();
            v.extend(self.queues.borrow().iter().filter_map(|w| w.upgrade()));
            v
        };
        all.iter().map(|a| a.abandon()).sum()
    }
}

impl TaskState {
    pub(crate) fn retain_fallback(&self, handle: std::thread::JoinHandle<()>) {
        self.fallback_threads.borrow_mut().push(handle);
    }

    /// The App's blocking pool, created on first use.
    pub(crate) fn blocking_pool(&self) -> Arc<Pool> {
        if let Some(p) = self.pool.borrow().as_ref() {
            return p.clone();
        }
        let threads = self.blocking_threads.unwrap_or_else(Pool::default_threads);
        let p = Pool::new(threads);
        *self.pool.borrow_mut() = Some(p.clone());
        p
    }
}

/// Counts of what a [`TaskState`] holds, for diagnostics and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskStats {
    /// Pending host-clock timers.
    pub timers: usize,
    /// Blocking jobs submitted and not yet finished.
    pub outstanding_blocking: usize,
}
