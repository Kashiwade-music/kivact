//! Role: Accessibility identities, accepted fingerprints and pending invalidation.
//!
//! Payload ownership belongs exclusively to the shell. IDs may be allocated
//! before acceptance; fingerprints and pending dirt commit only after acceptance.
//! Pending dirt exists only while a tree is published: before the first tree
//! and while no assistive technology reads the window, the next publication is
//! a complete tree, so nothing is kept pending.
use kivact_types::{NodeId, RegionId, SemanticsId};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// The rows of the clipping list-like containers that lie clipped out of
/// them (hidden from assistive technology), by list. The rows of two lists
/// never meet: a list's rows end at the first node with a role, and a list
/// has one.
#[derive(Clone, Default, PartialEq)]
pub(super) struct Clipping {
    by_list: BTreeMap<NodeId, BTreeSet<NodeId>>,
    /// Projection: the union of `by_list`, written only by [`Clipping::set_list`].
    rows: BTreeSet<NodeId>,
}

impl Clipping {
    /// Every clipped row.
    pub(super) fn rows(&self) -> &BTreeSet<NodeId> {
        &self.rows
    }

    /// Whether `list` clips rows out.
    pub(super) fn has_list(&self, list: NodeId) -> bool {
        self.by_list.contains_key(&list)
    }

    /// Replace the clipped rows of `list` (empty: it clips none); the rows
    /// whose clipping changed.
    pub(super) fn set_list(&mut self, list: NodeId, clipped: BTreeSet<NodeId>) -> Vec<NodeId> {
        let old = if clipped.is_empty() {
            self.by_list.remove(&list)
        } else {
            self.by_list.insert(list, clipped.clone())
        }
        .unwrap_or_default();
        for row in old.difference(&clipped) {
            self.rows.remove(row);
        }
        self.rows.extend(clipped.difference(&old).copied());
        old.symmetric_difference(&clipped).copied().collect()
    }
}

#[derive(Clone, Default)]
pub(super) struct Metadata {
    pub clipping: Clipping,
    pub parents: BTreeMap<NodeId, Option<NodeId>>,
    pub children: BTreeMap<NodeId, u64>,
    pub reverse: BTreeMap<NodeId, BTreeSet<NodeId>>,
    pub regions: BTreeSet<(NodeId, RegionId)>,
    pub authored: BTreeSet<(NodeId, u64)>,
    pub live: BTreeSet<u64>,
    pub modal: Option<NodeId>,
}

/// The payloads to build again over the accepted identity metadata.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Payloads {
    /// Nodes whose own payload changed (a text, a paint, state styles, a
    /// scroll offset, an image).
    pub(in crate::a11y) nodes: BTreeSet<NodeId>,
    /// Nodes that moved with their subtrees: every payload under them is
    /// built again (bounds are content-space and absolute).
    pub(in crate::a11y) subtrees: BTreeSet<NodeId>,
    /// Nodes that moved, scrolled, took or lost the focus or changed their
    /// clip: the clipping lists at or above them look at their rows again.
    pub(in crate::a11y) clip_sources: BTreeSet<NodeId>,
}

/// What changed since the accepted tree.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Dirt {
    /// Nothing.
    Clean,
    /// Only payloads: the accepted identity metadata (hierarchy, relations,
    /// regions, authored runs, modal) is reused; work in proportion to the
    /// payloads.
    Payload(Payloads),
    /// The identity and order metadata is enumerated again; besides what it
    /// finds, `nodes` (own payloads), `subtrees` (whole subtrees) and
    /// `removed` are dirty.
    Metadata {
        nodes: BTreeSet<NodeId>,
        subtrees: BTreeSet<NodeId>,
        removed: BTreeSet<NodeId>,
    },
    /// A complete replacement (a new root, a rejected publication, an
    /// environment change).
    Full,
}

/// Whether and what the window's platform tree holds.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum A11yPublish {
    /// No assistive technology reads the window: nothing is built or pending.
    Inactive,
    /// Activated: the next publication is a complete tree.
    AwaitingFirstTree,
    /// The platform holds the accepted tree of `root`; `dirt` is what
    /// changed since.
    Published { root: SemanticsId, dirt: Dirt },
}

#[derive(Clone)]
pub(crate) struct A11yState {
    pub(super) native_regions: BTreeMap<u64, (NodeId, RegionId)>,
    pub(super) region_ids: BTreeMap<(NodeId, RegionId), u64>,
    pub(super) ids: SemanticsIdAllocator,
    pub(super) authored: BTreeMap<(NodeId, u64), u64>,
    pub(super) hashes: BTreeMap<u64, u64>,
    pub(super) publish: A11yPublish,
    pub(super) metadata: Rc<Metadata>,
    #[cfg(feature = "testing")]
    pub(crate) constructed: u64,
    #[cfg(feature = "testing")]
    pub(crate) published: u64,
    /// Nodes the identity enumeration visited (one per node per Metadata or
    /// Full build).
    #[cfg(feature = "testing")]
    pub(crate) enumerated: u64,
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
            metadata: Rc::default(),
            #[cfg(feature = "testing")]
            constructed: 0,
            #[cfg(feature = "testing")]
            published: 0,
            #[cfg(feature = "testing")]
            enumerated: 0,
        }
    }
}
impl A11yState {
    /// Assistive technology started reading the window; whether it was not
    /// (the next publication is then a complete tree).
    pub(crate) fn activate(&mut self) -> bool {
        if self.publish != A11yPublish::Inactive {
            return false;
        }
        self.publish = A11yPublish::AwaitingFirstTree;
        true
    }

