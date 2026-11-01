//! Role: Reuse accepted identity metadata for proven local text updates.
//!
//! Accepted ID metadata is an immutable UI-thread Rc snapshot. A local build
//! takes only a thin clone; even an unwind cannot displace the accepted state.
use super::{Model, a, wire};
use crate::a11y::state::Metadata;
use std::collections::BTreeSet;

impl Model {
    pub(super) fn build_local_text(
        &mut self,
        root: wire::NodeId,
        engine: &mut a::Engine,
    ) -> ui_types::SemanticsTree {
        let metadata = std::rc::Rc::clone(&self.a11y.metadata);
        self.build_local_text_with(root, &metadata, engine)
    }

    fn build_local_text_with(
        &mut self,
        root: wire::NodeId,
        metadata: &Metadata,
        engine: &mut a::Engine,
    ) -> ui_types::SemanticsTree {
        let mut dirty = self.a11y.nodes.clone();
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
        self.a11y.prepared = None;
        ui_types::SemanticsTree {
            root: ui_types::NodeId(root.0),
            focus: self.semantics_focus(&metadata.clipped),
            nodes,
        }
    }
}
