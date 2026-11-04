//! Role: Expand dirt using current identity, child-order, clipping and relation metadata.
//!
//! The O(N) enumeration stores IDs/hashes only. Payload construction remains
//! proportional to the expanded dirt; no accepted payload is retained here.
use super::node::{drawable, structural};
use crate::a11y::state::{Dirt, Metadata};
use crate::model::Model;
use crate::semantic_role;
use arvane as a;
use kivact_types as types;
use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};

/// A semantics tree built for publication and the identity metadata it was
/// built with (`None`: a local text build reused the accepted metadata).
pub(in crate::a11y) struct Built {
    pub(in crate::a11y) tree: kivact_types::render::SemanticsTree,
    pub(in crate::a11y) metadata: Option<Metadata>,
}

impl Model {
    /// Independent full reference; callers isolate mutable identities and caches.
    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) fn build_semantics(
        &mut self,
        engine: &mut a::Engine,
    ) -> kivact_types::render::SemanticsTree {
        self.build_semantics_for(engine, &Dirt::Full).tree
    }

    /// Build the payloads `dirt` asks for, with the identity metadata the
    /// build used (`None`: a local text build reused the accepted one).
    pub(in crate::a11y) fn build_semantics_for(
        &mut self,
        engine: &mut a::Engine,
        dirt: &Dirt,
    ) -> Built {
        let full = *dirt == Dirt::Full;
        let Some(root) = self.tree().root() else {
            let id = match self.a11y.publish {
                crate::a11y::state::A11yPublish::Published { root }
                    if !self.tree.contains(types::NodeId(root.0)) =>
                {
                    root.0
                }
                _ => self.next_semantics_identity(),
            };
            let node = structural(
                types::NodeId(id),
                Vec::new(),
                kivact_types::Rect {
                    x: 0.0,
                    y: 0.0,
                    width: self.viewport.width,
                    height: self.viewport.height,
                },
                kivact_types::Point::default(),
                kivact_types::Transform::IDENTITY,
                false,
            );
            #[cfg(feature = "testing")]
            {
                self.a11y.constructed += 1;
            }
            return Built {
                tree: kivact_types::render::SemanticsTree {
                    root: kivact_types::SemanticsId(id),
                    focus: None,
                    nodes: vec![node],
                },
                metadata: Some(Metadata {
                    live: BTreeSet::from([id]),
                    ..Default::default()
                }),
            };
        };
        if let Dirt::LocalText { nodes } = dirt {
            return Built {
                tree: self.build_local_text(root, nodes, engine),
                metadata: None,
            };
        }
        let empty = BTreeSet::new();
        let (own, subtrees, removed) = match dirt {
            Dirt::Metadata {
                nodes,
                subtrees,
                removed,
            } => (nodes, subtrees, removed),
            _ => (&empty, &empty, &empty),
        };
        self.prepare_region_ids();
        let clipped = self.clipped_rows();
        let mut metadata = Metadata {
            clipped,
            modal: self.modal(),
            ..Default::default()
        };
        let order = self.tree().flat_preorder();
        for &id in &order {
            let Some(n) = self.node(id) else { continue };
            metadata.parents.insert(id, self.tree().flat_parent(n));
            let mut children = DefaultHasher::new();
            let mut count = 0usize;
            for child in self.tree().flat_children(self.tree().nodes.id_at(n.index)) {
                child.hash(&mut children);
                count += 1;
            }
            count.hash(&mut children);
            metadata.children.insert(id, children.finish());
            metadata.live.insert(id.0);
            let props = n.semantics();
            let region_props = n
                .scenes()
                .iter()
                .flatten()
                .flat_map(|s| s.regions.iter())
                .map(|r| &r.semantics);
            for props in props.into_iter().chain(region_props) {
                for target in props
                    .described_by
                    .iter()
                    .chain(&props.controls)
                    .chain(&props.labelled_by)
                    .chain(props.active_descendant.iter())
                {
                    metadata.reverse.entry(target.node).or_default().insert(id);
                }
            }
            if semantic_role(n, self.editors[n.index.0].is_some()).is_some() {
                for r in n
                    .scenes()
                    .iter()
                    .flatten()
                    .flat_map(|s| s.regions.iter())
                    .filter(|r| drawable(r.rect))
                {
                    metadata.regions.insert((id, r.id));
                    if let Some(native) = self.a11y.region_ids.get(&(id, r.id)) {
                        metadata.live.insert(*native);
                    }
                }
                if let Some(text) = props.and_then(|p| p.text.as_ref()) {
                    metadata
                        .authored
                        .extend(text.runs.iter().map(|r| (id, r.id.0)));
                }
            }
        }
        let mut dirty = own.clone();
        dirty.extend(removed.iter().copied());
        dirty.extend(
            metadata
                .clipped
                .symmetric_difference(&self.a11y.metadata.clipped)
                .copied(),
        );
        for &id in &order {
            if metadata.children.get(&id) != self.a11y.metadata.children.get(&id) {
                dirty.insert(id);
            }
            if metadata.parents.get(&id) != self.a11y.metadata.parents.get(&id) {
                dirty.insert(id);
                if let Some(Some(p)) = metadata.parents.get(&id) {
                    dirty.insert(*p);
                }
            }
        }
        let modal_changed = metadata.modal != self.a11y.metadata.modal;
        let mut inherited = BTreeSet::new();
        for &id in &order {
            let parent_dirty = metadata
                .parents
                .get(&id)
                .and_then(|p| *p)
                .is_some_and(|p| inherited.contains(&p));
            if modal_changed || parent_dirty || subtrees.contains(&id) {
                inherited.insert(id);
                dirty.insert(id);
            }
        }
        // Existing overlay placement authority also follows anchors outside a subtree.
        // Follow those existing anchor/ancestor dependencies to a fixed point,
        // including overlays anchored in other overlays; do not retain a placement graph.
        loop {
            let mut expanded = false;
            for &overlay in &self.tree.overlays {
                if let Some(anchor) = self
                    .node(overlay)
                    .and_then(|n| n.overlay())
                    .and_then(|o| o.anchor())
                    && self
                        .tree()
                        .chain(Some(anchor))
                        .iter()
                        .any(|a| inherited.contains(a) || dirty.contains(a))
                {
                    for &id in &order {
                        if self.tree().is_in_subtree(id, overlay) && dirty.insert(id) {
                            expanded = true;
                        }
                    }
                }
            }
            if !expanded {
                break;
            }
        }
        // Both old and current reverse edges matter for removal and relation replacement.
        let mut queue: Vec<_> = dirty.iter().copied().collect();
        while let Some(target) = queue.pop() {
            for source in self
                .a11y
                .metadata
                .reverse
                .get(&target)
                .into_iter()
                .flatten()
                .chain(metadata.reverse.get(&target).into_iter().flatten())
            {
                if dirty.insert(*source) {
                    queue.push(*source);
                }
            }
        }
        let mut nodes = Vec::new();
        let mut active = BTreeSet::new();
        let mut stack = vec![(root, types::Point::default())];
        while let Some((id, offset)) = stack.pop() {
            if full || dirty.contains(&id) {
                self.build_node(
                    id,
                    offset,
                    &metadata.clipped,
                    &mut nodes,
                    &mut active,
                    engine,
                );
            }
            if let Some(n) = self.node(id) {
                let s = n.scroll();
                let below = types::Point {
                    x: offset.x + s.x,
                    y: offset.y + s.y,
                };
                stack.extend(
                    self.tree()
                        .flat_children(self.tree().nodes.id_at(n.index))
                        .rev()
                        .map(|child| (child, below)),
                );
            }
        }
        #[cfg(feature = "testing")]
        {
            self.a11y.constructed += nodes.len() as u64;
        }
        let focus = self.semantics_focus(&metadata.clipped);
        Built {
            tree: kivact_types::render::SemanticsTree {
                root: kivact_types::SemanticsId(root.0),
                focus,
                nodes,
            },
            metadata: Some(metadata),
        }
    }
}