    /// Assistive technology stopped reading the window: nothing is kept
    /// pending or accepted (an activation hands over a complete tree).
    pub(crate) fn deactivate(&mut self) {
        self.publish = A11yPublish::Inactive;
        self.hashes.clear();
        self.metadata = Rc::default();
    }

    /// The next publication is a complete tree.
    pub(crate) fn invalidate(&mut self) {
        if let A11yPublish::Published { dirt, .. } = &mut self.publish {
            *dirt = Dirt::Full;
        }
    }

    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) fn publication_pending(&self) -> bool {
        !matches!(
            self.publish,
            A11yPublish::Published {
                dirt: Dirt::Clean,
                ..
            }
        )
    }

    /// The pending dirt (`None`: a complete tree is pending, or none is
    /// published), raised from `Clean` to an empty payload dirt.
    fn pending(&mut self) -> Option<&mut Dirt> {
        let A11yPublish::Published { dirt, .. } = &mut self.publish else {
            return None;
        };
        if *dirt == Dirt::Clean {
            *dirt = Dirt::Payload(Payloads::default());
        }
        if *dirt == Dirt::Full {
            return None;
        }
        Some(dirt)
    }

    /// Only the payload of `id` changed: a text, a paint, state styles, an
    /// image, a placeholder (not its role, relations or regions).
    pub(crate) fn payload(&mut self, id: NodeId) {
        match self.pending() {
            Some(Dirt::Payload(p)) => {
                p.nodes.insert(id);
            }
            Some(Dirt::Metadata { nodes, .. }) => {
                nodes.insert(id);
            }
            _ => {}
        }
    }

    /// `id` moved with its subtree (layout, a transform, a sticky shift, a
    /// layout or state motion, a drawable's geometry).
    pub(crate) fn moved(&mut self, id: NodeId) {
        match self.pending() {
            Some(Dirt::Payload(p)) => {
                p.subtrees.insert(id);
                p.clip_sources.insert(id);
            }
            Some(Dirt::Metadata { subtrees, .. }) => {
                subtrees.insert(id);
            }
            _ => {}
        }
    }

    /// The scroll offset of `id` changed: its own payload (its offset) and
    /// the clipping lists at or above it; `Model::a11y_scrolled` adds the
    /// layers under it.
    pub(crate) fn scrolled(&mut self, id: NodeId) {
        match self.pending() {
            Some(Dirt::Payload(p)) => {
                p.nodes.insert(id);
                p.clip_sources.insert(id);
            }
            Some(Dirt::Metadata { nodes, .. }) => {
                nodes.insert(id);
            }
            _ => {}
        }
    }

    /// The clip of `id` may have changed (its paint's `clip`): the clipping
    /// lists at or above it look at their rows again.
    pub(crate) fn reclip(&mut self, id: NodeId) {
        if let Some(Dirt::Payload(p)) = self.pending() {
            p.clip_sources.insert(id);
        }
    }

    /// The focus moved from `old` to `new`: the focus is published, and the
    /// clipping lists around both look again (a focused row stays exposed).
    pub(crate) fn focus_moved(&mut self, old: Option<NodeId>, new: Option<NodeId>) {
        if let Some(Dirt::Payload(p)) = self.pending() {
            p.clip_sources.extend(old);
            p.clip_sources.extend(new);
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

    /// Raise the pending dirt to `Metadata` (keeping a payload dirt's nodes
    /// and subtrees; its clip sources are dropped: the metadata build looks
    /// at every clipping list); its sets, or `None` when a complete tree is
    /// pending or none is published.
    fn metadata_dirt(
        &mut self,
    ) -> Option<(
        &mut BTreeSet<NodeId>,
        &mut BTreeSet<NodeId>,
        &mut BTreeSet<NodeId>,
    )> {
        let A11yPublish::Published { dirt, .. } = &mut self.publish else {
            return None;
        };
        match std::mem::replace(dirt, Dirt::Clean) {
            Dirt::Clean => {
                *dirt = Dirt::Metadata {
                    nodes: BTreeSet::new(),
                    subtrees: BTreeSet::new(),
                    removed: BTreeSet::new(),
                }
            }
            Dirt::Payload(Payloads {
                nodes, subtrees, ..
            }) => {
                *dirt = Dirt::Metadata {
                    nodes,
                    subtrees,
                    removed: BTreeSet::new(),
                }
            }
            other => *dirt = other,
        }
        match dirt {
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
    // xtask-allow(state-fields): identity: the next synthetic semantics id, counting down
    next: u64,
    // xtask-allow(state-fields): identity: the id reserved for the empty root, independent of the published tree
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
