//! Role: The cross-thread handle that ends an App's run.
//!
//! One fact lives here: whether the run this handle drives was asked to end
//! from outside (`AppHandle::close`). The runtime's `ExitState` owns the
//! drain that follows. Every transition runs under the link's mutex; the
//! waker runs after the lock is released, so no host or user code runs while
//! it is held.

use std::fmt;
use std::panic::Location;
use std::sync::{Arc, Mutex, MutexGuard};

use kivact_runtime::{Owner, Runtime, ThreadWaker};

/// Ends a running App from any thread, like the user closing its main window.
///
/// `Send + Sync + Clone`; clones share the request. Get it before the run
/// with [`App::handle`](crate::App::handle), or inside the App with
/// [`app_handle`]. [`close`](AppHandle::close) returns at once: the App's
/// next turn begins the exit, queued actions drain (at most 3 s, as for a
/// close request), owners are disposed and
/// [`App::run`](crate::App::run) returns `Ok(())`.
///
/// Closing is not a dismissal request: no `on_dismiss` runs, and it cannot be
/// cancelled. A handle belongs to one run, and a `close` after that run ended
/// does nothing. An embedded App ([`App::embed`](crate::App::embed)) is
/// closed by dropping its `Embedded` handle; there `close` reports
/// `ErrorKind::Unsupported` to the root owner.
#[derive(Clone)]
pub struct AppHandle {
    link: Arc<AppLink>,
}

impl AppHandle {
    /// A handle with no run attached; `App::new` makes one.
    pub(crate) fn new() -> Self {
        Self {
            link: Arc::new(AppLink {
                state: Mutex::new(LinkState::Unstarted),
            }),
        }
    }

    /// Ask the App to end. Callable from any thread, any number of times.
    ///
    /// Before the run starts, the run returns `Ok(())` without opening its
    /// window; after the run ended it does nothing.
    pub fn close(&self) {
        if let Some(waker) = self.link.close() {
            // The waker reaches the host's event loop. Never call it with the
            // link locked: the UI thread takes the same lock at a turn start.
            waker.wake();
        }
    }

    /// What the run that is starting must do (see [`Begin`]).
    #[track_caller]
    pub(crate) fn begin(&self) -> Begin {
        self.link.begin()
    }

    /// Bind the run's waker (the UI thread, during `Launch::start`).
    pub(crate) fn attach(&self, waker: ThreadWaker) {
        if let Some(waker) = self.link.attach(waker) {
            waker.wake();
        }
    }

    /// Take a pending request; `true` only the first time after a `close`
    /// since the last taken one.
    pub(crate) fn take_request(&self, accept: bool) -> bool {
        self.link.take_request(accept)
    }

    /// The run ended (or never started): later `close` calls do nothing.
    pub(crate) fn end(&self) {
        self.link.end();
    }
}

impl fmt::Debug for AppHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AppHandle({})", self.link.name())
    }
}

/// What the run that is starting must do with this handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Begin {
    /// No run attached and none asked to end: start the run.
    Run,
    /// `close` came before the run: return without opening a window.
    Skip,
}

/// The handle of the App this code belongs to (the one `App::handle` returned).
///
/// Call it in setup or in a handler, a task or an effect; it panics outside an
/// owner (a misuse, with a fix hint).
#[track_caller]
pub fn app_handle() -> AppHandle {
    let owner = Owner::ambient("app_handle", Location::caller());
    AppHandleExt::of(&owner.runtime()).0.clone()
}

/// The App's handle, installed as a runtime extension so `app_handle` returns
/// the handle of the App whose code is running.
pub(crate) struct AppHandleExt(pub(crate) AppHandle);

impl AppHandleExt {
    /// The handle of `rt`'s App.
    pub(crate) fn of(rt: &Runtime) -> &Self {
        rt.ext::<Self>()
    }
}

/// The single owner of "this run was asked to end from outside".
struct AppLink {
    /// The state. Lock it only to read or transition; the wake runs after the
    /// lock is released.
    state: Mutex<LinkState>,
}

