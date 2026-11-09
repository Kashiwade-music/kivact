//! Role: Accessibility identities, accepted fingerprints and pending invalidation.
//!
//! Payload ownership belongs exclusively to the shell. IDs may be allocated
//! before acceptance; fingerprints and pending dirt commit only after acceptance.
use kivact_types::{NodeId, RegionId, SemanticsId};
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
    /// Only geometry changed since the last accepted tree: the payloads of
    /// `nodes` and of every node under `subtrees` are built again; the
    /// accepted identity metadata is reused when the clipped rows are the same.
    Geometry {
        nodes: BTreeSet<NodeId>,
        subtrees: BTreeSet<NodeId>,
    },
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum A11yPublish {
    Inactive,
    AwaitingFirstTree,
    Published { root: SemanticsId },
}

#[derive(Clone)]
pub(crate) struct A11yState {
    pub(super) native_regions: BTreeMap<u64, (NodeId, RegionId)>,
    pub(super) region_ids: BTreeMap<(NodeId, RegionId), u64>,
    pub(super) ids: SemanticsIdAllocator,
    pub(super) authored: BTreeMap<(NodeId, u64), u64>,
    pub(super) hashes: BTreeMap<u64, u64>,
    pub(crate) publish: A11yPublish,
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
            ids: SemanticsIdAllocator {
                next: u64::MAX,
                empty_root: None,
            },
            authored: BTreeMap::new(),
            hashes: BTreeMap::new(),
            publish: A11yPublish::Inactive,
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
            Dirt::LocalText { nodes }
            | Dirt::Geometry { nodes, .. }
            | Dirt::Metadata { nodes, .. } => {
                nodes.insert(id);
            }
            Dirt::Full => {}
        }
    }
    /// Note that `id` moved with its subtree: only geometry changed.
    pub(crate) fn moved(&mut self, id: NodeId) {
        let dirt = std::mem::replace(&mut self.dirt, Dirt::Clean);
        self.dirt = match dirt {
            Dirt::Clean => Dirt::Geometry {
                nodes: BTreeSet::new(),
                subtrees: BTreeSet::from([id]),
            },
            Dirt::LocalText { nodes } => Dirt::Geometry {
                nodes,
                subtrees: BTreeSet::from([id]),
            },
            Dirt::Geometry {
                nodes,
                mut subtrees,
            } => {
                subtrees.insert(id);
                Dirt::Geometry { nodes, subtrees }
            }
            Dirt::Metadata {
                nodes,
                mut subtrees,
                removed,
            } => {
                subtrees.insert(id);
                Dirt::Metadata {
                    nodes,
                    subtrees,
                    removed,
                }
            }
            Dirt::Full => Dirt::Full,
        };
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
            Dirt::Geometry { nodes, subtrees } => {
                self.dirt = Dirt::Metadata {
                    nodes,
                    subtrees,
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

/// Monotonic synthetic identities; consumers reject live authored node IDs.
#[derive(Clone)]
pub(super) struct SemanticsIdAllocator {
    // xtask-allow(state-fields): S-8 ID - the next synthetic semantics id, counting down
    next: u64,
    // xtask-allow(state-fields): S-8 ID - the id reserved for the empty root, independent of the published tree
    /// Reserved across authored roots, independently of the published tree.
    pub(super) empty_root: Option<SemanticsId>,
}
impl SemanticsIdAllocator {
    pub(super) fn next(&mut self) -> SemanticsId {
        let id = self.next;
        self.next = id
            .checked_sub(1)
            .expect("invariant: semantics identity space cannot exhaust in a process lifetime");
        SemanticsId(id)
    }

    /// The next identity that no node of `tree` has.
    pub(super) fn fresh(&mut self, tree: &crate::tree::NodeTree) -> u64 {
        loop {
            let id = self.next().0;
            if !tree.contains(kivact_types::NodeId(id)) {
                return id;
            }
        }
    }
}
