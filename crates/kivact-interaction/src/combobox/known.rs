//! Role: Keep the labels of selected combobox options across replaced option results.
use crate::{ListItem, Selection};
use kivact_core::state::{ItemKey, Signal, Value, effect, memo, signal};

/// Owns retained item labels without copying the caller's selection.
struct LabelCache<K> {
    retained: Vec<ListItem<K>>,
}

impl<K: ItemKey> LabelCache<K> {
    fn observe(&mut self, source: &[ListItem<K>], selection: Selection<K>) {
        let selected = |key: &K| match selection {
            Selection::Single(v) => v.with(|v| v.as_ref() == Some(key)),
            Selection::Multiple(v) => v.with(|v| v.contains(key)),
        };
        self.retained.retain(|item| {
            selected(item.key()) && !source.iter().any(|row| row.key() == item.key())
        });
        self.retained
            .extend(source.iter().filter(|item| selected(item.key())).cloned());
    }
}

/// The options a combobox shows for its selection: the current options plus the selected ones a later result replaced, so a selected item keeps its label.
pub struct KnownOptions<K: ItemKey> {
    rows: Signal<Vec<ListItem<K>>>,
}

impl<K: ItemKey> Copy for KnownOptions<K> {}

impl<K: ItemKey> Clone for KnownOptions<K> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Starts the options a combobox shows, allocated before selection transaction callbacks.
pub fn known_options<K: ItemKey>(source: Value<Vec<ListItem<K>>>) -> KnownOptions<K> {
    KnownOptions {
        rows: signal(source.get_untracked()),
    }
}

impl<K: ItemKey> KnownOptions<K> {
    /// The options to show, including the retained selected ones.
    pub fn values(self) -> Value<Vec<ListItem<K>>> {
        memo(move || self.rows.get())
    }

    /// Follows the option source and the selection, retaining the labels of selected keys.
    pub fn follow(self, source: Value<Vec<ListItem<K>>>, selection: Selection<K>) {
        effect(move || {
            let source = source.get();
            let mut retained = LabelCache {
                retained: self.rows.get_untracked(),
            };
            retained.observe(&source, selection);
            let mut rows = source;
            for item in retained.retained {
                if !rows.iter().any(|row| row.key() == item.key()) {
                    rows.push(item);
                }
            }
            // xtask-allow(effect-writes): S-9 (a pure cache stays apart from authoritative state): the retained-label display cache follows option sources and selected keys across asynchronous result replacement; it never writes query, selection or active-item authority and reads its previous cache untracked.
            self.rows.set(rows);
        });
    }
}