/// Where the run this handle drives is, as seen from any thread.
///
/// `Unstarted` and `Ended` differ only in `close`; `Ended` is terminal, so a
/// late `close` (a session thread racing the user's own window close) never
/// affects a later run. There is no later run: a handle belongs to one run.
#[derive(Clone)]
enum LinkState {
    /// No run attached yet; nothing asked.
    Unstarted,
    /// `close` came before a run attached its waker: the next `begin` skips
    /// the run, or the next `attach` turns it into `CloseRequested`.
    ClosePending,
    /// A run is attached; its waker starts a turn from any thread.
    Running(ThreadWaker),
    /// `close` came during the run; the UI thread has not taken it yet.
    CloseRequested(ThreadWaker),
    /// The UI thread took the request; the App drains and exits.
    Exiting,
    /// The run returned (or never started): `close` does nothing.
    Ended,
}

impl AppLink {
    fn lock(&self) -> MutexGuard<'_, LinkState> {
        self.state
            .lock()
            .expect("invariant: the app link lock is not poisoned")
    }

    /// Record a close request; returns the waker to call after unlocking.
    fn close(&self) -> Option<ThreadWaker> {
        let mut state = self.lock();
        match state.clone() {
            LinkState::Unstarted => {
                *state = LinkState::ClosePending;
                None
            }
            LinkState::Running(waker) => {
                *state = LinkState::CloseRequested(waker.clone());
                Some(waker)
            }
            LinkState::ClosePending
            | LinkState::CloseRequested(_)
            | LinkState::Exiting
            | LinkState::Ended => None,
        }
    }

    #[track_caller]
    fn begin(&self) -> Begin {
        let mut state = self.lock();
        let outcome = match state.clone() {
            LinkState::Unstarted => Ok(Begin::Run),
            LinkState::ClosePending => {
                *state = LinkState::Ended;
                Ok(Begin::Skip)
            }
            LinkState::Ended => Err(
                "one AppHandle drives one run; App::run consumes the App and cannot run again: \
                 make another App with App::new",
            ),
            LinkState::Running(_) | LinkState::CloseRequested(_) | LinkState::Exiting => {
                Err("an AppHandle drives one run at a time; make another App with App::new")
            }
        };
        // Do not poison the link with a misuse panic: another clone may still
        // ask about the run that is going.
        drop(state);
        match outcome {
            Ok(begin) => begin,
            Err(why) => panic!("kivact misuse: {why}"),
        }
    }

    /// Bind the waker of the run that is starting; returns a waker to call
    /// after unlocking when a request is already pending.
    fn attach(&self, waker: ThreadWaker) -> Option<ThreadWaker> {
        let mut state = self.lock();
        match state.clone() {
            LinkState::Unstarted => {
                *state = LinkState::Running(waker);
                None
            }
            LinkState::ClosePending => {
                *state = LinkState::CloseRequested(waker.clone());
                Some(waker)
            }
            LinkState::Running(_) | LinkState::CloseRequested(_) | LinkState::Exiting => {
                // `begin` refuses a second run, so a host start is the only attach.
                drop(state);
                unreachable!("invariant: one run attaches to an AppHandle once")
            }
            // `begin` turns `Ended` into a skipping run or a misuse panic, so
            // a started run never attaches to an ended handle.
            LinkState::Ended => {
                drop(state);
                unreachable!("invariant: an ended AppHandle starts no run")
            }
        }
    }

    fn take_request(&self, accept: bool) -> bool {
        let mut state = self.lock();
        match state.clone() {
            LinkState::CloseRequested(waker) => {
                *state = if accept {
                    LinkState::Exiting
                } else {
                    LinkState::Running(waker)
                };
                true
            }
            LinkState::Unstarted
            | LinkState::ClosePending
            | LinkState::Running(_)
            | LinkState::Exiting
            | LinkState::Ended => false,
        }
    }

    fn end(&self) {
        *self.lock() = LinkState::Ended;
    }

    fn name(&self) -> &'static str {
        match &*self.lock() {
            LinkState::Unstarted => "Unstarted",
            LinkState::ClosePending => "ClosePending",
            LinkState::Running(_) => "Running",
            LinkState::CloseRequested(_) => "CloseRequested",
            LinkState::Exiting => "Exiting",
            LinkState::Ended => "Ended",
        }
    }
}

#[cfg(test)]
mod link_tests {
    //! Role: Tests for the AppHandle link state machine.

    use super::{AppHandle, Begin};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use kivact_runtime::ThreadWaker;

