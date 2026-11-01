//! Role: This module manages worker threads and blocking jobs.

use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Bound DLL-unload cleanup without waiting forever for uncooperative jobs.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);

pub(crate) type Job = Box<dyn FnOnce() + Send>;

pub(crate) fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Fixed pool of worker threads (default `min(4, CPUs)`).
pub(crate) struct Pool {
    handles: Mutex<Vec<JoinHandle<()>>>,
    tx: Mutex<Option<mpsc::Sender<Job>>>,
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
        let tx = if !handles.is_empty() { Some(tx) } else { None };
        Arc::new(Pool {
            handles: Mutex::new(handles),
            tx: Mutex::new(tx),
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
        let tx = lock(&self.tx).clone();
        let sent = match tx {
            Some(tx) => tx.send(job).map_err(|e| e.0),
            None => Err(job),
        };
        if let Err(job) = sent {
            // No worker available: retain the fallback for App shutdown.
            if let Ok(handle) = std::thread::Builder::new()
                .name("kivact-blocking-fallback".into())
                .spawn(job)
            {
                let mut handles = lock(&self.handles);
                // Finished fallbacks need no join; keep the list bounded.
                handles.retain(|h| !h.is_finished());
                handles.push(handle);
            }
        }
    }

    /// Stop accepting jobs; idle workers exit.
    pub(crate) fn shutdown(&self) {
        lock(&self.tx).take();
        wait_for_threads(std::mem::take(&mut *lock(&self.handles)));
    }
}

/// Join completed jobs and warn before detaching those beyond their deadline.
pub(crate) fn wait_for_threads(handles: Vec<JoinHandle<()>>) {
    let mut detached = 0;
    for handle in handles {
        let start = Instant::now();
        while !handle.is_finished() && start.elapsed() < SHUTDOWN_WAIT {
            std::thread::sleep(Duration::from_millis(1));
        }
        if handle.is_finished() {
            let _ = handle.join();
        } else {
            crate::detached::DETACHED.store(true, std::sync::atomic::Ordering::Release);
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
