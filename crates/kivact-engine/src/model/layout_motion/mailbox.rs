//! Role: Record departed shared-layout boxes and hand them to arriving nodes.
//!
//! Invariants:
//! - A node is recorded once, when it leaves its mounted or arriving state
//!   (`Model::record_departures`); a node already leaving is not.
//! - A record is taken by at most one node: the one with the highest id
//!   among those that get their first box in the next pass with that id
//!   (the node created last).
//! - The mailbox is emptied at the end of every layout pass that retargets
//!   the runs, so a record lives until the next pass only.

use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use kivact_types::{NodeId, Rect};

use crate::contract::HostEvent;
use crate::model::Model;

impl Model {
    /// Report each layout id that two or more live nodes carry, once: the
    /// node created right after the first one is named. A duplicate is an
    /// error of the app (`HostEvent::DuplicateLayoutId`).
    pub(crate) fn report_duplicate_layout_ids(&mut self) {
        let duplicates: Vec<(NodeId, Arc<str>)> = self
            .sources
            .duplicate_layout_ids()
            .map(|(node, key)| (node, key.clone()))
            .collect();
        for (node, id) in duplicates {
            if self.layout_motion.reported_ids.note(id.clone()) {
                self.push(HostEvent::DuplicateLayoutId { node, id });
            }
        }
    }

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
            let Some(key) = node.layout_id().cloned() else {
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
        let keys: BTreeSet<Arc<str>> = self.layout_motion.shared.departed.keys().cloned().collect();
        let mut arrivals = Vec::new();
        for key in keys {
            let taker = self
                .sources
                .layout_id_nodes(&key)
                .filter(|id| !known.contains(id) && self.sources.has_layout_transition(*id))
                .max();
            let Some(id) = taker else {
                continue;
            };
            if let Some(rect) = self.layout_motion.shared.departed.remove(&key) {
                arrivals.push((id, rect));
            }
        }
        arrivals
    }
}
