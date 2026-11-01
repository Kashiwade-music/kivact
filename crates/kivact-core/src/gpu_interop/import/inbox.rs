//! Role: State an engine's threads share with a UI task, and the wakes between them.
//!
//! The one way the engine's threads reach the UI thread in
//! `kivact::gpu_interop` (`FramePublisher`, `ShareLink`): they change the
//! state and wake the task the UI side awaits it in, through the waker of
//! the UI thread's executor (the host's `HostWaker`), as a worker's
//! channel wakes a receiving task (the canvas guide's worker). An engine
//! thread waits for the UI's changes on a condition variable with a time
//! limit.
//!
//! Invariants:
//! - The task's waker is kept under the state's lock, so a change made
//!   after the task found nothing to do always wakes it.
//! - Every change wakes both sides; a side that finds nothing for it waits
//!   again. Wakers are woken after the lock is released.

use std::future::Future;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::task::{Poll, Waker};
use std::time::{Duration, Instant};

/// The state `S` and the waker of the task awaiting it.
struct Locked<S> {
    state: S,
    waker: Option<Waker>,
}

/// State shared between an engine's threads and a task of the UI thread.
pub(crate) struct Inbox<S> {
    locked: Mutex<Locked<S>>,
    changed: Condvar,
}

impl<S> Inbox<S> {
    pub(crate) fn new(state: S) -> Self {
        Inbox {
            locked: Mutex::new(Locked { state, waker: None }),
            changed: Condvar::new(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Locked<S>> {
        self.locked.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Change the state with `f`, from any thread, and wake the UI task
    /// and the waiting engine threads.
    pub(crate) fn update<R>(&self, f: impl FnOnce(&mut S) -> R) -> R {
        let (out, waker) = {
            let mut locked = self.lock();
            let out = f(&mut locked.state);
            (out, locked.waker.take())
        };
        self.changed.notify_all();
        if let Some(waker) = waker {
            waker.wake();
        }
        out
    }

    /// Read the state with `f`, from any thread.
    pub(crate) fn read<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        f(&self.lock().state)
    }

    /// Block the calling (engine) thread until `f` answers `Some`, for at
    /// most `timeout`; `None` when the time ran out.
    pub(crate) fn wait<R>(
        &self,
        timeout: Duration,
        mut f: impl FnMut(&mut S) -> Option<R>,
    ) -> Option<R> {
        let deadline = Instant::now().checked_add(timeout);
        let mut locked = self.lock();
        loop {
            if let Some(out) = f(&mut locked.state) {
                return Some(out);
            }
            let left = match deadline {
                Some(deadline) => deadline.checked_duration_since(Instant::now())?,
                // A timeout past what an `Instant` can hold: wait in long steps.
                None => Duration::from_secs(3600),
            };
            locked = self
                .changed
                .wait_timeout(locked, left)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }

    /// Await, in a task of the UI thread, the first time `f` answers
    /// `Some`.
    pub(crate) fn next<'a, R>(
        &'a self,
        mut f: impl FnMut(&mut S) -> Option<R> + 'a,
    ) -> impl Future<Output = R> + 'a {
        std::future::poll_fn(move |cx| {
            let mut locked = self.lock();
            match f(&mut locked.state) {
                Some(out) => Poll::Ready(out),
                None => {
                    locked.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        })
    }
}
