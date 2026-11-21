//! Role: Derive the layer facts of the subtrees from the looks the walk uses.
//!
//! The facts come from one bottom-up pass before the walk, from the looks
//! the walk uses (`layer`): whether a node is a layer, and what its children
//! hold. The paint list and the hit test both read these facts, so a 3D
//! context has the same planes when drawn and when hit.

use crate::layer::{LayerTraits, SubtreeFacts};
use crate::model::Model;
use kivact_types::{NodeId, WillChange};
use std::collections::HashMap;

impl Model {
    /// The facts of every drawn subtree of the layer roots, see [`layer_facts`].
    pub(crate) fn layer_facts(&self) -> HashMap<NodeId, SubtreeFacts> {
        layer_facts(self)
    }

    /// The facts of the subtree at `id` (its parent's look is `parent`),
    /// recorded in `out`, see [`subtree_facts`].
    pub(crate) fn subtree_facts(
        &self,
        id: NodeId,
        parent: Option<&LayerTraits>,
        out: &mut HashMap<NodeId, SubtreeFacts>,
    ) -> SubtreeFacts {
        subtree_facts(self, id, parent, out)
    }
}

/// The facts of every drawn subtree of the layer roots (the tree and the
/// overlays and fixed boxes); only the nodes with something to say are kept.
pub(crate) fn layer_facts(host: &Model) -> HashMap<NodeId, SubtreeFacts> {
    let mut out = HashMap::new();
    for root in host.layer_roots() {
        subtree_facts(host, root, None, &mut out);
    }
    out
}

/// The facts of the subtree at `id` (its parent's look is `parent`),
/// recorded in `out` for the nodes that have any.
pub(crate) fn subtree_facts(
    host: &Model,
    id: NodeId,
    parent: Option<&LayerTraits>,
    out: &mut HashMap<NodeId, SubtreeFacts>,
) -> SubtreeFacts {
    let Some(look) = host.drawn_look(id) else {
        return SubtreeFacts::default();
    };
    let mut below = SubtreeFacts::default();
    for child in host.tree().flat_children(id) {
        if host.is_layer_node(child) {
            continue;
        }
        let f = subtree_facts(host, child, Some(&look), out);
        below = merge_below(below, f);
    }
    let (scene_layer, scene_item) = host.scene_facts(id);
    // Its own background shader is an item inside its group too.
    below = below
        .with(SubtreeFacts::HAS_LAYER, scene_layer)
        .with(SubtreeFacts::HAS_ITEM, scene_item || look.shader());
    let layer = own_layer(host, id, &look, parent) || promoted(&look, &below);
    let facts = own_facts(&look, &below, layer);
    if facts != SubtreeFacts::default() {
        out.insert(id, facts);
    }
    facts
}

/// The facts of `id` from its children's recorded facts (`facts`), given its parent's look.
pub(crate) fn node_facts(
    host: &Model,
    id: NodeId,
    parent: Option<&LayerTraits>,
    facts: &HashMap<NodeId, SubtreeFacts>,
) -> SubtreeFacts {
    let Some(look) = host.drawn_look(id) else {
        return SubtreeFacts::default();
    };
    let mut below = SubtreeFacts::default();
    for child in host.tree().flat_children(id) {
        if host.is_layer_node(child) {
            continue;
        }
        let f = facts.get(&child).copied().unwrap_or_default();
        below = merge_below(below, f);
    }
    let (scene_layer, scene_item) = host.scene_facts(id);
    // Its own background shader is an item inside its group too.
    below = below
        .with(SubtreeFacts::HAS_LAYER, scene_layer)
        .with(SubtreeFacts::HAS_ITEM, scene_item || look.shader());
    let layer = own_layer(host, id, &look, parent) || promoted(&look, &below);
    own_facts(&look, &below, layer)
}

/// The children's facts `f` added to the facts of the children `below` so far
/// (every fact but the node's own layer).
fn merge_below(below: SubtreeFacts, f: SubtreeFacts) -> SubtreeFacts {
    below
        .with(SubtreeFacts::HAS_LAYER, f.has_layer())
        .with(SubtreeFacts::HAS_ITEM, f.has_item())
        .with(SubtreeFacts::HAS_BACKDROP, f.has_backdrop())
        .with(SubtreeFacts::HAS_BLEND, f.has_blend())
}

/// The facts of a node with `look`, whose layer is `layer` and whose
/// children hold `below`.
fn own_facts(look: &LayerTraits, below: &SubtreeFacts, layer: bool) -> SubtreeFacts {
    SubtreeFacts::default()
        .with(SubtreeFacts::LAYER, layer)
        .with(SubtreeFacts::HAS_LAYER, layer || below.has_layer())
        .with(SubtreeFacts::HAS_ITEM, look.shader() || below.has_item())
        .with(
            SubtreeFacts::HAS_BACKDROP,
            look.backdropped() || below.has_backdrop(),
        )
        .with(SubtreeFacts::HAS_BLEND, look.blended() || below.has_blend())
}

/// Whether `id` is a layer for its own reasons, given its parent's look: an
/// overlay or fixed box, a filter, a mask or a backdrop filter, a `will-change`,
/// a motion, or a part of a 3D rendering context.
fn own_layer(host: &Model, id: NodeId, look: &LayerTraits, parent: Option<&LayerTraits>) -> bool {
    let parent_plane_3d = parent.is_some_and(LayerTraits::plane_3d);
    let parent_perspective = parent.is_some_and(|p| p.perspective());
    let overlay = host.is_layer_node(id);
    let moves = host.moves_layer(id);
    overlay
        || look.filtered()
        || look.masked()
        || look.backdropped()
        || look.will_change != WillChange::Auto
        || moves
        || !look.flat()
        || (look.transformed() && parent_perspective)
        || (parent_plane_3d && (!look.flat() || look.preserves()))
        || look.plane_3d()
}

/// Whether the node with `look` is promoted to a layer by what its inner
/// subtree holds: a group over a layer or an item, a backdrop root over a
/// backdrop filter, or an isolated node over a blend mode and a layer.
fn promoted(look: &LayerTraits, inner: &SubtreeFacts) -> bool {
    let group = look.opacity < 1.0 || look.grouped();
    let backdrop_root = look.opacity < 1.0
        || look.filtered()
        || look.masked()
        || look.grouped()
        || look.backdropped();
    (group && (inner.has_layer() || inner.has_item()))
        || (backdrop_root && inner.has_backdrop())
        || (look.isolated() && inner.has_blend() && inner.has_layer())
}
