//! Role: The keys a collection's selection holds, followed per key.
//!
//! One memo holds the selected keys as a set. One effect compares each new
//! set with the previous one and notifies only the notifiers of the keys
//! that entered or left it. An item reads its key in O(1) and follows only
//! its own notifier, so a selection change costs O(S) once per collection
//! plus O(1) per changed key, never O(S) per item. A read is always fresh
//! (the set is a memo); the notification arrives with the turn's user
//! effects.

use crate::Selection;
use kivact_core::state::{Notifier, Value, Var, effect, memo, notifier, on_cleanup, var};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// The notifiers of the items that read a key, by key.
type Watchers<K> = Rc<RefCell<HashMap<K, Vec<Notifier>>>>;

/// Which keys a selection holds, read per key in O(1). Copy.
pub(crate) struct Membership<K: kivact_core::state::ItemKey> {
    /// The selected keys: the collection's one memo of them.
    keys: Value<HashSet<K>>,
    /// Shared with the effect that notifies and with each item's cleanup.
    watchers: Var<Watchers<K>>,
}

impl<K: kivact_core::state::ItemKey> Clone for Membership<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for Membership<K> {}

impl<K: kivact_core::state::ItemKey> Membership<K> {
    /// Follow `selection`. Setup only, in the collection's owner.
    #[track_caller]
    pub(crate) fn new(selection: Selection<K>) -> Self {
        let keys = memo(move || match selection {
            Selection::Single(binding) => {
                binding.with(|key| key.iter().cloned().collect::<HashSet<K>>())
            }
            Selection::Multiple(binding) => {
                binding.with(|keys| keys.iter().cloned().collect::<HashSet<K>>())
            }
        });
        let watchers: Watchers<K> = Rc::new(RefCell::new(HashMap::new()));
        let notified = watchers.clone();
        let previous = RefCell::new(keys.get_untracked());
        // Derives notifications only; it writes no state.
        effect(move || {
            let changed = keys.with(|next| {
                let mut previous = previous.borrow_mut();
                let changed = next
                    .symmetric_difference(&previous)
                    .cloned()
                    .collect::<Vec<K>>();
                *previous = next.clone();
                changed
            });
            if changed.is_empty() {
                return;
            }
            let targets = {
                let watchers = notified.borrow();
                changed
                    .iter()
                    .filter_map(|key| watchers.get(key))
                    .flatten()
                    .copied()
                    .collect::<Vec<Notifier>>()
            };
            for target in targets {
                target.notify();
            }
        });
        Self {
            keys,
            watchers: var(watchers),
        }
    }

    /// A notifier for `key`, registered until the current owner goes.
    /// Setup only, in the item's owner.
    #[track_caller]
    pub(crate) fn watch(self, key: K) -> Notifier {
        let watch = notifier();
        let watchers = self.watchers.get();
        watchers
            .borrow_mut()
            .entry(key.clone())
            .or_default()
            .push(watch);
        on_cleanup(move || {
            let mut watchers = watchers.borrow_mut();
            if let Some(list) = watchers.get_mut(&key) {
                list.retain(|candidate| *candidate != watch);
                if list.is_empty() {
                    watchers.remove(&key);
                }
            }
        });
        watch
    }

    /// Whether the selection holds `key`. The running computation follows
    /// `watch` (made by [`watch`](Self::watch) for the same key) and
    /// nothing else.
    pub(crate) fn contains(self, key: &K, watch: Notifier) -> bool {
        watch.track();
        self.keys.with_untracked(|keys| keys.contains(key))
    }

    /// The selected keys as one set, tracked as a whole: for computations
    /// over the whole collection (the active row, the tab stop), never per item.
    pub(crate) fn keys(self) -> Value<HashSet<K>> {
        self.keys
    }
}
