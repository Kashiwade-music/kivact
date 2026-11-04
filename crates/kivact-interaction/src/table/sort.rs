//! Role: Compute ordered sort transitions while caller bindings remain authoritative.
use super::data::SortOrder;
use kivact_core::state::ItemKey;

#[derive(Clone, PartialEq)]
pub(super) enum Sort<C> {
    Unsorted,
    Sorted(Vec<(C, SortOrder)>),
}
impl<C> Sort<C> {
    pub(super) fn from_keys(keys: Vec<(C, SortOrder)>) -> Self {
        if keys.is_empty() {
            Self::Unsorted
        } else {
            Self::Sorted(keys)
        }
    }
    pub(super) fn keys(self) -> Vec<(C, SortOrder)> {
        match self {
            Self::Unsorted => Vec::new(),
            Self::Sorted(keys) => keys,
        }
    }
}
pub(super) struct Event<C> {
    pub(super) current: Sort<C>,
    pub(super) column: C,
    pub(super) extend: bool,
    pub(super) multiple: bool,
}
impl<C: ItemKey> Sort<C> {
    pub(super) fn activate(event: Event<C>) -> Self {
        let mut keys = event.current.keys();
        let index = keys.iter().position(|(key, _)| *key == event.column);
        let next = match index.map(|index| keys[index].1) {
            None => Some(SortOrder::Ascending),
            Some(SortOrder::Ascending) => Some(SortOrder::Descending),
            Some(SortOrder::Descending) if event.multiple => None,
            Some(SortOrder::Descending) => Some(SortOrder::Ascending),
        };
        if !event.extend || !event.multiple {
            keys.clear();
            keys.extend(next.map(|next| (event.column, next)));
        } else if let Some(index) = index {
            if let Some(next) = next {
                keys[index].1 = next;
            } else {
                keys.remove(index);
            }
        } else if let Some(next) = next {
            keys.push((event.column, next));
        }
        Sort::from_keys(keys)
    }
}
