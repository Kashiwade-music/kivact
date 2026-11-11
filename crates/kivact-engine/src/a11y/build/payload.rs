//! Role: Build the payloads of payload-only dirt over the accepted identity metadata.
//!
//! Invariants:
//! - Work in proportion to the dirt: the nodes written, the subtrees that
//!   moved (each node once, outer roots first), the overlays placed by them
//!   (to a fixed point), the relation sources of all of these, their boxless
//!   ancestors, and the rows whose clipping changed in the clipping lists at
//!   or above the clip sources. No walk of the whole tree.
//! - The accepted identity metadata is an immutable Rc snapshot that the
//!   build only reads; a changed clipping goes back with the build and is
//!   committed on acceptance. A modal that changed without metadata dirt
//!   (none should) enumerates again, as a metadata build.
//! - Only nodes of the accepted tree (`Metadata::live`) are built: a node
//!   outside it (a leaving subtree's) is published by a metadata build.
//! - Nodes are built parents first (depth, then id), each with the scroll
//!   offsets above it, as the full builder computes them.
use super::dirty::{Built, MetadataUpdate};
use crate::a11y::state::{Clipping, Payloads};
use crate::model::Model;
use arvane as a;
use kivact_types::{NodeId, Point};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// A node to build: its depth and the scroll offsets above it.
type Target = (usize, Point);

impl Model {
    pub(super) fn build_payloads(
        &mut self,
        root: NodeId,
        dirt: &Payloads,
        engine: &mut a::Engine,
    ) -> Built {
        let metadata = Rc::clone(&self.a11y.metadata);
        if self.modal() != metadata.modal {
            let promoted = crate::a11y::state::Dirt::Metadata {
                nodes: dirt.nodes.clone(),
                subtrees: dirt.subtrees.clone(),
                removed: BTreeSet::new(),
            };
            return self.build_semantics_for(engine, &promoted);
        }
        let mut targets: BTreeMap<NodeId, Target> = BTreeMap::new();
        self.collect_subtrees(dirt.subtrees.iter().copied(), &mut targets);
        let mut sources = dirt.clip_sources.clone();
        // An overlay is placed by its anchor: one anchored at or under a
        // dirty or moved node moves with it (to a fixed point: overlays
        // anchored in other overlays).
        loop {
            let anchored: Vec<NodeId> = self
                .tree
                .overlays
                .iter()
                .copied()
                .filter(|o| !targets.contains_key(o))
                .filter(|o| {
                    self.node(*o)
                        .and_then(|n| n.overlay())
                        .and_then(|overlay| overlay.anchor())
                        .is_some_and(|anchor| {
                            self.tree()
                                .chain(Some(anchor))
                                .iter()
                                .any(|a| dirt.nodes.contains(a) || targets.contains_key(a))
                        })
                })
                .collect();
            if anchored.is_empty() {
                break;
            }
            sources.extend(anchored.iter().copied());
            self.collect_subtrees(anchored.into_iter(), &mut targets);
        }
        let mut dirty: BTreeSet<NodeId> = dirt.nodes.clone();
        dirty.extend(targets.keys().copied());
        let clipping = self.reclip(&metadata.clipping, &sources, &mut dirty);
        self.expand_relation_dirt(&metadata, &mut dirty);
        self.invalidate_boxless_ancestors(&mut dirty);
        dirty.retain(|id| metadata.live.contains(&id.0));
        let mut order: Vec<(usize, NodeId, Point)> = dirty
            .into_iter()
            .map(|id| match targets.get(&id) {
                Some(&(depth, offset)) => (depth, id, offset),
                None => (self.tree().depth(id), id, self.scroll_above(id)),
            })
            .collect();
        order.sort_unstable_by_key(|&(depth, id, _)| (depth, id));
        let clipped = match &clipping {
            Some(c) => c.rows(),
            None => metadata.clipping.rows(),
        };
        let mut nodes = Vec::new();
        let mut regions = BTreeSet::new();
        for (_, id, offset) in order {
            self.build_node(id, offset, clipped, &mut nodes, &mut regions, engine);
        }
        #[cfg(feature = "testing")]
        {
            self.a11y.constructed += nodes.len() as u64;
        }
        let focus = self.semantics_focus(clipped);
        Built {
            tree: kivact_types::render::SemanticsTree {
                root: kivact_types::SemanticsId(root.0),
                focus,
                nodes,
            },
            metadata: clipping.map_or(MetadataUpdate::Keep, MetadataUpdate::Clipping),
        }
    }

    /// Add every node under `roots` (each root included) with its depth and
    /// the scroll offsets above it; a root already added (inside an outer
    /// root, which goes first) is skipped.
    fn collect_subtrees(
        &self,
        roots: impl Iterator<Item = NodeId>,
        targets: &mut BTreeMap<NodeId, Target>,
    ) {
        let mut roots: Vec<(usize, NodeId)> = roots.map(|r| (self.tree().depth(r), r)).collect();
        roots.sort_unstable();
        for (depth, root) in roots {
            if targets.contains_key(&root) {
                continue;
            }
            let mut stack = vec![(root, depth, self.scroll_above(root))];
            while let Some((id, d, offset)) = stack.pop() {
                if targets.insert(id, (d, offset)).is_some() {
                    continue;
                }
                let Some(n) = self.node(id) else { continue };
                let s = n.scroll();
                let below = Point {
                    x: offset.x + s.x,
                    y: offset.y + s.y,
                };
                stack.extend(
                    self.semantic_children(id)
                        .into_iter()
                        .map(|c| (c, d + 1, below)),
                );
            }
        }
    }

    /// The clipping after the clipping lists at or above `sources` look at
    /// their rows again (`None`: no such list, or no row changed); the rows
    /// whose clipping changed go into `dirty`.
    fn reclip(
        &self,
        accepted: &Clipping,
        sources: &BTreeSet<NodeId>,
        dirty: &mut BTreeSet<NodeId>,
    ) -> Option<Clipping> {
        let mut lists = BTreeSet::new();
        let mut seen = BTreeSet::new();
        for &source in sources {
            let mut at = Some(source);
            while let Some(x) = at {
                if !seen.insert(x) {
                    break; // its ancestors were looked at
                }
                if self.is_clipping_list_node(x) || accepted.has_list(x) {
                    lists.insert(x);
                }
                at = self.tree().get(x).and_then(|n| self.tree().parent_id(n));
            }
        }
        if lists.is_empty() {
            return None;
        }
        let mut clipping = accepted.clone();
        let mut changed = false;
        for list in lists {
            let clipped = self.list_rows(list).map(|(_, c)| c).unwrap_or_default();
            let rows = clipping.set_list(list, clipped);
            changed |= !rows.is_empty();
            dirty.extend(rows);
        }
        changed.then_some(clipping)
    }
}