    fn counted_waker() -> (ThreadWaker, Arc<AtomicUsize>) {
        let count = Arc::new(AtomicUsize::new(0));
        let woken = count.clone();
        let waker = ThreadWaker::new(move || {
            woken.fetch_add(1, Ordering::SeqCst);
        });
        (waker, count)
    }

    #[test]
    fn close_before_the_run_skips_it_and_ends_the_handle() {
        let handle = AppHandle::new();
        handle.close();
        assert_eq!(handle.begin(), Begin::Skip, "a pending close skips the run");
        assert_eq!(format!("{handle:?}"), "AppHandle(Ended)");
        handle.close();
        assert_eq!(
            format!("{handle:?}"),
            "AppHandle(Ended)",
            "close after the run keeps `Ended`"
        );
    }

    #[test]
    fn close_during_the_run_wakes_once() {
        let handle = AppHandle::new();
        assert_eq!(handle.begin(), Begin::Run);
        let (waker, count) = counted_waker();
        handle.attach(waker);
        handle.close();
        assert_eq!(count.load(Ordering::SeqCst), 1, "one wake reaches the host");
        handle.close();
        assert_eq!(
            count.load(Ordering::SeqCst),
            1,
            "a second close takes the same request"
        );
        assert_eq!(format!("{handle:?}"), "AppHandle(CloseRequested)");
        assert!(handle.take_request(true));
        assert!(!handle.take_request(true), "a request is taken once");
        assert_eq!(format!("{handle:?}"), "AppHandle(Exiting)");
        handle.close();
        assert_eq!(
            count.load(Ordering::SeqCst),
            1,
            "an exiting run is not woken"
        );
        handle.end();
        assert_eq!(format!("{handle:?}"), "AppHandle(Ended)");
        handle.close();
        assert_eq!(
            count.load(Ordering::SeqCst),
            1,
            "close after the run does nothing"
        );
    }

    #[test]
    fn close_before_attach_becomes_a_request_that_wakes() {
        let handle = AppHandle::new();
        assert_eq!(handle.begin(), Begin::Run);
        handle.close();
        assert_eq!(format!("{handle:?}"), "AppHandle(ClosePending)");
        let (waker, count) = counted_waker();
        handle.attach(waker);
        assert_eq!(count.load(Ordering::SeqCst), 1, "attach starts the exit");
        assert_eq!(format!("{handle:?}"), "AppHandle(CloseRequested)");
        assert!(handle.take_request(true));
    }

    #[test]
    fn a_refused_request_keeps_the_run_attached() {
        let handle = AppHandle::new();
        assert_eq!(handle.begin(), Begin::Run);
        let (waker, count) = counted_waker();
        handle.attach(waker);
        handle.close();
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(handle.take_request(false));
        assert_eq!(format!("{handle:?}"), "AppHandle(Running)");
        handle.close();
        assert_eq!(
            count.load(Ordering::SeqCst),
            2,
            "an embedded App can be asked again"
        );
        assert!(handle.take_request(true));
    }

    #[test]
    fn debug_names_the_state() {
        let handle = AppHandle::new();
        assert_eq!(format!("{handle:?}"), "AppHandle(Unstarted)");
    }

    #[test]
    fn the_handle_is_send_sync_clone_and_shared_between_clones() {
        fn assert_send_sync_clone<T: Send + Sync + Clone>(handle: &T) -> T {
            handle.clone()
        }
        let handle = AppHandle::new();
        let clone = assert_send_sync_clone(&handle);
        clone.close();
        assert_eq!(format!("{handle:?}"), "AppHandle(ClosePending)");
        assert_eq!(handle.begin(), Begin::Skip);
    }

    #[test]
    #[should_panic(expected = "one AppHandle drives one run")]
    fn a_finished_handle_starts_no_run() {
        let handle = AppHandle::new();
        handle.close();
        assert_eq!(handle.begin(), Begin::Skip);
        handle.begin();
    }

    #[test]
    #[should_panic(expected = "drives one run at a time")]
    fn a_running_handle_starts_no_second_run() {
        let handle = AppHandle::new();
        assert_eq!(handle.begin(), Begin::Run);
        let (waker, _) = counted_waker();
        handle.attach(waker);
        handle.begin();
    }
}
