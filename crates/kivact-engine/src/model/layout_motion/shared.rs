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
use crate::tree::node::HostNode;

/// The shared-layout id of `node`. §12.11 keeps it on the node's properties,
/// which do not carry one yet, so no node has an id (the value the removed
/// `LayoutProps::layout_id` always had).
fn shared_layout_id(_node: &HostNode) -> Option<Arc<str>> {
    None
}

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
            if self.tree.leaving(id) {
                continue;
            }
            let Some(key) = shared_layout_id(node) else {
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
            if node.layout().is_none_or(|l| l.layout_transition.is_none()) {
                continue;
            }
            let Some(key) = shared_layout_id(node) else {
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
