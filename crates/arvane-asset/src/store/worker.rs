//! Role: The store's worker thread, which runs jobs in order and wakes the host.
//!
//! The store's worker thread: one per store, running jobs in order and
//! waking the host after each.
//!
//! Invariants:
//! - Exactly one thread per worker; jobs run one at a time, in the order
//!   they were submitted.
//! - The host is woken after a result is sent, so a turn that polls after
//!   the wake sees it.
//! - Dropping the worker stops the thread and waits for the job in progress,
//!   up to `SHUTDOWN_WAIT`; a longer job's thread is detached with a warning.
//!   Queued jobs are skipped. The thread owns everything it touches, and its
//!   waker tolerates a host that has already closed.
//! - A job sent to a thread that ended (a panic in a job) resumes that
//!   panic at once: no job is dropped silently.
//! - Jobs run on a thread with a `JOB_STACK` stack, the worker's and, for
//!   an inline store, a helper thread the call waits for
//!   ([`run_on_job_stack`]): usvg and resvg recurse once per element level
//!   (usvg accepts up to 1,024), which overflows a default 1–2 MiB stack
//!   and aborts the process.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::job::{self, Done, Job};

/// Stop before a DLL unload, without letting a stuck decode block closing forever.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);

/// The stack of the threads that run jobs (32 MiB, reserved address space;
/// only the pages a job touches are committed).
const JOB_STACK: usize = 32 * 1024 * 1024;

/// Runs `job` to completion on a helper thread with a `JOB_STACK` stack,
/// waiting for it (an inline store's jobs). Runs it on the calling thread
/// when the system cannot start one.
pub(crate) fn run_on_job_stack(job: Job) -> Done {
    let job = std::sync::Mutex::new(Some(job));
    let take = || {
        job.lock()
            .ok()
            .and_then(|mut j| j.take())
            .expect("invariant: the job is taken once, by the thread that runs it")
    };
    std::thread::scope(|scope| {
        let helper = std::thread::Builder::new()
            .name("arvane-asset-inline".to_owned())
            .stack_size(JOB_STACK)
            .spawn_scoped(scope, || job::run(take()));
        match helper.map(|h| h.join()) {
            Ok(Ok(done)) => done,
            // A panic in the job is a bug: let it reach the caller as it
            // would inline.
            Ok(Err(panic)) => std::panic::resume_unwind(panic),
            Err(_) => job::run(take()),
        }
    })
}

/// Wakes the host when the worker thread ends, by return or by unwind, so
/// the host polls and sees a thread that died.
struct WakeOnExit<W: Fn()>(W);
impl<W: Fn()> Drop for WakeOnExit<W> {
    fn drop(&mut self) {
        (self.0)();
    }
}

pub(crate) struct Worker {
    /// `None` only in `Drop` (closing the queue ends the thread's wait).
    jobs: Option<Sender<Job>>,
    done: Receiver<Done>,
    stop: Arc<AtomicBool>,
    /// `None` once joined (a resumed panic, or `Drop`).
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Starts the thread; `Err` when the system cannot start one.
    pub(crate) fn spawn(waker: impl Fn() + Send + 'static) -> std::io::Result<Worker> {
        let (jobs, job_rx) = mpsc::channel::<Job>();
        let (done_tx, done) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let thread = std::thread::Builder::new()
            .name("arvane-asset".to_owned())
            .stack_size(JOB_STACK)
            .spawn(move || {
                let wake = WakeOnExit(waker);
                while let Ok(job) = job_rx.recv() {
                    if stopped.load(Ordering::Acquire) {
                        break;
                    }
                    let done = job::run(job);
                    if stopped.load(Ordering::Acquire) || done_tx.send(done).is_err() {
                        break;
                    }
                    (wake.0)();
                }
            })?;
        Ok(Worker {
            jobs: Some(jobs),
            done,
            stop,
            thread: Some(thread),
        })
    }

    pub(crate) fn submit(&mut self, job: Job) {
        let jobs = self
            .jobs
            .as_ref()
            .expect("invariant: the queue closes only in Drop");
        if jobs.send(job).is_err() {
            self.resume_panic();
        }
    }

    /// The next finished job, if any. A worker thread that ended on a panic
    /// resumes it here, on the host thread.
    pub(crate) fn try_recv(&mut self) -> Option<Done> {
        match self.done.try_recv() {
            Ok(done) => Some(done),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => self.resume_panic(),
        }
    }

    /// The thread ended before `Drop`, which only a panic in a job does:
    /// resume that panic here, on the host thread (it ends the App).
    fn resume_panic(&mut self) -> ! {
        let thread = self
            .thread
            .take()
            .expect("invariant: a resumed worker panic ends the App, so it is resumed once");
        match thread.join() {
            Err(payload) => std::panic::resume_unwind(payload),
            Ok(()) => {
                unreachable!("invariant: the asset worker stops only when dropped or on a panic")
            }
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Closing the queue ends the thread's wait once it is idle.
        self.jobs = None;
        if let Some(thread) = self.thread.take() {
            let started = Instant::now();
            while !thread.is_finished() && started.elapsed() < SHUTDOWN_WAIT {
                std::thread::sleep(Duration::from_millis(1));
            }
            if thread.is_finished() {
                if let Err(payload) = thread.join()
                    && !std::thread::panicking()
                {
                    std::panic::resume_unwind(payload);
                }
            } else {
                crate::detached::DETACHED.store(true, Ordering::Release);
                tracing::warn!(target: "kivact", "asset job still running after the host closed; its thread is detached");
            }
        }
    }
}
