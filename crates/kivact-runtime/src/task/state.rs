//! Role: Task state tracking and completion.
//!
//! Per-App state of the task subsystem (executor, timers, blocking pool,
//! finished tasks, queued actions and their drains), installed as a runtime
//! extension when the App is created. [`TaskState`] is the driver API of the
//! host that runs the App's turns.

use kivact_types::cell::Registry;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::Waker;

use super::executor::Executor;
use crate::task::worker_pool::Pool;
use crate::{Owned, OwnerId};

/// Arrival identity of a registered timer.
pub(crate) type TimerId = u64;

/// The timer table's key: its deadline, then its arrival identity.
pub(crate) type TimerKey = (u64, TimerId);

/// Cancellation handoffs never borrow the timer table.
#[derive(Default)]
pub(crate) struct CancelQueue(kivact_types::cell::LocalMailbox<Vec<TimerKey>>);

impl CancelQueue {
    pub(crate) fn push(&self, key: TimerKey) {
        let mut pending = self.0.take().unwrap_or_default();
        pending.push(key);
        self.0.put(pending);
    }
}

/// One registered host-clock timer.
pub(crate) struct Timer {
    pub(crate) waker: Waker,
    pub(crate) created_at: &'static Location<'static>,
    pub(crate) owner: Option<OwnerId>,
}

/// Pending host-clock timers ordered by deadline and arrival.
#[derive(Default)]
pub(crate) struct Scheduler {
    pub(crate) timers: BTreeMap<(u64, TimerId), Timer>,
    pub(crate) next_id: u64,
    pub(crate) cancel: Rc<CancelQueue>,
}

impl Scheduler {
    fn drain_cancel(&mut self) {
        while let Some(pending) = self.cancel.0.take() {
            for key in pending {
                self.timers.remove(&key);
            }
        }
    }

    pub(crate) fn next_deadline(&mut self) -> Option<u64> {
        self.drain_cancel();
        self.timers.keys().next().map(|k| k.0)
    }

    pub(crate) fn len(&mut self) -> usize {
        self.drain_cancel();
        self.timers.len()
    }

    /// Remove and return wakers of timers due at `now` (ns).
    pub(crate) fn take_due(&mut self, now: u64) -> Vec<Waker> {
        self.drain_cancel();
        let due: Vec<(u64, TimerId)> = self
            .timers
            .range(..=(now, u64::MAX))
            .map(|(k, _)| *k)
            .collect();
        due.into_iter()
            .filter_map(|k| self.timers.remove(&k))
            .map(|timer| timer.waker)
            .collect()
    }
}

/// What a spawned task stores under its owner: its executor task (removing
/// the object cancels the future). The object is removed after the poll that
/// completes the future, by `cancel`, or with its owner.
pub(crate) struct TaskCell {
    /// Set right after the spawn, before the first poll.
    #[expect(
        clippy::disallowed_types,
        reason = "Construction fills this cell before returning or polling; its empty state is inaccessible."
    )]
    pub(super) task: std::cell::OnceCell<super::executor::RawTask>,
}

// xtask-allow(state): legacy: tokio, time, blocking_threads, finished, queues, draining, pool; absent means the default worker count; remotes lists the open mailboxes of other threads and remote_wake marks mail; clocks lists the shared clocks and their users
/// Everything the task subsystem keeps per App, driven by the host that runs
/// the App's turns: it installs the state with its waker (`new`),
/// polls ready futures and fires due timers each turn, and drains and shuts
/// down at exit. Fields drop in declaration order (executor first, the pool
/// last).
pub struct TaskState {
    #[cfg(feature = "tokio")]
    pub(super) tokio: RefCell<super::tokio_state::TokioState>,
    /// The UI-thread executor of spawned futures and action queue workers.
    pub(super) exec: Executor,
    /// Pending host-clock timers.
    pub(super) time: RefCell<Scheduler>,
    /// Worker count of the blocking pool (`None`: the default).
    pub(super) blocking_threads: Option<usize>,
    /// Owned tasks whose future completed; their objects are removed after
    /// the poll that finished them.
    pub(super) finished: RefCell<Vec<Owned<Rc<TaskCell>>>>,
    /// Live queued actions (for the exit drain).
    pub(super) queues: RefCell<Registry<dyn Drain>>,
    /// The UI side of every open `remote` and `remote_trigger` mailbox.
    pub(crate) remotes: RefCell<Vec<Rc<dyn super::remote::Deliver>>>,
    /// The mark and waker other threads use to reach the mailboxes.
    pub(crate) remote_wake: super::remote::RemoteWake,
    /// The shared clocks and their users (`task::clock`).
    pub(super) clocks: RefCell<Vec<Rc<super::clock::ClockEntry>>>,
    /// The blocking pool, created on first use.
    pub(super) pool: RefCell<PoolState>,
}

