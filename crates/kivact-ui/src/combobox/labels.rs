//! Role: Look up the display label of an option key in the combobox's option source.
use kivact_core::state::{ItemKey, Value};
use kivact_interaction::ListItem;

pub(super) fn label<K: ItemKey>(source: Value<Vec<ListItem<K>>>, key: &K) -> String {
    source.with(|rows| {
        rows.iter()
            .find(|row| row.key() == key)
            .map(ListItem::label)
            .unwrap_or_default()
    })
}
