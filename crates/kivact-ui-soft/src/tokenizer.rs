//! Role: Choose and remove unique tokens from a keyed suggestion source.
use crate::typeahead::Typeahead;
use kivact::interaction::ListItem;
use kivact::interaction::Selection;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::ItemKey;
use kivact::state::binding;
use kivact::state::memo;

pub type Tokenizer<K> = Typeahead<K>;

pub fn tokenizer<K: ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl IntoBinding<Vec<K>>,
) -> Tokenizer<K> {
    let value = value.into_binding();
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
    let mut field = Typeahead::new(label, available, Selection::Single(add)).exclude_selected();
    field.chip_labels = Some(options);
    field.chips = Some(value);
    field
}