impl TaskState {
    /// Empty state to install as a runtime extension; `blocking_threads` sizes
    /// the blocking pool when it is created (`None`: `min(4, CPUs)`).
    pub fn new(blocking_threads: Option<usize>, waker: crate::ThreadWaker) -> Self {
        TaskState {
            #[cfg(feature = "tokio")]
            tokio: RefCell::new(super::tokio_state::TokioState::NotStarted),
            remotes: RefCell::new(Vec::new()),
            remote_wake: super::remote::RemoteWake::new(waker.clone()),
            exec: Executor::new(waker),
            time: RefCell::new(Scheduler::default()),
            blocking_threads,
            finished: RefCell::new(Vec::new()),
            queues: RefCell::new(Registry::new()),
            clocks: RefCell::new(Vec::new()),
            pool: RefCell::new(PoolState::NotStarted),
        }
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
    /// next pending timer. `None` when there is none.
    pub fn next_deadline(&self) -> Option<u64> {
        self.time.borrow_mut().next_deadline()
    }

    /// Counts of what the subsystem holds now. Reading them creates nothing
    /// (not the blocking pool).
    pub fn stats(&self) -> TaskStats {
        TaskStats {
            timers: self.time.borrow_mut().len(),
            outstanding_blocking: self.pool.borrow().outstanding(),
        }
    }

    /// The pending timers, earliest deadline first.
    pub fn list(&self) -> Vec<TaskEntry> {
        let mut time = self.time.borrow_mut();
        time.drain_cancel();
        time.timers
            .iter()
            .map(|((deadline, _), timer)| TaskEntry {
                created_at: timer.created_at,
                owner: timer.owner,
                deadline_ns: *deadline,
            })
            .collect()
    }

    /// Stop the blocking pool, waiting up to two seconds per worker.
    pub fn shutdown(&self) {
        #[cfg(feature = "tokio")]
        self.tokio.borrow_mut().shutdown();
        let pool = self.pool.replace(PoolState::ShutDown);
        if let PoolState::Running(p) = pool {
            p.shutdown();
        }
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
    /// Any live queued action still has work: App exit waits for this to
    /// turn false.
    pub fn queues_busy(&self) -> bool {
        self.queues
            .borrow_mut()
            .live()
            .into_iter()
            .any(|a| a.busy())
    }

    /// Abandon every live queued action (the exit deadline passed); returns
    /// how many runs were dropped.
    pub fn abandon_all_queues(&self) -> usize {
        self.queues
            .borrow_mut()
            .live()
            .iter()
            .map(|a| a.abandon())
            .sum()
    }
}

impl TaskState {
    /// The App's blocking pool, created on first use.
    pub(crate) fn blocking_pool(&self) -> Arc<Pool> {
        let mut state = self.pool.borrow_mut();
        match &*state {
            PoolState::Running(pool) => return pool.clone(),
            PoolState::ShutDown => {
                panic!("invariant: no App code runs after the blocking pool shut down")
            }
            PoolState::NotStarted => {}
        }
        let threads = self.blocking_threads.unwrap_or_else(Pool::default_threads);
        let pool = Pool::new(threads);
        *state = PoolState::Running(pool.clone());
        pool
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

/// One pending timer, as [`TaskState::list`] reports it.
pub struct TaskEntry {
    /// Where the timer was created.
    pub created_at: &'static Location<'static>,
    /// The owner that created it.
    pub owner: Option<OwnerId>,
    /// The host-clock deadline (ns).
    pub deadline_ns: u64,
}

/// Lifecycle of the App's lazily constructed blocking pool.
pub(super) enum PoolState {
    NotStarted,
    Running(Arc<Pool>),
    ShutDown,
}
impl PoolState {
    fn outstanding(&self) -> usize {
        match self {
            Self::Running(pool) => pool.outstanding.load(Ordering::SeqCst),
            Self::NotStarted | Self::ShutDown => 0,
        }
    }
}
