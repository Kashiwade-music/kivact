//! Role: Accessibility identities, accepted fingerprints and pending invalidation.
//!
//! Payload ownership belongs exclusively to the shell. IDs may be allocated
//! before acceptance; fingerprints and pending dirt commit only after acceptance.
use kivact_wire::{NodeId, RegionId};
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

#[derive(Clone)]
pub(crate) struct A11yState {
    pub(super) native_regions: BTreeMap<u64, (NodeId, RegionId)>,
    pub(super) region_ids: BTreeMap<(NodeId, RegionId), u64>,
    pub(super) next_semantics_id: u64,
    pub(super) authored: BTreeMap<(NodeId, u64), u64>,
    pub(super) hashes: BTreeMap<u64, u64>,
    pub(super) root: Option<kivact_types::NodeId>,
    pub(super) empty_root: Option<u64>,
    pub(super) focus: Option<kivact_types::NodeId>,
    pub(super) pending: bool,
    pub(super) full: bool,
    /// Consumed by the builder: unclassified dirt requires current metadata.
    pub(super) metadata_refresh: bool,
    pub(super) nodes: BTreeSet<NodeId>,
    pub(super) subtrees: BTreeSet<NodeId>,
    pub(super) removed: BTreeSet<NodeId>,
    pub(super) metadata: Rc<Metadata>,
    pub(super) prepared: Option<Metadata>,
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
            pending: true,
            full: true,
            metadata_refresh: true,
            nodes: BTreeSet::new(),
            subtrees: BTreeSet::new(),
            removed: BTreeSet::new(),
            metadata: Rc::default(),
            prepared: None,
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
        self.pending
    }
    pub(crate) fn invalidate(&mut self) {
        self.pending = true;
        self.full = true;
        self.metadata_refresh = true;
    }
    pub(crate) fn local_text(&mut self, id: NodeId) {
        self.nodes.insert(id);
        self.pending = true;
    }
    pub(crate) fn node(&mut self, id: NodeId) {
        self.metadata_refresh = true;
        self.nodes.insert(id);
        self.pending = true;
    }
    pub(crate) fn subtree(&mut self, id: NodeId) {
        self.metadata_refresh = true;
        self.subtrees.insert(id);
        self.pending = true;
    }
    pub(crate) fn request_publication(&mut self, removed: impl Iterator<Item = NodeId>) {
        self.metadata_refresh = true;
        self.pending = true;
        self.removed.extend(removed);
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
