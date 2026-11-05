//! Role: Retain only labels needed by selected keys across changing result sources.
use kivact_core::state::{ItemKey, Signal, Value, effect, memo, signal};
use kivact_interaction::{ListItem, Selection};
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
/// A true item-label cache, allocated before selection transaction callbacks.
pub(super) struct KnownLabels<K: ItemKey> {
    rows: Signal<Vec<ListItem<K>>>,
}
impl<K: ItemKey> Copy for KnownLabels<K> {}
impl<K: ItemKey> Clone for KnownLabels<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: ItemKey> KnownLabels<K> {
    pub(super) fn new(source: Value<Vec<ListItem<K>>>) -> Self {
        Self {
            rows: signal(source.get_untracked()),
        }
    }
    pub(super) fn values(self) -> Value<Vec<ListItem<K>>> {
        memo(move || self.rows.get())
    }
    pub(super) fn observe(self, source: Value<Vec<ListItem<K>>>, selection: Selection<K>) {
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
pub(super) fn label<K: ItemKey>(source: Value<Vec<ListItem<K>>>, key: &K) -> String {
    source.with(|rows| {
        rows.iter()
            .find(|row| row.key() == key)
            .map(ListItem::label)
            .unwrap_or_default()
    })
}
