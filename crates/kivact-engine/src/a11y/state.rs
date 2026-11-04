//! Role: Accessibility identities, accepted fingerprints and pending invalidation.
//!
//! Payload ownership belongs exclusively to the shell. IDs may be allocated
//! before acceptance; fingerprints and pending dirt commit only after acceptance.
use kivact_types::{NodeId, RegionId};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Clone, Default)]
pub(super) struct Metadata {
    pub clipped: BTreeSet<NodeId>,
    pub parents: BTreeMap<NodeId, Option<NodeId>>,
    pub children: BTreeMap<NodeId, u64>,
    pub reverse: BTreeMap<NodeId, BTreeSet<NodeId>>,
    pub regions: BTreeSet<(NodeId, RegionId)>,
    pub authored: BTreeSet<(NodeId, u64)>,
    pub live: BTreeSet<u64>,
    pub modal: Option<NodeId>,
}

/// What changed since the last accepted publication.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Dirt {
    /// Published as accepted.
    Clean,
    /// Only the payloads of `nodes` changed, by proven local text writes: the
    /// accepted identity metadata is reused.
    LocalText { nodes: BTreeSet<NodeId> },
    /// The identity and order metadata is enumerated again; besides what it
    /// finds, `nodes` (own payloads), `subtrees` (whole subtrees) and `removed`
    /// are dirty.
    Metadata {
        nodes: BTreeSet<NodeId>,
        subtrees: BTreeSet<NodeId>,
        removed: BTreeSet<NodeId>,
    },
    /// A complete replacement (initial, reactivated, new root, rejected).
    Full,
}

#[derive(Clone)]
pub(crate) struct A11yState {
    pub(super) native_regions: BTreeMap<u64, (NodeId, RegionId)>,
    pub(super) region_ids: BTreeMap<(NodeId, RegionId), u64>,
    pub(super) next_semantics_id: u64,
    pub(super) authored: BTreeMap<(NodeId, u64), u64>,
    pub(super) hashes: BTreeMap<u64, u64>,
    pub(super) root: Option<kivact_types::SemanticsId>,
    pub(super) empty_root: Option<u64>,
    pub(super) focus: Option<kivact_types::SemanticsId>,
    /// What changed since the last accepted publication.
    pub(super) dirt: Dirt,
    pub(super) metadata: Rc<Metadata>,
    #[cfg(feature = "testing")]
    pub(crate) constructed: u64,
    #[cfg(feature = "testing")]
    pub(crate) published: u64,
}
impl Default for A11yState {
    fn default() -> Self {
        Self {
            native_regions: BTreeMap::new(),
            region_ids: BTreeMap::new(),
            next_semantics_id: u64::MAX,
            authored: BTreeMap::new(),
            hashes: BTreeMap::new(),
            root: None,
            empty_root: None,
            focus: None,
            dirt: Dirt::Full,
            metadata: Rc::default(),
            #[cfg(feature = "testing")]
            constructed: 0,
            #[cfg(feature = "testing")]
            published: 0,
        }
    }
}
impl A11yState {
    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) fn publication_pending(&self) -> bool {
        self.dirt != Dirt::Clean
    }
    pub(crate) fn invalidate(&mut self) {
        self.dirt = Dirt::Full;
    }
    pub(crate) fn local_text(&mut self, id: NodeId) {
        match &mut self.dirt {
            Dirt::Clean => {
                self.dirt = Dirt::LocalText {
                    nodes: BTreeSet::from([id]),
                };
            }
            Dirt::LocalText { nodes } | Dirt::Metadata { nodes, .. } => {
                nodes.insert(id);
            }
            Dirt::Full => {}
        }
    }
    pub(crate) fn node(&mut self, id: NodeId) {
        if let Some((nodes, _, _)) = self.metadata_dirt() {
            nodes.insert(id);
        }
    }
    pub(crate) fn subtree(&mut self, id: NodeId) {
        if let Some((_, subtrees, _)) = self.metadata_dirt() {
            subtrees.insert(id);
        }
    }
    pub(crate) fn request_publication(&mut self, removed: impl Iterator<Item = NodeId>) {
        if let Some((_, _, gone)) = self.metadata_dirt() {
            gone.extend(removed);
        }
    }
    /// Raise the dirt to `Metadata` (keeping a local-text level's nodes); its
    /// sets (own payloads, subtrees, removed), or `None` at `Full`.
    fn metadata_dirt(
        &mut self,
    ) -> Option<(
        &mut BTreeSet<NodeId>,
        &mut BTreeSet<NodeId>,
        &mut BTreeSet<NodeId>,
    )> {
        match std::mem::replace(&mut self.dirt, Dirt::Clean) {
            Dirt::Clean => {
                self.dirt = Dirt::Metadata {
                    nodes: BTreeSet::new(),
                    subtrees: BTreeSet::new(),
                    removed: BTreeSet::new(),
                };
            }
            Dirt::LocalText { nodes } => {
                self.dirt = Dirt::Metadata {
                    nodes,
                    subtrees: BTreeSet::new(),
                    removed: BTreeSet::new(),
                };
            }
            other => self.dirt = other,
        }
        match &mut self.dirt {
            Dirt::Metadata {
                nodes,
                subtrees,
                removed,
            } => Some((nodes, subtrees, removed)),
            _ => None,
        }
    }
    #[cfg(feature = "testing")]
    pub(crate) fn region_of(&self, native: u64) -> Option<(NodeId, RegionId)> {
        self.native_regions.get(&native).copied()
    }
    #[cfg(feature = "testing")]
    pub(crate) fn region_id(&self, node: NodeId, region: RegionId) -> Option<u64> {
        self.region_ids.get(&(node, region)).copied()
    }
}

impl A11yState {
    /// A new native identity: no node of `tree`, no scene region, and
    /// never given before.
    pub(super) fn next_identity(&mut self, tree: &crate::tree::NodeTree) -> u64 {
        loop {
            let candidate = self.next_semantics_id;
            self.next_semantics_id = candidate.checked_sub(1).expect(
                "invariant: native semantics identity space cannot exhaust in a process lifetime",
            );
            // The counter only decreases, so this candidate cannot be an earlier synthetic ID.
            if !tree.contains(NodeId(candidate)) && !self.native_regions.contains_key(&candidate) {
                return candidate;
            }
        }
    }
}
