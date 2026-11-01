//! Role: Logical list order normalizes keys and chooses enabled destinations.

use super::data::{ListBoundary, ListItem};
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

/// Normalized logical order. The first occurrence of a duplicate key wins;
/// callers report `duplicates()` and never silently give it a second identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Order<K: Eq + Hash> {
    items: Vec<(K, bool)>,
    index: HashMap<K, usize>,
    duplicates: Vec<K>,
}

impl<K: Eq + Hash + Clone> Order<K> {
    pub(crate) fn new(items: Vec<ListItem<K>>) -> Self {
        let mut order = Self {
            items: Vec::with_capacity(items.len()),
            index: HashMap::with_capacity(items.len()),
            duplicates: Vec::new(),
        };
        let mut duplicates = HashSet::new();
        for item in items {
            if order.index.contains_key(&item.key) {
                if duplicates.insert(item.key.clone()) {
                    order.duplicates.push(item.key);
                }
            } else {
                order.index.insert(item.key.clone(), order.items.len());
                order.items.push((item.key, item.disabled));
            }
        }
        order
    }

    pub(super) fn keys(&self) -> Vec<K> {
        self.items.iter().map(|(key, _)| key.clone()).collect()
    }
    pub(super) fn position(&self, key: &K) -> Option<usize> {
        self.index.get(key).copied()
    }

    pub(crate) fn duplicates(&self) -> &[K] {
        &self.duplicates
    }

    pub(crate) fn enabled(&self, key: &K) -> bool {
        self.index
            .get(key)
            .is_some_and(|&index| !self.items[index].1)
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
            .items
            .iter()
            .filter(|item| !item.1 && available(&item.0))
            .map(|item| &item.0)
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
