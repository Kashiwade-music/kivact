//! Role: Reuse accepted identity metadata for proven local text updates.
//!
//! Accepted ID metadata is an immutable UI-thread Rc snapshot. A local build
//! takes only a thin clone; even an unwind cannot displace the accepted state.
use crate::a11y::state::Metadata;
use crate::model::Model;
use arvane as a;
use kivact_wire as wire;
use std::collections::BTreeSet;

impl Model {
    pub(super) fn build_local_text(
        &mut self,
        root: wire::NodeId,
        nodes: &BTreeSet<wire::NodeId>,
        engine: &mut a::Engine,
    ) -> kivact_types::protocol::SemanticsTree {
        let metadata = std::rc::Rc::clone(&self.a11y.metadata);
        self.build_local_text_with(root, nodes, &metadata, engine)
    }

    fn build_local_text_with(
        &mut self,
        root: wire::NodeId,
        nodes: &BTreeSet<wire::NodeId>,
        metadata: &Metadata,
        engine: &mut a::Engine,
    ) -> kivact_types::protocol::SemanticsTree {
        let mut dirty = nodes.clone();
        let mut queue: Vec<_> = dirty.iter().copied().collect();
        // Current naming policy has no implicit ancestor name. Only raw
        // relation sources depend on another node's content.
        while let Some(target) = queue.pop() {
            for source in metadata.reverse.get(&target).into_iter().flatten() {
                if dirty.insert(*source) {
                    queue.push(*source);
                }
            }
        }
        let mut order: Vec<_> = dirty.into_iter().collect();
        order.sort_by_key(|id| (self.tree().chain(Some(*id)).len(), *id));
        let mut nodes = Vec::new();
        let mut active = BTreeSet::new();
        for id in order {
            self.build_node(
                id,
                self.scroll_above(id),
                &metadata.clipped,
                &mut nodes,
                &mut active,
                engine,
            );
        }
        #[cfg(feature = "testing")]
        {
            self.a11y.constructed += nodes.len() as u64;
        }
        kivact_types::protocol::SemanticsTree {
            root: kivact_types::NodeId(root.0),
            focus: self.semantics_focus(&metadata.clipped),
            nodes,
        }
    }
}
