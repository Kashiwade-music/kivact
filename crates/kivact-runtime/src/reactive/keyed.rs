//! Role: Keyed rows that keep their owner while their key stays (`mapArray`).
//!
//! A keyed collection (a list, the regions of a canvas) keeps one [`Row`]
//! per key: a child owner and the signal holding the row's item. [`diff`]
//! compares new items with the current [`Rows`] and says which rows keep
//! their owner (and whether their item changed), which are new and which
//! keys repeat; the caller mounts, moves and disposes accordingly.
//!
//! Invariants:
//! - The items and keys are compared without tracking and without cloning
//!   unchanged items; only changed and new items are cloned.
//! - Same keys in the same order is the fast path ([`Diff::Values`]): no
//!   key is hashed.
//! - A row's setup ([`Row::setup`]) runs in a setup frame of the row's own
//!   owner; a panic there propagates.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::panic::Location;
use std::rc::Rc;

use crate::reactive::data::SourceCell;
use crate::reactive::registry::runtime;
use crate::reactive::{Owner, Signal, Value};

/// One row of a keyed collection: its key, its own owner (a child of the
/// collection's owner) and the signal holding its item.
pub struct Row<K, T: 'static> {
    key: K,
    owner: Owner,
    item: Signal<T>,
    cell: Rc<SourceCell<T>>,
}

/// The rows of a keyed collection in order, with an index by key.
pub struct Rows<K, R> {
    rows: Vec<R>,
    index: HashMap<K, usize>,
}

/// What [`diff`] found.
pub enum Diff<K, T> {
    /// Same keys in the same order: only the items of these rows (by
    /// position) changed.
    Values(Vec<(usize, T)>),
    /// The keys changed: the new order of keys, the plan of each key, and
    /// the hashes of keys that repeat (their later items are dropped).
    Structure {
        /// The keys in their new order.
        keys: Vec<K>,
        /// What each key in `keys` becomes.
        plans: Vec<Plan<T>>,
        /// [`key_hash`] of each repeated key.
        dups: Vec<u64>,
    },
}

/// What one key of a [`Diff::Structure`] becomes.
pub enum Plan<T> {
    /// The row at `old` keeps its owner; `changed` is its new item if it
    /// differs (`PartialEq`).
    Keep {
        /// Position of the row in the current rows.
        old: usize,
        /// The new item, when it differs from the current one.
        changed: Option<T>,
    },
    /// A new row for this item.
    New(T),
}

impl<K, T: 'static> Row<K, T> {
    /// A row for `item` under a new child owner of `parent`, created at
    /// `at`. `None` when `parent` is already gone (the child is created
    /// disposed and keeps no item).
    pub fn new(parent: Owner, key: K, item: T, at: &'static Location<'static>) -> Option<Self> {
        let owner = parent.child(at);
        let item = owner.signal(item, at);
        let cell = runtime(owner.app, at).source_rc::<T>(item.key)?;
        Some(Self {
            key,
            owner,
            item,
            cell,
        })
    }

    /// The row's key.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// The row's own owner: disposing it disposes the row.
    pub fn owner(&self) -> Owner {
        self.owner
    }

    /// The signal holding the row's item.
    pub fn item(&self) -> Signal<T> {
        self.item
    }

    /// Run `make` with the row's item as setup of the row's owner (untracked
    /// as every setup). A panic in it propagates.
    pub fn setup<R>(&self, at: &'static Location<'static>, make: impl FnOnce(Value<T>) -> R) -> R {
        let rt = runtime(self.owner.app, at);
        let _setup = rt.setup_frame(self.owner.key);
        make(Value::from(self.item))
    }

    fn differs(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.cell
            .value
            .try_borrow()
            .map(|cur| *cur != *item)
            .unwrap_or(true)
    }
}

impl<K, T: 'static> AsRef<Row<K, T>> for Row<K, T> {
    fn as_ref(&self) -> &Row<K, T> {
        self
    }
}

impl<K, R> Rows<K, R> {
    /// No rows.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// The rows in order.
    pub fn rows(&self) -> &[R] {
        &self.rows
    }

    /// Take the rows out (to rebuild them with [`Rows::replace`]).
    pub fn take(&mut self) -> Vec<R> {
        std::mem::take(&mut self.rows)
    }

    /// Make `rows` the current rows and index them by key.
    pub fn replace<T: 'static>(&mut self, rows: Vec<R>)
    where
        K: Eq + Hash + Clone,
        R: AsRef<Row<K, T>>,
    {
        self.index.clear();
        for (i, row) in rows.iter().enumerate() {
            self.index.insert(row.as_ref().key.clone(), i);
        }
        self.rows = rows;
    }
}

impl<K, R> Default for Rows<K, R> {
    fn default() -> Self {
        Self::new()
    }
}

/// The hash a [`Diff::Structure`] names a repeated key by (also the
/// `key_hash` of `ErrorKind::DuplicateKey`).
pub fn key_hash<K: Hash>(key: &K) -> u64 {
    let mut h = DefaultHasher::new();
    key.hash(&mut h);
    h.finish()
}

/// Compare `list` (keyed by `key`) with the current `rows`.
pub fn diff<K, T, R>(rows: &Rows<K, R>, list: &[T], key: &dyn Fn(&T) -> K) -> Diff<K, T>
where
    K: Eq + Hash + Clone,
    T: crate::PropValue,
    R: AsRef<Row<K, T>>,
{
    let current = &rows.rows;
    // Fast path: same keys in the same order (no hashing).
    let mut changed = Vec::new();
    let mut i = 0;
    let mut first_key = None;
    while i < list.len() && i < current.len() {
        let k = key(&list[i]);
        let row = current[i].as_ref();
        if k != row.key {
            first_key = Some(k);
            break;
        }
        if row.differs(&list[i]) {
            changed.push((i, list[i].clone()));
        }
        i += 1;
    }
    if first_key.is_none() && i == list.len() && i == current.len() {
        return Diff::Values(changed);
    }
    // Structural change: the matching prefix is kept in place, the rest is
    // matched by key.
    let mut keys: Vec<K> = current[..i]
        .iter()
        .map(|r| r.as_ref().key.clone())
        .collect();
    let mut plans: Vec<Plan<T>> = (0..i)
        .map(|o| Plan::Keep {
            old: o,
            changed: None,
        })
        .collect();
    for (o, v) in changed {
        plans[o] = Plan::Keep {
            old: o,
            changed: Some(v),
        };
    }
    let mut seen: HashSet<K> = keys.iter().cloned().collect();
    let mut dups = Vec::new();
    for (n, it) in list.iter().enumerate().skip(i) {
        let k = match first_key.take() {
            Some(k) if n == i => k,
            _ => key(it),
        };
        if !seen.insert(k.clone()) {
            dups.push(key_hash(&k));
            continue;
        }
        match rows.index.get(&k) {
            Some(&old) => {
                let changed = current[old].as_ref().differs(it).then(|| it.clone());
                plans.push(Plan::Keep { old, changed });
            }
            None => plans.push(Plan::New(it.clone())),
        }
        keys.push(k);
    }
    Diff::Structure { keys, plans, dups }
}
