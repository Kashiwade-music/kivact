//! Role: Which nodes are compositor layers (§6.1), and what each subtree
//! holds that decides it.
//!
//! Invariants:
//! - A node is a layer for its own reasons: it is a 3D or a flat preserve-3d
//!   member, it has a filter, a mask or a backdrop filter, its `will-change`
//!   is set, it moves (a running transition or a layout motion), or it is an
//!   overlay or fixed box.
//! - An inline group that holds a layer or an item is a layer; so is a
//!   backdrop root that holds a backdrop filter, and an isolated node that
//!   holds a blend mode and a layer. The inline groups it replaces never
//!   span an item (§6.3).
//! - The facts come from one bottom-up pass before the walk, from the looks
//!   the walk uses, so no node's layer depends on its layout location.

use crate::lower::scene::scene_layers;
use crate::model::Model;
use crate::paint::state::{Look, SubtreeFacts};
use kivact_types::{NodeId, WillChange};
use std::collections::HashMap;

impl Model {
    /// The facts of every drawn subtree of the tree and of the layer roots
    /// (overlays and fixed boxes); only the nodes with something to say are
    /// kept.
    pub(super) fn layer_facts(&self) -> HashMap<NodeId, SubtreeFacts> {
        let mut out = HashMap::new();
        if let Some(root) = self.tree().root() {
            self.subtree_facts(root, None, &mut out);
        }
        for (layer, _) in self.layers() {
            self.subtree_facts(layer, None, &mut out);
        }
        out
    }

    /// The facts of the subtree at `id` (its parent's look is `parent`),
    /// recorded in `out` for the nodes that have any.
    pub(super) fn subtree_facts(
        &self,
        id: NodeId,
        parent: Option<&Look>,
        out: &mut HashMap<NodeId, SubtreeFacts>,
    ) -> SubtreeFacts {
        let Some(n) = self.node(id) else {
            return SubtreeFacts::default();
        };
        if !self.tree().drawn(id) {
            return SubtreeFacts::default();
        }
        let look = Look::now(self, id, n);
        let mut below = SubtreeFacts::default();
        for child in self.tree().flat_children(id) {
            if self.node(child).is_some_and(|c| c.is_layer()) {
                continue;
            }
            let f = self.subtree_facts(child, Some(&look), out);
            below.has_layer |= f.has_layer;
            below.has_item |= f.has_item;
            below.has_backdrop |= f.has_backdrop;
            below.has_blend |= f.has_blend;
        }
        let (scene_layer, scene_item) =
            n.scenes()
                .iter()
                .flatten()
                .fold((false, false), |(layer, item), scene| {
                    let (l, i) = scene_layers(scene);
                    (layer || l, item || i)
                });
        below.has_layer |= scene_layer;
        // Its own background shader is an item inside its group too.
        below.has_item |= scene_item || look.shader;
        let layer = self.own_layer(id, &look, parent) || promoted(&look, &below);
        let facts = SubtreeFacts {
            layer,
            has_layer: layer || below.has_layer,
            has_item: look.shader || below.has_item,
            has_backdrop: look.backdropped || below.has_backdrop,
            has_blend: look.blended || below.has_blend,
        };
        if facts != SubtreeFacts::default() {
            out.insert(id, facts);
        }
        facts
    }

    /// Whether `id` is a layer for its own reasons (R1 to R5 and R7 of
    /// §6.1), given its parent's look.
    fn own_layer(&self, id: NodeId, look: &Look, parent: Option<&Look>) -> bool {
        let parent_plane_3d = parent.is_some_and(Look::plane_3d);
        let parent_perspective = parent.is_some_and(|p| p.perspective);
        let overlay = self.node(id).is_some_and(|n| n.is_layer());
        let moves = self.motion.moves_layer(id) || self.layout_motion_transform(id).is_some();
        overlay
            || look.filtered
            || look.masked
            || look.backdropped
            || look.will_change != WillChange::Auto
            || moves
            || !look.flat
            || (look.transformed && parent_perspective)
            || (parent_plane_3d && (!look.flat || look.preserves))
            || look.plane_3d()
    }
}

/// Whether the node with `look` is promoted to a layer by what its inner
/// subtree holds (P1 to P3 of §6.1).
fn promoted(look: &Look, inner: &SubtreeFacts) -> bool {
    let group = look.opacity < 1.0 || look.grouped;
    let backdrop_root =
        look.opacity < 1.0 || look.filtered || look.masked || look.grouped || look.backdropped;
    (group && (inner.has_layer || inner.has_item))
        || (backdrop_root && inner.has_backdrop)
        || (look.isolated && inner.has_blend && inner.has_layer)
}
