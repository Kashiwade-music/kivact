//! Role: Share diagnostic latches, caches, generations and mailboxes.
//!
//! Fields stay private so their one update rule is enforced by these methods.
//! These helpers store a latch, cache, identity or take-once value; they are not
//! substitutes for interaction state machines or event notification counters.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::{Rc, Weak};
use std::sync::Mutex;

#[derive(Clone, Copy)]
enum Reported {
    Not,
    Done,
}

/// One-shot diagnostic latch, the named reporting exception to S-9.
pub struct ReportOnce(Cell<Reported>);

impl ReportOnce {
    /// Start with no diagnostic reported.
    pub const fn new() -> Self {
        Self(Cell::new(Reported::Not))
    }

    /// Mark the diagnostic as reported; return true only on the first call.
    pub fn note(&self) -> bool {
        matches!(self.0.replace(Reported::Done), Reported::Not)
    }
}

impl Default for ReportOnce {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostic deduplication cache keyed by the fact being reported (S-9).
pub struct ReportOnceSet<K>(RefCell<HashSet<K>>);

impl<K> ReportOnceSet<K> {
    /// Start with no keys reported.
    pub fn new() -> Self {
        Self(RefCell::new(HashSet::new()))
    }

    /// Forget every reported key that `keep` rejects. Keys it accepts stay
    /// reported. `keep` must not touch this set.
    pub fn retain(&self, keep: impl FnMut(&K) -> bool) {
        self.0.borrow_mut().retain(keep);
    }
}

impl<K: Eq + Hash> ReportOnceSet<K> {
    /// Mark a key as reported; return true only for its first report.
    pub fn note(&self, key: K) -> bool {
        self.0.borrow_mut().insert(key)
    }
}

impl<K> Default for ReportOnceSet<K> {
    fn default() -> Self {
        Self::new()
    }
}

/// Last value handed to an outside party, the deduplication cache of S-9.
#[derive(Clone, Debug)]
pub struct Sent<T>(Option<T>);

impl<T> Sent<T> {
    /// The last value handed to the outside party.
    pub fn get(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// Start before any value has been sent.
    pub const fn new() -> Self {
        Self(None)
    }
}

impl<T: PartialEq> Sent<T> {
    /// Return the new value to send, or None when it equals the last value.
    ///
    /// The first value always changes the cache, including None when T itself
    /// is an Option. Equal values leave the previously sent value in place.
    pub fn update(&mut self, value: T) -> Option<&T> {
        if self.0.as_ref() == Some(&value) {
            return None;
        }
        self.0 = Some(value);
        self.0.as_ref()
    }
}

impl<T> Default for Sent<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Monotonic identity for rejecting stale results, the generation exception to S-9.
///
/// Copy a generation when starting work and compare it with is_current when
/// work completes. Advancing is identity allocation, not event notification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Generation(u64);

impl Generation {
    /// The initial generation, before any advance.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Advance and return the new generation snapshot.
    ///
    /// Panics if all u64 generation identities have been exhausted; identities
    /// must never wrap and make an old result current again.
    #[expect(
        clippy::should_implement_trait,
        reason = "generation allocation returns a token, not an optional iterator item"
    )]
    pub fn next(&mut self) -> Self {
        self.0 = self
            .0
            .checked_add(1)
            .expect("invariant: generation counter exhausted");
        *self
    }

    /// The numeric identity of this generation, for APIs that use a `u64` id.
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Whether a saved generation still identifies the current work.
    pub fn is_current(&self, generation: Self) -> bool {
        *self == generation
    }
}

/// Latest-wins, take-once cross-thread mailbox, the handoff exception to S-9.
///
/// Poisoning is an invariant failure. A mailbox never recovers or exposes the
/// contents of a poisoned lock.
pub struct Mailbox<T>(Mutex<Option<T>>);

impl<T> Mailbox<T> {
    /// Start empty.
    pub const fn new() -> Self {
        Self(Mutex::new(None))
    }

    /// Replace any pending value. The displaced value is dropped after unlocking.
    pub fn put(&self, value: T) {
        let previous = self
            .0
            .lock()
            .expect("invariant: mailbox lock is not poisoned")
            .replace(value);
        drop(previous);
    }

