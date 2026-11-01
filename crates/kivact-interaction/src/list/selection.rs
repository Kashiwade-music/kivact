//! Role: Selection transitions preserve caller order and hidden selections.

use super::data::{ListGesture, ListMissingAnchor, ListRangeMode};
use std::collections::HashSet;
use std::hash::Hash;

/// The next caller-owned selection and the next interaction anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SelectionTransition<K> {
    pub(crate) selected: Vec<K>,
    pub(crate) anchor: Option<K>,
}

/// Apply a gesture to selectable keys in visible order.
///
/// Keys absent from `visible` cannot be targeted. Reordering or filtering does
/// not itself remove selected keys. Replace uses visible order; toggle and add
/// preserve existing order and append new keys in visible order. Duplicate
/// visible and selected keys are interpreted once, at their first occurrence.
/// A range keeps its anchor; a missing anchor establishes the chosen fallback.
pub(crate) fn selection_transition<K: Eq + Hash + Clone>(
    selected: &[K],
    anchor: Option<&K>,
    visible: &[K],
    target: &K,
    gesture: ListGesture,
    missing_anchor: ListMissingAnchor,
) -> SelectionTransition<K> {
    let mut seen = HashSet::new();
    let visible: Vec<&K> = visible.iter().filter(|key| seen.insert(*key)).collect();
    let Some(target_index) = visible.iter().position(|key| *key == target) else {
        return SelectionTransition {
            selected: selected.to_vec(),
            anchor: anchor.cloned(),
        };
    };
    let mut selected_set = HashSet::new();
    let existing: Vec<K> = selected
        .iter()
        .filter(|key| selected_set.insert((*key).clone()))
        .cloned()
        .collect();
    match gesture {
        ListGesture::Replace => SelectionTransition {
            selected: vec![target.clone()],
            anchor: Some(target.clone()),
        },
        ListGesture::Toggle => {
            let mut next = existing;
            if selected_set.contains(target) {
                next.retain(|key| key != target);
            } else {
                next.push(target.clone());
            }
            SelectionTransition {
                selected: next,
                anchor: Some(target.clone()),
            }
        }
        ListGesture::Extend { mode } => {
            let start = anchor
                .and_then(|anchor| visible.iter().position(|key| *key == anchor))
                .or_else(|| match missing_anchor {
                    ListMissingAnchor::Target => None,
                    ListMissingAnchor::FirstSelected => {
                        visible.iter().position(|key| selected_set.contains(*key))
                    }
                })
                .unwrap_or(target_index);
            let mut next = match mode {
                ListRangeMode::Replace => {
                    selected_set.clear();
                    Vec::new()
                }
                ListRangeMode::Add => existing,
            };
            for key in &visible[start.min(target_index)..=start.max(target_index)] {
                if selected_set.insert((*key).clone()) {
                    next.push((*key).clone());
                }
            }
            SelectionTransition {
                selected: next,
                anchor: Some(visible[start].clone()),
            }
        }
    }
}
