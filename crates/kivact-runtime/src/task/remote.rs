//! Role: Values and notifications sent to the UI from other threads.
//!
//! `remote(initial)` returns a `Signal` read on the UI thread and a
//! `Remote` any thread may `set`. The mailbox keeps the latest value only.
//! A set marks the App's inbox and wakes its event loop through the App's
//! `ThreadWaker` (the path a finished `blocking` job takes). The runtime
//! driver delivers every mailbox once at the start of a turn
//! (`Runtime::begin_turn`), so the signal is written at most once per
//! turn. The owner's cleanup closes the mailbox; a later set drops its
//! value. No user code runs on another thread or under the lock.

use std::panic::Location;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::{Owner, Runtime, Signal, ThreadWaker, Trigger};

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What an App's remotes share with other threads: whether a mailbox has
/// mail, and the waker of the App's event loop.
#[derive(Clone)]
pub(crate) struct RemoteWake {
    pending: Arc<AtomicBool>,
    waker: ThreadWaker,
}

impl RemoteWake {
    pub(crate) fn new(waker: ThreadWaker) -> Self {
        RemoteWake {
            pending: Arc::new(AtomicBool::new(false)),
            waker,
        }
    }

    /// A mailbox got mail: wake the loop once until the next delivery.
    fn post(&self) {
        if !self.pending.swap(true, Ordering::AcqRel) {
            self.waker.wake();
        }
    }

    /// Whether mail waits (`Runtime::has_mail`).
    pub(crate) fn is_pending(&self) -> bool {
        self.pending.load(Ordering::Acquire)
    }

    /// Take the mark before a delivery; mail that arrives later marks it again.
    pub(crate) fn take(&self) -> bool {
        self.pending.swap(false, Ordering::AcqRel)
    }
}

/// The UI side of one mailbox, registered with the App's task state.
pub(crate) trait Deliver {
    /// Write what arrived since the last delivery (at most one write).
    fn deliver(&self);
}

/// A value mailbox: open with the latest value not yet delivered, or
/// closed with its owner.
enum Mailbox<T> {
    Open { latest: Option<T> },
    Closed,
}

/// The writing half of [`remote`]: `Clone + Send + Sync`, usable from any
/// thread. `set` never waits for the UI thread and runs no user code.
pub struct Remote<T> {
    mail: Arc<Mutex<Mailbox<T>>>,
    wake: RemoteWake,
}

impl<T> Clone for Remote<T> {
    fn clone(&self) -> Self {
        Remote {
            mail: Arc::clone(&self.mail),
            wake: self.wake.clone(),
        }
    }
}

impl<T: Send + 'static> Remote<T> {
    /// Replace the value; the UI shows the latest one at its next turn.
    /// After the signal's component is removed, the value is dropped.
    pub fn set(&self, value: T) {
        let (open, replaced) = {
            let mut mail = lock(&self.mail);
            match &mut *mail {
                Mailbox::Open { latest } => (true, latest.replace(value)),
                Mailbox::Closed => (false, Some(value)),
            }
        };
        // Dropped outside the lock.
        drop(replaced);
        if open {
            self.wake.post();
        }
    }
}

// xtask-allow(state): independent: mail is the mailbox other threads fill through Remote::set and the owner's cleanup closes; signal is the UI value that only deliver writes, once per turn with the latest mail
struct RemoteInbox<T: 'static> {
    mail: Arc<Mutex<Mailbox<T>>>,
    signal: Signal<T>,
}

impl<T: Send + 'static> Deliver for RemoteInbox<T> {
    fn deliver(&self) {
        let value = match &mut *lock(&self.mail) {
            Mailbox::Open { latest } => latest.take(),
            Mailbox::Closed => None,
        };
        if let Some(value) = value {
            self.signal.update(move |slot| *slot = value);
        }
    }
}

/// Register `inbox` with `owner`'s App; the owner's cleanup runs `close`
/// (outside every lock) and forgets the inbox.
fn register(owner: Owner, inbox: Rc<dyn Deliver>, close: impl FnOnce() + 'static) {
    let rt = owner.runtime();
    rt.tasks().remotes.borrow_mut().push(Rc::clone(&inbox));
    let app = rt.id();
    owner.on_cleanup(move || {
        close();
        if let Some(rt) = Runtime::get(app) {
            rt.tasks()
                .remotes
                .borrow_mut()
                .retain(|r| !Rc::ptr_eq(r, &inbox));
        }
    });
}

/// A signal other threads write through the returned [`Remote`] (setup
/// only). Writes from other threads wake the event loop and are merged:
/// each turn writes the signal at most once, with the latest value.
/// Removing the component closes it; later values are dropped.
///
/// ```ignore
/// let (progress, sender) = remote(0.0);
/// std::thread::spawn(move || for p in 0..=100 { sender.set(f64::from(p) / 100.0) });
/// ```
#[track_caller]
pub fn remote<T: Send + 'static>(initial: T) -> (Signal<T>, Remote<T>) {
    let at = Location::caller();
    let owner = Owner::current("remote", at);
    let wake = owner.runtime().tasks().remote_wake.clone();
    let signal = owner.signal(initial, at);
    let mail = Arc::new(Mutex::new(Mailbox::Open { latest: None }));
    let closing = Arc::clone(&mail);
    register(
        owner,
        Rc::new(RemoteInbox {
            mail: Arc::clone(&mail),
            signal,
        }),
        move || {
            // The value left is dropped after the lock is released.
            let left = std::mem::replace(&mut *lock(&closing), Mailbox::Closed);
            drop(left);
        },
    );
    (signal, Remote { mail, wake })
}

/// A trigger's mailbox: 0 open, 1 open and notified, 2 closed.
struct TriggerMail {
    state: AtomicU8,
}

const OPEN: u8 = 0;
const NOTIFIED: u8 = 1;
const CLOSED: u8 = 2;

/// The notifying half of [`remote_trigger`]: `Clone + Send + Sync`.
#[derive(Clone)]
pub struct RemoteTrigger {
    mail: Arc<TriggerMail>,
    wake: RemoteWake,
}

impl RemoteTrigger {
    /// Notify the trigger's readers at the UI's next turn. Never blocks;
    /// notifications before that turn are one; after the component is
    /// removed it does nothing.
    pub fn notify(&self) {
        if self
            .mail
            .state
            .compare_exchange(OPEN, NOTIFIED, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.wake.post();
        }
    }
}

struct TriggerInbox {
    mail: Arc<TriggerMail>,
    trigger: Trigger,
}

impl Deliver for TriggerInbox {
    fn deliver(&self) {
        if self
            .mail
            .state
            .compare_exchange(NOTIFIED, OPEN, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.trigger.notify();
        }
    }
}

/// A [`Trigger`] other threads notify through the returned
/// [`RemoteTrigger`] (setup only), for state the App keeps outside signals
/// (a buffer a worker fills). It closes with its component, as `remote`.
#[track_caller]
pub fn remote_trigger() -> (Trigger, RemoteTrigger) {
    let at = Location::caller();
    let owner = Owner::current("remote_trigger", at);
    let wake = owner.runtime().tasks().remote_wake.clone();
    let trigger = owner.trigger(at);
    let mail = Arc::new(TriggerMail {
        state: AtomicU8::new(OPEN),
    });
    let closing = Arc::clone(&mail);
    register(
        owner,
        Rc::new(TriggerInbox {
            mail: Arc::clone(&mail),
            trigger,
        }),
        move || closing.state.store(CLOSED, Ordering::Release),
    );
    (trigger, RemoteTrigger { mail, wake })
}