    /// Remove the pending value; it can be returned only once.
    pub fn take(&self) -> Option<T> {
        self.0
            .lock()
            .expect("invariant: mailbox lock is not poisoned")
            .take()
    }
}

impl<T> Default for Mailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Latest-wins, take-once UI-thread mailbox, the local handoff exception to S-9.
pub struct LocalMailbox<T>(Cell<Option<T>>);

impl<T> LocalMailbox<T> {
    /// Start empty.
    pub const fn new() -> Self {
        Self(Cell::new(None))
    }

    /// Replace and drop any pending value without requiring T to be Copy.
    pub fn put(&self, value: T) {
        self.0.set(Some(value));
    }

    /// Remove the pending value; it can be returned only once.
    pub fn take(&self) -> Option<T> {
        self.0.take()
    }
}

impl<T> Default for LocalMailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Weak UI-thread registrations with one prune rule, the pure-cache exception to S-9.
///
/// An entry lives exactly while its Rc can be upgraded. Registry operations
/// never prolong ownership beyond the live snapshot they return.
pub struct Registry<T: ?Sized> {
    entries: Vec<Weak<T>>,
    /// The length at which `insert` next prunes dead entries: twice the live
    /// count after the last pruning, so inserting stays amortized O(1).
    prune_at: usize,
}

impl<T: ?Sized> Registry<T> {
    /// Start without registrations.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            prune_at: 0,
        }
    }

    /// Register a weak reference, pruning registrations whose owners are
    /// gone once the list has doubled since the last pruning.
    pub fn insert(&mut self, entry: &Rc<T>) {
        if self.entries.len() >= self.prune_at {
            self.entries.retain(|entry| entry.strong_count() > 0);
            self.prune_at = (self.entries.len() * 2).max(8);
        }
        self.entries.push(Rc::downgrade(entry));
    }

    /// Snapshot live registrations in insertion order and remove dead entries.
    pub fn live(&mut self) -> Vec<Rc<T>> {
        let mut live = Vec::with_capacity(self.entries.len());
        self.entries.retain(|entry| {
            if let Some(entry) = entry.upgrade() {
                live.push(entry);
                true
            } else {
                false
            }
        });
        live
    }
}

impl<T: ?Sized> Default for Registry<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// The last few distinct values of one kind, shared behind `Rc` between the
/// owners that store an equal value (repeated components): a value equal to a
/// recent one is that one's `Rc`, any other value replaces the oldest.
/// Compared by value; an `Rc` handed out is never changed in place (a writer
/// that changes it clones it, `Rc::make_mut`).
pub struct Recent<T> {
    slots: [Option<Rc<T>>; RECENT],
    /// The slot the next new value takes (the oldest).
    next: usize,
    /// The slot of the last hit or insert, compared first.
    last: usize,
}

const RECENT: usize = 8;

impl<T> Default for Recent<T> {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| None),
            next: 0,
            last: 0,
        }
    }
}

// The derives would require `T: Clone` / `T: Debug`; the slots are `Rc`s.
impl<T> Clone for Recent<T> {
    fn clone(&self) -> Self {
        Self {
            slots: self.slots.clone(),
            next: self.next,
            last: self.last,
        }
    }
}

impl<T> std::fmt::Debug for Recent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kept = self.slots.iter().flatten().count();
        f.debug_struct("Recent").field("kept", &kept).finish()
    }
}

impl<T: PartialEq> Recent<T> {
    /// `value` behind an `Rc`: an equal recent value's, else a new one.
    pub fn share(&mut self, value: T) -> Rc<T> {
        for step in 0..RECENT {
            let at = (self.last + RECENT - step) % RECENT;
            if let Some(found) = &self.slots[at]
                && **found == value
            {
                self.last = at;
                return Rc::clone(found);
            }
        }
        let rc = Rc::new(value);
        self.slots[self.next] = Some(Rc::clone(&rc));
        self.last = self.next;
        self.next = (self.next + 1) % RECENT;
        rc
    }
}
