//! Role: Derive full logical table coordinates without copying caller state.
use super::types::Key;
use kivact_core::state::{Binding, ItemKey, Value, memo};
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub(super) struct Rows<K: ItemKey> {
    pub(super) count: Option<u32>,
    pub(super) positions: HashMap<K, u32>,
    pub(super) groups: HashMap<String, u32>,
}

pub(super) fn columns(
    count: usize,
    accessories: [(bool, usize, &'static str); 4],
    actions: bool,
    order: Option<Value<Vec<usize>>>,
) -> Value<(Option<u32>, HashMap<usize, u32>)> {
    memo(move || {
        let mut data = (0..count).collect::<Vec<_>>();
        if let Some(order) = order {
            let ranks = order.with(|order| {
                order
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(rank, key)| (key, rank))
                    .collect::<HashMap<_, _>>()
            });
            data.sort_by_key(|key| ranks.get(key).copied().unwrap_or(usize::MAX));
        }
        let mut columns = accessories
            .iter()
            .filter(|(present, _, _)| *present)
            .map(|(_, key, _)| *key)
            .collect::<Vec<_>>();
        columns.extend(data);
        if actions {
            columns.push(super::column_keys::ACTIONS);
        }
        let count = u32::try_from(columns.len()).ok();
        let positions = columns
            .into_iter()
            .enumerate()
            .map(|(index, key)| (key, u32::try_from(index + 1).unwrap_or(0)))
            .collect();
        (count, positions)
    })
}

pub(super) fn rows<R: Clone + 'static, K: ItemKey>(
    source: Value<Vec<R>>,
    ordered: Value<Vec<K>>,
    key: Key<R, K>,
    group: Option<Key<R, String>>,
    page: Option<(Binding<usize>, Value<usize>)>,
    remote: bool,
) -> Value<Rows<K>> {
    memo(move || {
        let ordered = ordered.get();
        let size = page.map_or(ordered.len().max(1), |(_, size)| size.get().max(1));
        let current = page.map_or(0, |(index, _)| index.get());
        let mut result = Rows {
            count: None,
            positions: HashMap::new(),
            groups: HashMap::new(),
        };
        if remote {
            if group.is_none() {
                let offset = current.saturating_mul(size);
                result.positions = ordered
                    .into_iter()
                    .enumerate()
                    .map(|(index, key)| {
                        (
                            key,
                            u32::try_from(offset.saturating_add(index).saturating_add(2))
                                .unwrap_or(0),
                        )
                    })
                    .collect();
            }
            return result;
        }
        let current = current.min(ordered.len().div_ceil(size).saturating_sub(1));
        let labels = group.as_ref().map(|group| {
            source.with(|rows| {
                rows.iter()
                    .map(|row| (key(row), group(row)))
                    .collect::<HashMap<_, _>>()
            })
        });
        let mut position = 1usize;
        for (page, keys) in ordered.chunks(size).enumerate() {
            let mut groups = Vec::<(String, Vec<K>)>::new();
            for key in keys {
                let label = labels
                    .as_ref()
                    .and_then(|labels| labels.get(key))
                    .cloned()
                    .unwrap_or_default();
                if let Some((_, rows)) = groups.iter_mut().find(|(group, _)| *group == label) {
                    rows.push(key.clone());
                } else {
                    groups.push((label, vec![key.clone()]));
                }
            }
            for (label, keys) in groups {
                if labels.is_some() {
                    position = position.saturating_add(1);
                    if page == current {
                        result
                            .groups
                            .insert(label, u32::try_from(position).unwrap_or(0));
                    }
                }
                for key in keys {
                    position = position.saturating_add(1);
                    result
                        .positions
                        .insert(key, u32::try_from(position).unwrap_or(0));
                }
            }
        }
        result.count = u32::try_from(position).ok();
        result
    })
}
