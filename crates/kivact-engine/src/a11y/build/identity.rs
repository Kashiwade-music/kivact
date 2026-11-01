//! Role: Map engine nodes and canvas regions to native accessibility identities and back.

use crate::model::Model;
use kivact_wire as wire;

impl Model {
    /// Resolve existing focus identities into the platform accessibility tree.
    pub(super) fn semantic_target(
        &self,
        target: wire::FocusTarget,
    ) -> Option<kivact_types::NodeId> {
        self.node(target.node)?;
        match target.region {
            None => Some(kivact_types::NodeId(target.node.0)),
            Some(region) => {
                self.node(target.node)?.region(region)?;
                self.a11y
                    .region_ids
                    .get(&(target.node, region))
                    .copied()
                    .map(kivact_types::NodeId)
            }
        }
    }

    pub(super) fn semantic_identity(
        &self,
        target: kivact_types::NodeId,
    ) -> Option<wire::FocusTarget> {
        if let Some((node, region)) = self.a11y.native_regions.get(&target.0) {
            self.node(*node)?.region(*region)?;
            Some(wire::FocusTarget {
                node: *node,
                region: Some(*region),
            })
        } else {
            self.node(wire::NodeId(target.0))?;
            Some(wire::FocusTarget {
                node: wire::NodeId(target.0),
                region: None,
            })
        }
    }

    pub(super) fn semantic_owner(&self, target: kivact_types::NodeId) -> Option<wire::NodeId> {
        if let Some((node, region)) = self.a11y.native_regions.get(&target.0) {
            self.node(*node)?.region(*region)?;
            Some(*node)
        } else {
            self.node(wire::NodeId(target.0))
                .map(|_| wire::NodeId(target.0))
        }
    }

    pub(super) fn prepare_region_ids(&mut self) {
        let targets: Vec<_> = self
            .tree()
            .preorder()
            .into_iter()
            .flat_map(|node| {
                self.node(node).into_iter().flat_map(move |n| {
                    n.scenes()
                        .iter()
                        .flatten()
                        .flat_map(move |s| s.regions.iter().map(move |r| (node, r.id)))
                })
            })
            .collect();
        for (node, region) in targets {
            self.region_identity(node, region);
        }
    }

    pub(super) fn region_identity(&mut self, owner: wire::NodeId, region: wire::RegionId) -> u64 {
        if let Some(&id) = self.a11y.region_ids.get(&(owner, region))
            && !self.tree().contains(wire::NodeId(id))
        {
            return id;
        }
        let candidate = self.next_semantics_identity();
        self.a11y.native_regions.insert(candidate, (owner, region));
        self.a11y.region_ids.insert((owner, region), candidate);
        candidate
    }

    pub(super) fn next_semantics_identity(&mut self) -> u64 {
        self.a11y.next_identity(&self.tree)
    }
}
