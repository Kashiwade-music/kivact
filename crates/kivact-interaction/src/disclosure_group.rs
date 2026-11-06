//! Role: Bind each disclosure panel directly to a single or multiple selection.
use crate::{DisclosureApi, IntoSelection, Selection, use_disclosure};
use kivact_core::state::{Binding, binding};

/// A group whose selection binding is the sole authority for expanded panels.
pub struct DisclosureGroupApi<K: 'static> {
    selection: Selection<K>,
}
impl<K: 'static> Copy for DisclosureGroupApi<K> {}
impl<K: 'static> Clone for DisclosureGroupApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
/// Create an accordion selection controller during setup.
pub fn use_disclosure_group<K: Clone + PartialEq + 'static>(
    selection: impl IntoSelection<K>,
) -> DisclosureGroupApi<K> {
    DisclosureGroupApi {
        selection: selection.into_selection(),
    }
}
impl<K: Clone + PartialEq + 'static> DisclosureGroupApi<K> {
    /// Bind the expanded state of one panel to its key.
    pub fn panel(self, key: K) -> DisclosureApi {
        use_disclosure(self.panel_open(key))
    }
    /// The writable expanded state of a panel; no parallel state is stored.
    pub fn panel_open(self, key: K) -> Binding<bool> {
        let read_key = key.clone();
        binding(
            move || match self.selection {
                Selection::Single(selected) => selected.get().as_ref() == Some(&read_key),
                Selection::Multiple(selected) => selected.get().contains(&read_key),
            },
            move |open| match self.selection {
                Selection::Single(selected) => {
                    if open {
                        selected.set(Some(key.clone()));
                    } else if selected.get_untracked().as_ref() == Some(&key) {
                        selected.set(None);
                    }
                }
                Selection::Multiple(selected) => {
                    let mut keys = selected.get_untracked();
                    keys.retain(|current| current != &key);
                    if open {
                        keys.push(key.clone());
                    }
                    selected.set(keys);
                }
            },
        )
    }
}
