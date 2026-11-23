//! Role: One collection's keys in order: built once per change, read per key in O(1).

use crate::{ListBoundary, ListItem};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// A move in logical order. Controllers map physical keys to this policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Movement {
    Next,
    Previous,
    First,
    Last,
}

/// The first occurrence of a key in the source of an [`Index`].
#[derive(Clone, Debug, PartialEq, Eq)]
struct Row<K> {
    key: K,
    /// Whether the row can take the focus or a choice.
    enabled: bool,
    /// The row's place in the source slice the index was built from.
    source: usize,
}

/// A collection's keys in logical order, built in one memo per collection
/// whenever its items change and read by every item part in O(1). The
/// first occurrence of a duplicate key wins; callers report
/// [`duplicates`](Self::duplicates) and never give a key a second identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Index<K: Eq + Hash> {
    rows: Vec<Row<K>>,
    /// Key → its place in `rows`.
    at: HashMap<K, usize>,
    duplicates: Vec<K>,
}

impl<K: Eq + Hash + Clone> Index<K> {
    /// Index `source`. `key` gives an entry's key, `None` for an entry that
    /// is not an item (a menu separator or heading); `enabled` whether the
    /// entry can take the focus or a choice.
    pub(crate) fn new<T>(
        source: &[T],
        key: impl Fn(&T) -> Option<&K>,
        enabled: impl Fn(&T) -> bool,
    ) -> Self {
        let mut index = Self {
            rows: Vec::with_capacity(source.len()),
            at: HashMap::with_capacity(source.len()),
            duplicates: Vec::new(),
        };
        let mut repeated = HashSet::new();
        for (place, entry) in source.iter().enumerate() {
            let Some(found) = key(entry) else {
                continue;
            };
            if index.at.contains_key(found) {
                if repeated.insert(found.clone()) {
                    index.duplicates.push(found.clone());
                }
            } else {
                index.at.insert(found.clone(), index.rows.len());
                index.rows.push(Row {
                    key: found.clone(),
                    enabled: enabled(entry),
                    source: place,
                });
            }
        }
        index
    }

    /// The index of a list's items: a disabled item is not enabled.
    pub(crate) fn of_items(items: &[ListItem<K>]) -> Self {
        Self::new(items, |item| Some(&item.key), |item| !item.disabled)
    }

    /// The distinct keys in order.
    pub(crate) fn keys(&self) -> Vec<K> {
        self.rows.iter().map(|row| row.key.clone()).collect()
    }

    /// How many distinct keys there are (an item's `set_size`).
    pub(crate) fn count(&self) -> usize {
        self.rows.len()
    }

    /// The ordinal of `key` among the distinct keys, from 0.
    pub(crate) fn position(&self, key: &K) -> Option<usize> {
        self.at.get(key).copied()
    }

    /// Where the first occurrence of `key` is in the source slice.
    pub(crate) fn source(&self, key: &K) -> Option<usize> {
        self.at.get(key).map(|&row| self.rows[row].source)
    }

    pub(crate) fn duplicates(&self) -> &[K] {
        &self.duplicates
    }

    pub(crate) fn enabled(&self, key: &K) -> bool {
        self.at.get(key).is_some_and(|&row| self.rows[row].enabled)
    }

    /// Navigate only among enabled and available keys, in logical item order.
    /// An empty/all-disabled/unmounted order has no target.
    pub(crate) fn navigate(
        &self,
        current: Option<&K>,
        movement: Movement,
        boundary: ListBoundary,
        available: impl Fn(&K) -> bool,
    ) -> Option<K> {
        let keys: Vec<&K> = self
            .rows
            .iter()
            .filter(|row| row.enabled && available(&row.key))
            .map(|row| &row.key)
            .collect();
        let last = keys.len().checked_sub(1)?;
        let current = current.and_then(|key| keys.iter().position(|candidate| *candidate == key));
        let index = match movement {
            Movement::First => 0,
            Movement::Last => last,
            Movement::Next => match current {
                Some(index) if index < last => index + 1,
                Some(_) if boundary == ListBoundary::Clamp => last,
                _ => 0,
            },
            Movement::Previous => match current {
                Some(index) if index > 0 => index - 1,
                Some(_) if boundary == ListBoundary::Clamp => 0,
                _ => last,
            },
        };
        Some(keys[index].clone())
    }
}
