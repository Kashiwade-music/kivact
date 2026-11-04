//! Role: Plans where a dragged row lands: before, after or inside a target.

use crate::outliner_data::{TreeDrop, TreeItem};

/// Which part of a row the pointer is over during a drag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Placement {
    Before,
    After,
    Into,
}

/// The row under the dragged pointer and the part of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Plan {
    pub(super) over: u64,
    pub(super) placement: Placement,
}

/// The drop for `dragged` released over `over`, `fraction` of the way down
/// that row (0 at its top edge). `None` when the item cannot go there.
pub(super) fn plan_drop(
    items: &[TreeItem],
    dragged: u64,
    over: u64,
    fraction: f64,
) -> Option<(Plan, TreeDrop)> {
    let parent_of = |id: u64| {
        items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.parent)
    };
    // Not on itself, and not into its own subtree.
    let mut at = Some(over);
    for _ in 0..=items.len() {
        match at {
            Some(id) if id == dragged => return None,
            Some(id) => at = parent_of(id)?,
            None => break,
        }
    }
    let placement = if fraction < 1.0 / 3.0 {
        Placement::Before
    } else if fraction > 2.0 / 3.0 {
        Placement::After
    } else {
        Placement::Into
    };
    let children = |parent: Option<u64>| {
        items
            .iter()
            .filter(|item| item.parent == parent && item.id != dragged)
            .map(|item| item.id)
            .collect::<Vec<_>>()
    };
    let drop = match placement {
        Placement::Into => TreeDrop {
            id: dragged,
            target: Some(over),
            after: children(Some(over)).last().copied(),
        },
        Placement::After | Placement::Before => {
            let parent = parent_of(over)?;
            let siblings = children(parent);
            let at = siblings.iter().position(|id| *id == over)?;
            TreeDrop {
                id: dragged,
                target: parent,
                after: match placement {
                    Placement::After => Some(over),
                    _ => at.checked_sub(1).map(|before| siblings[before]),
                },
            }
        }
    };
    Some((Plan { over, placement }, drop))
}
