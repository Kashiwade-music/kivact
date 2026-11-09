//! Role: Worker threads that run blocking jobs, and their shutdown.

use std::any::Any;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{JoinHandle, ThreadId};
use std::time::{Duration, Instant};

/// Bound DLL-unload cleanup without waiting forever for uncooperative jobs.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);

pub(crate) type Job = Box<dyn FnOnce() + Send>;

pub(crate) fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    // A worker never holds a pool lock while it runs a job, so a poisoned
    // lock means a panic already happened elsewhere.
    m.lock()
        .expect("blocking pool lock poisoned: a panic already happened")
}

/// Lifecycle of the pool's worker threads and their job channel.
pub(crate) enum Workers {
    /// No worker thread could be started.
    Unspawned,
    /// The workers receive jobs through `tx`.
    Running {
        tx: mpsc::Sender<Job>,
        handles: Vec<JoinHandle<()>>,
    },
    /// The pool stopped accepting jobs.
    Closed,
}

/// Fixed pool of worker threads (default `min(4, CPUs)`).
pub(crate) struct Pool {
    workers: Mutex<Workers>,
    /// Threads started for jobs when no worker could take them.
    fallbacks: Mutex<Vec<JoinHandle<()>>>,
    // xtask-allow(state-fields): statistic: the number of submitted jobs not yet finished, which the shutdown waits on
    pub(crate) outstanding: Arc<AtomicUsize>,
}

impl Pool {
    pub(crate) fn new(threads: usize) -> Arc<Pool> {
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));
        let mut handles = Vec::new();
        for i in 0..threads.max(1) {
            let rx = rx.clone();
            let r = std::thread::Builder::new()
                .name(format!("kivact-blocking-{i}"))
                .spawn(move || {
                    loop {
                        let job = lock(&rx).recv();
                        match job {
                            Ok(job) => job(),
                            Err(_) => break,
                        }
                    }
                });
            if let Ok(handle) = r {
                handles.push(handle);
            }
        }
        let workers = if handles.is_empty() {
            Workers::Unspawned
        } else {
            Workers::Running { tx, handles }
        };
        Arc::new(Pool {
            workers: Mutex::new(workers),
            fallbacks: Mutex::new(Vec::new()),
            outstanding: Arc::new(AtomicUsize::new(0)),
        })
    }

    pub(crate) fn default_threads() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .min(4)
    }

    pub(crate) fn submit(&self, job: Job) {
        let sent = {
            let workers = lock(&self.workers);
            match &*workers {
                Workers::Running { tx, .. } => tx.send(job).map_err(|e| e.0),
                Workers::Unspawned | Workers::Closed => Err(job),
            }
        };
        if let Err(job) = sent {
            // No worker available: retain the fallback for App shutdown.
            if let Ok(handle) = std::thread::Builder::new()
                .name("kivact-blocking-fallback".into())
                .spawn(job)
            {
                let mut fallbacks = lock(&self.fallbacks);
                // Finished fallbacks need no join; keep the list bounded.
                fallbacks.retain(|h| !h.is_finished());
                fallbacks.push(handle);
            }
        }
    }

    /// Join the dead worker thread `id` and take the payload of the panic
    /// that ended it; `None` if the thread is not (or no longer) here.
    pub(crate) fn take_panic(&self, id: ThreadId) -> Option<Box<dyn Any + Send>> {
        let from_workers = {
            let mut workers = lock(&self.workers);
            match &mut *workers {
                Workers::Running { handles, .. } => handles
                    .iter()
                    .position(|h| h.thread().id() == id)
                    .map(|at| handles.swap_remove(at)),
                Workers::Unspawned | Workers::Closed => None,
            }
        };
        let handle = match from_workers {
            Some(handle) => handle,
            None => {
                let mut fallbacks = lock(&self.fallbacks);
                let at = fallbacks.iter().position(|h| h.thread().id() == id)?;
                fallbacks.swap_remove(at)
            }
        };
        handle.join().err()
    }

    /// Stop accepting jobs; idle workers exit.
    pub(crate) fn shutdown(&self) {
        let handles = match std::mem::replace(&mut *lock(&self.workers), Workers::Closed) {
            Workers::Running { handles, .. } => handles,
            Workers::Unspawned | Workers::Closed => Vec::new(),
        };
        wait_for_threads(handles);
        wait_for_threads(std::mem::take(&mut *lock(&self.fallbacks)));
    }
}

/// Join completed jobs and warn before detaching those beyond their deadline.
/// A job that panicked and was never awaited resumes its panic here, unless
/// this thread is already panicking.
pub(crate) fn wait_for_threads(handles: Vec<JoinHandle<()>>) {
    let mut detached = 0;
    for handle in handles {
        let start = Instant::now();
        while !handle.is_finished() && start.elapsed() < SHUTDOWN_WAIT {
            std::thread::sleep(Duration::from_millis(1));
        }
        if handle.is_finished() {
            if let Err(payload) = handle.join()
                && !std::thread::panicking()
            {
                std::panic::resume_unwind(payload);
            }
        } else {
            kivact_types::process::DetachedThreads::note();
            detached += 1;
        }
    }
    if detached > 0 {
        tracing::warn!(target: "kivact", threads = detached,
            "blocking jobs still running after the App closed; their threads are detached");
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        self.shutdown();
    }
}
