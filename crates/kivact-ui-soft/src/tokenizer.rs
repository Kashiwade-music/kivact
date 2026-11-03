//! Role: Choose and remove unique tokens from a keyed suggestion source.
use crate::typeahead::Typeahead;
use kivact::interaction::ListSelection;
use kivact::ui::ListItem;
use kivact::view::{IntoText, IntoValue};
use kivact::{Binding, binding, memo};
use std::hash::Hash;

pub type Tokenizer<K> = Typeahead<K>;

pub fn tokenizer<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl Into<Binding<Vec<K>>>,
) -> Tokenizer<K> {
    let value = value.into();
    let options = options.into_value();
    let add = binding(
        || None::<K>,
        move |key: Option<K>| {
            if let Some(key) = key {
                let mut selected = value.get_untracked();
                if !selected.contains(&key) {
                    selected.push(key);
                    value.set(selected);
                }
            }
        },
    );
    let available = memo(move || {
        value.with(|selected| {
            options.with(|options| {
                options
                    .iter()
                    .filter(|item| !selected.contains(item.key()))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    });
    let mut field = Typeahead::new(label, available, ListSelection::Single(add));
    field.chip_labels = Some(options);
    field.chips = Some(value);
    field
}
