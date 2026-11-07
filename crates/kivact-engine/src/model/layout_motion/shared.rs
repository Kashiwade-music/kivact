//! Role: Keep the window boxes of removed nodes with a layout id for the next layout pass.
//!
//! Invariants:
//! - A node is recorded once, when it leaves its mounted or arriving state
//!   (`Model::record_departures`); a node already leaving is not.
//! - A record is taken by at most one node: the one with the highest id
//!   among those that get their first box in the next pass with that id
//!   (the node created last).
//! - The mailbox is emptied at the end of every layout pass that retargets
//!   the runs, so a record lives until the next pass only.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use kivact_types::{NodeId, Rect};

use crate::model::Model;
use crate::tree::node::NodePresence;

/// Window layout boxes of removed nodes with a layout id, kept until the
/// next layout pass has had the chance to use them (a per-pass mailbox).
#[derive(Default)]
pub(crate) struct SharedLayouts {
    /// The departed box of each layout id.
    pub(super) departed: HashMap<Arc<str>, Rect>,
}

impl Model {
    /// Record the window boxes of the nodes of `subtree` that depart now
    /// with a layout id: only those not leaving yet, so each is recorded
    /// once. A later node of the subtree replaces an earlier one's record.
    pub(crate) fn record_departures(&mut self, subtree: &[NodeId]) {
        for &id in subtree {
            let Some(node) = self.tree.get(id) else {
                continue;
            };
            if matches!(node.presence, NodePresence::Leaving(_)) {
                continue;
            }
            let Some(key) = node.layout().and_then(|l| l.layout_id.clone()) else {
                continue;
            };
            if let Some(rect) = self.window_layout_rect(id) {
                self.layout_motion.shared.departed.insert(key, rect);
            }
        }
    }

    /// The departed boxes that the nodes getting their first layout box in
    /// this pass take over: a node with a layout transition and a layout id
    /// that has a record, not in `previous` (which holds the boxes of the
    /// nodes laid out before). Each id goes to the node with the highest
    /// id and is taken out of the mailbox.
    pub(crate) fn take_arrivals(&mut self, previous: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
        if self.layout_motion.shared.departed.is_empty() {
            return Vec::new();
        }
        let known: HashSet<NodeId> = previous.iter().map(|(id, _)| *id).collect();
        let mut chosen: BTreeMap<Arc<str>, NodeId> = BTreeMap::new();
        for (id, node) in self.tree.iter() {
            if known.contains(&id) {
                continue;
            }
            let Some(layout) = node.layout().filter(|l| l.layout_transition.is_some()) else {
                continue;
            };
            let Some(key) = layout.layout_id.clone() else {
                continue;
            };
            if !self.layout_motion.shared.departed.contains_key(&key) {
                continue;
            }
            let slot = chosen.entry(key).or_insert(id);
            if id > *slot {
                *slot = id;
            }
        }
        chosen
            .into_iter()
            .filter_map(|(key, id)| {
                let rect = self.layout_motion.shared.departed.remove(&key)?;
                Some((id, rect))
            })
            .collect()
    }
}
