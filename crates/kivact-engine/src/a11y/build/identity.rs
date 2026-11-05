//! Role: Map engine nodes and canvas regions to native accessibility identities and back.

use crate::model::Model;
use kivact_types as types;

impl Model {
    /// Resolve existing focus identities into the platform accessibility tree.
    pub(super) fn semantic_target(
        &self,
        target: types::FocusTarget,
    ) -> Option<kivact_types::SemanticsId> {
        self.node(target.node)?;
        match target.region {
            None => Some(kivact_types::SemanticsId(target.node.0)),
            Some(region) => {
                self.node(target.node)?.region(region)?;
                self.a11y
                    .region_ids
                    .get(&(target.node, region))
                    .copied()
                    .map(kivact_types::SemanticsId)
            }
        }
    }

    pub(super) fn semantic_identity(
        &self,
        target: kivact_types::SemanticsId,
    ) -> Option<types::FocusTarget> {
        if let Some((node, region)) = self.a11y.native_regions.get(&target.0) {
            self.node(*node)?.region(*region)?;
            Some(types::FocusTarget {
                node: *node,
                region: Some(*region),
            })
        } else {
            self.node(types::NodeId(target.0))?;
            Some(types::FocusTarget {
                node: types::NodeId(target.0),
                region: None,
            })
        }
    }

    pub(super) fn semantic_owner(
        &self,
        target: kivact_types::SemanticsId,
    ) -> Option<types::NodeId> {
        if let Some((node, region)) = self.a11y.native_regions.get(&target.0) {
            self.node(*node)?.region(*region)?;
            Some(*node)
        } else {
            self.node(types::NodeId(target.0))
                .map(|_| types::NodeId(target.0))
        }
    }

    pub(super) fn prepare_region_ids(&mut self) {
        let targets: Vec<_> = self
            .semantic_order()
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

    pub(super) fn region_identity(&mut self, owner: types::NodeId, region: types::RegionId) -> u64 {
        if let Some(&id) = self.a11y.region_ids.get(&(owner, region))
            && !self.tree().contains(types::NodeId(id))
        {
            return id;
        }
        let candidate = self.next_semantics_identity();
        self.a11y.native_regions.insert(candidate, (owner, region));
        self.a11y.region_ids.insert((owner, region), candidate);
        candidate
    }

    pub(in crate::a11y) fn next_semantics_identity(&mut self) -> u64 {
        loop {
            let id = self.a11y.ids.next().0;
            if !self.tree.contains(types::NodeId(id)) {
                return id;
            }
        }
    }
}
