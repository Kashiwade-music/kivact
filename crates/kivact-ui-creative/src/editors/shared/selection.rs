//! Role: Shared editor selection uses the public list gesture policy.

use kivact::Binding;
use kivact::interaction::{List, ListGesture, ListPolicy, ListSelection, use_list};
use kivact::ui::ListItem;
use kivact::view::{IntoValue, Key, KeyEvent, Modifiers};

/// Whether the key press is Ctrl+A or Meta+A (without Alt).
pub(crate) fn select_all(event: &KeyEvent) -> bool {
    (event.modifiers.ctrl || event.modifiers.meta)
        && !event.modifiers.alt
        && matches!(event.key, Key::Char('a' | 'A'))
}

pub(crate) fn controller(
    items: impl IntoValue<Vec<ListItem<u64>>>,
    selection: Binding<Vec<u64>>,
) -> List<u64> {
    use_list(items, ListSelection::Multiple(selection))
}

pub(crate) enum Press {
    Pending(u64),
    Select(ListGesture),
}

pub(crate) fn press(selected: &[u64], id: u64, modifiers: Modifiers, policy: &ListPolicy) -> Press {
    let gesture = ListGesture::from_modifiers(modifiers, policy);
    if gesture == ListGesture::Replace && selected.contains(&id) {
        Press::Pending(id)
    } else {
        Press::Select(gesture)
    }
}

pub(crate) fn merge_marquee(
    current: &[u64],
    hit: impl IntoIterator<Item = u64>,
    modifiers: Modifiers,
) -> Vec<u64> {
    let current = if modifiers.shift || modifiers.ctrl || modifiers.meta {
        current
    } else {
        &[]
    };
    let mut selected = Vec::new();
    for id in current.iter().copied().chain(hit) {
        if !selected.contains(&id) {
            selected.push(id);
        }
    }
    selected
}
