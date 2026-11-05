//! Role: Retain only labels needed by selected keys across changing result sources.
use kivact_core::state::{ItemKey, Value, effect, memo, signal};
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
pub(super) fn labels<K: ItemKey>(
    source: Value<Vec<ListItem<K>>>,
    selection: Selection<K>,
) -> Value<Vec<ListItem<K>>> {
    let cache = signal(Vec::<ListItem<K>>::new());
    effect(move || {
        let source = source.get();
        let mut retained = LabelCache {
            retained: cache.get_untracked(),
        };
        retained.observe(&source, selection);
        cache.set(retained.retained);
    });
    memo(move || {
        let mut rows = source.get();
        for item in cache.get() {
            if !rows.iter().any(|row| row.key() == item.key()) {
                rows.push(item);
            }
        }
        rows
    })
}
pub(super) fn label<K: ItemKey>(source: Value<Vec<ListItem<K>>>, key: &K) -> String {
    source.with(|rows| {
        rows.iter()
            .find(|row| row.key() == key)
            .map(ListItem::label)
            .unwrap_or_default()
    })
}
