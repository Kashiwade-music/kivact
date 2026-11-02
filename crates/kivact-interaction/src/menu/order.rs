//! Role: Pure menu lookup and retained-level ordering helpers.

use super::state::{Edge, MenuEntry, MenuRow};

pub(super) type Rows<K> = Vec<(MenuRow<K>, MenuEntry<K>)>;

/// The entries of the level reached by `prefix`, or none if the path no
/// longer leads to a submenu.
fn find_level<'a, K: PartialEq>(
    entries: &'a [MenuEntry<K>],
    prefix: &[K],
) -> Option<&'a [MenuEntry<K>]> {
    let mut level = entries;
    for key in prefix {
        level = item_in(level, key)?.submenu_entries()?;
    }
    Some(level)
}

/// The rows of a level with its groups opened in place.
fn flat<'a, K>(level: &'a [MenuEntry<K>], out: &mut Vec<&'a MenuEntry<K>>) {
    for entry in level {
        match entry.group_entries() {
            Some(children) => {
                out.push(entry);
                flat(children, out);
            }
            None => out.push(entry),
        }
    }
}

fn item_in<'a, K: PartialEq>(level: &'a [MenuEntry<K>], key: &K) -> Option<&'a MenuEntry<K>> {
    let mut rows = Vec::new();
    flat(level, &mut rows);
    rows.into_iter().find(|entry| entry.key() == Some(key))
}

/// Whether the item `key` of the level `prefix` exists and can be used.
pub(super) fn item_enabled<K: PartialEq>(entries: &[MenuEntry<K>], prefix: &[K], key: &K) -> bool {
    find_level(entries, prefix)
        .and_then(|level| item_in(level, key))
        .is_some_and(|entry| !entry.is_disabled())
}

/// How many levels, counted from the top level, the current entries keep:
/// the open submenus whose item still exists, enabled, up to the level whose
/// focused item still exists. Zero means the top level lost its focused item.
pub(super) fn kept_levels<K: PartialEq>(
    entries: &[MenuEntry<K>],
    path: &[K],
    focus: Option<&(usize, K)>,
) -> usize {
    let mut levels = vec![entries];
    for key in path {
        let Some(last) = levels.last().copied() else {
            break;
        };
        match item_in(last, key) {
            Some(entry) if !entry.is_disabled() => match entry.submenu_entries() {
                Some(children) => levels.push(children),
                None => break,
            },
            _ => break,
        }
    }
    let mut count = levels.len();
    if let Some((depth, key)) = focus
        && *depth < count
        && item_in(levels[*depth], key).is_none()
    {
        count = *depth;
    }
    count
}

pub(super) fn rows<K: Clone + PartialEq>(entries: &[MenuEntry<K>], prefix: &[K]) -> Rows<K> {
    let Some(level) = find_level(entries, prefix) else {
        return Vec::new();
    };
    let mut flattened = Vec::new();
    flat(level, &mut flattened);
    let (mut separators, mut labels) = (0, 0);
    flattened
        .into_iter()
        .filter_map(|entry| {
            let row = match (entry.key(), entry.item_kind()) {
                (Some(key), Some(kind)) => MenuRow::Item {
                    key: key.clone(),
                    kind,
                },
                // A group shows its label as a heading, if it has one.
                _ if entry.is_heading() || entry.is_group() => {
                    if entry.is_group() && entry.label.get().is_empty() {
                        return None;
                    }
                    labels += 1;
                    MenuRow::Label(labels - 1)
                }
                _ => {
                    separators += 1;
                    MenuRow::Separator(separators - 1)
                }
            };
            let entry = if entry.is_group() {
                entry.heading_of_group()
            } else {
                entry.clone()
            };
            Some((row, entry))
        })
        .collect()
}

/// The first or last enabled item of a level.
pub(super) fn edge_item<K: Clone>(rows: &Rows<K>, edge: Edge) -> Option<K> {
    let mut enabled = rows
        .iter()
        .filter(|(_, entry)| !entry.is_disabled())
        .filter_map(|(_, entry)| entry.key().cloned());
    match edge {
        Edge::First => enabled.next(),
        Edge::Last => enabled.last(),
    }
}
