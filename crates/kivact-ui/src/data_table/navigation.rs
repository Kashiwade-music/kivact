//! Role: Link mounted group rows into the grid's logical keyboard order.
use super::metadata::Rows;
use kivact_core::custom::NodeRefExt;
use kivact_core::state::{ItemKey, Value, on_cleanup};
use kivact_core::view::{Handled, Key, KeyEvent, Node, NodeRef, node_ref};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum Target<K> {
    Cell(K, usize),
    Header(usize),
    Group(String),
}

/// Mounted references are a navigation cache, never a second focus or selection owner.
pub(super) struct NavigationCache<K: ItemKey>(Rc<RefCell<HashMap<Target<K>, NodeRef>>>);
impl<K: ItemKey> Clone for NavigationCache<K> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<K: ItemKey> NavigationCache<K> {
    pub(super) fn new() -> Self {
        Self(Rc::new(RefCell::new(HashMap::new())))
    }
    pub(super) fn bind(&self, root: Node, key: Target<K>) -> Node {
        let target = node_ref();
        self.0.borrow_mut().insert(key.clone(), target);
        let cache = self.clone();
        on_cleanup(move || {
            let mut entries = cache.0.borrow_mut();
            if entries
                .get(&key)
                .is_some_and(|current| current.same_ref(target))
            {
                entries.remove(&key);
            }
        });
        root.node_ref(target)
    }
    pub(super) fn from_cell(&self, key: &K, event: &KeyEvent, rows: Value<Rows<K>>) -> Handled {
        let delta = match event.key() {
            Key::ArrowUp => -1,
            Key::ArrowDown => 1,
            _ => return Handled::No,
        };
        let rows = rows.get_untracked();
        let Some(position) = rows.positions.get(key) else {
            return Handled::No;
        };
        let desired = position.saturating_add_signed(delta);
        let target = rows
            .groups
            .iter()
            .find(|(_, position)| **position == desired)
            .and_then(|(label, _)| self.0.borrow().get(&Target::Group(label.clone())).copied());
        if let Some(target) = target {
            target.focus();
            Handled::Yes
        } else {
            Handled::No
        }
    }
    pub(super) fn from_group(
        &self,
        label: &str,
        event: &KeyEvent,
        rows: Value<Rows<K>>,
        columns: Value<(Option<u32>, HashMap<usize, u32>)>,
    ) -> Handled {
        let rows = rows.get_untracked();
        let Some(position) = rows.groups.get(label) else {
            return Handled::No;
        };
        let command = event.modifiers().ctrl || event.modifiers().meta;
        if matches!(event.key(), Key::ArrowLeft | Key::ArrowRight)
            || matches!(event.key(), Key::Home | Key::End) && !command
        {
            return Handled::Yes;
        }
        let direction = match event.key() {
            Key::ArrowUp | Key::PageUp | Key::Home => -1,
            Key::ArrowDown | Key::PageDown | Key::End => 1,
            _ => return Handled::No,
        };
        let distance = if matches!(event.key(), Key::PageUp | Key::PageDown) {
            10
        } else {
            1
        };
        let desired = position.saturating_add_signed(direction * distance);
        let columns = columns.get_untracked().1;
        let entries = self.0.borrow();
        let mut candidates = entries
            .iter()
            .filter_map(|(key, reference)| {
                let (row, column) = match key {
                    Target::Cell(key, column) => (*rows.positions.get(key)?, *columns.get(column)?),
                    Target::Header(column) => (1, *columns.get(column)?),
                    Target::Group(label) => (*rows.groups.get(label)?, 1),
                };
                reference.is_visible().then_some((row, column, *reference))
            })
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(row, column, _)| (*row, *column));
        let target = if event.key() == Key::Home && command {
            candidates.first()
        } else if event.key() == Key::End && command {
            candidates.last()
        } else if direction < 0 {
            candidates
                .iter()
                .rev()
                .filter(|(row, _, _)| *row < *position)
                .min_by_key(|(row, column, _)| (row.abs_diff(desired), *column))
        } else {
            candidates
                .iter()
                .filter(|(row, _, _)| *row > *position)
                .min_by_key(|(row, column, _)| (row.abs_diff(desired), *column))
        };
        let target = target.map(|(_, _, reference)| *reference);
        drop(entries);
        if let Some(target) = target {
            target.focus();
        }
        Handled::Yes
    }
}
