//! Role: Compositor layers (§6.1): a node's look and the facts that decide them.
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
//! - The paint list and the hit test read the same facts, so a 3D context
//!   has the same planes when drawn and when hit.
//! - This module does not name the model: the tree is read through
//!   [`LayerView`], which the model implements.

use crate::lower::effects;
use crate::lower::sanitize::{extent, offset, unit};
use kivact_types::paint::{BackfaceVisibility, Fill, FillImage, TransformStyle};
use kivact_types::pixel::BlendMode;
use kivact_types::{Edges, ElementTransform, FilterOp, Isolation, NodeId, PaintProps, WillChange};
use std::collections::HashMap;

/// The extent of what reaches everywhere (finite, so unions keep it).
pub(crate) const EVERYWHERE: f64 = f64::MAX / 4.0;

/// The tree facts the layer classification reads. The model implements it;
/// the classification takes it as a parameter, so it names no model type.
pub(crate) trait LayerView {
    /// The look of `id` now, or `None` for a node that is missing or not drawn.
    fn drawn_look(&self, id: NodeId) -> Option<Look>;
    /// The flat children of `id`, in paint order.
    fn flat_children(&self, id: NodeId) -> Vec<NodeId>;
    /// Whether `id` is a layer of its own: an overlay or a fixed box.
    fn is_layer_node(&self, id: NodeId) -> bool;
    /// Whether the scenes of `id` hold a layer, and whether they hold an item.
    fn scene_facts(&self, id: NodeId) -> (bool, bool);
    /// Whether `id` moves as a layer: a running transition or a layout motion.
    fn moves_layer(&self, id: NodeId) -> bool;
    /// The roots of the layer trees: the tree's root, then the layers
    /// (overlays and fixed boxes) in the order the window draws them.
    fn layer_roots(&self) -> Vec<NodeId>;
}

// xtask-allow(state-fields): Normalized paint cache stores independent optional color and geometric drawing facts (S-9).
/// What a node's paint says about where it draws, sanitized like
/// `paint::sanitize_paint` (read in place: most nodes have no state
/// styles, and their paint is not copied).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Look {
    pub(crate) opacity: f64,
    /// How far its outer shadows and focus ring reach outside its box.
    pub(crate) overflow: f64,
    /// The widths of its visible border sides (canvas scenes start inside
    /// them).
    pub(crate) border: Edges<f64>,
    /// How far its layer filters draw beyond what they filter.
    pub(crate) fx: f64,
    /// Its effects need a group (clip path, blend), whatever its opacity.
    pub(crate) grouped: bool,
    /// It clips its children to its padding box.
    pub(crate) clip: bool,
    /// How many of a background, a border, shadows and a ring it draws.
    pub(crate) items: u8,
    /// Its corners are rounded.
    pub(crate) rounded: bool,
    /// It has a CSS filter, masks or a backdrop filter (each needs a layer).
    pub(crate) filtered: bool,
    pub(crate) masked: bool,
    pub(crate) backdropped: bool,
    /// Its blend mode is not normal.
    pub(crate) blended: bool,
    /// It isolates its group (`isolation: isolate`).
    pub(crate) isolated: bool,
    /// Its `will-change` value.
    pub(crate) will_change: WillChange,
    /// Its `transform-style` is `preserve-3d`.
    pub(crate) preserves: bool,
    /// Its element transform has no z component and no perspective.
    pub(crate) flat: bool,
    /// Its element transform is not the identity.
    pub(crate) transformed: bool,
    /// Its children are in a perspective.
    pub(crate) perspective: bool,
    /// Its back face is hidden.
    pub(crate) backface_hidden: bool,
    /// One of its background layers is a shader.
    pub(crate) shader: bool,
}

// xtask-allow(state-fields): A pure derived record of independent presence facts about one subtree, rebuilt with the paint list and never a lifecycle (S-9).
/// What a subtree holds that decides its layers (§6.1): whether the node
/// itself is a layer, and what its subtree contains.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SubtreeFacts {
    /// The node is a compositor layer.
    pub(crate) layer: bool,
    /// A layer is in the subtree (the node's own included).
    pub(crate) has_layer: bool,
    /// An external frame or a shader is in the subtree.
    pub(crate) has_item: bool,
    /// A backdrop filter is in the subtree.
    pub(crate) has_backdrop: bool,
    /// A non-normal blend mode is in the subtree.
    pub(crate) has_blend: bool,
}

impl Look {
    /// The look of the paint `p`.
    pub(crate) fn of(p: &PaintProps) -> Self {
        let shadows = p
            .shadows
            .iter()
            .filter(|s| !s.inset)
            .map(|s| {
                // The blur's gaussian (standard deviation blur / 2) is
                // drawn at most kivact_types::render::SHADOW_REACH deviations out.
                let blur = extent(s.blur) / 2.0 * kivact_types::render::SHADOW_REACH;
                offset(s.offset.x).abs().max(offset(s.offset.y).abs())
                    + offset(s.spread).max(0.0)
                    + blur
            })
            .fold(0.0, f64::max);
        let ring = p.ring.map_or(0.0, |r| extent(r.width));
        let filter: &[FilterOp] = p.effects.as_ref().map_or(&[], |e| &e.filter);
        let border = crate::lower::shape::border_widths(p.border.as_ref());
        let items = [
            p.background.is_some(),
            border != Edges::all(0.0),
            !p.shadows.is_empty(),
            p.ring.is_some(),
        ]
        .iter()
        .filter(|x| **x)
        .count() as u8;
        let effects = p.effects.as_deref();
        Look {
            opacity: unit(p.opacity, 1.0),
            overflow: shadows.max(ring),
            border: Edges {
                top: extent(border.top),
                right: extent(border.right),
                bottom: extent(border.bottom),
                left: extent(border.left),
            },
            fx: effects::filter_reach(filter).min(EVERYWHERE),
            grouped: effects::grouped(p),
            clip: p.clip,
            items,
            rounded: !p.radius.is_zero(),
            filtered: effects.is_some_and(|e| !e.filter.is_empty()),
            masked: effects.is_some_and(|e| !e.masks.is_empty()),
            backdropped: effects.is_some_and(|e| !e.backdrop.is_empty()),
            blended: effects.is_some_and(|e| e.blend != BlendMode::Normal),
            isolated: p.isolation == Isolation::Isolate,
            will_change: p.will_change,
            preserves: p.transform_style == TransformStyle::Preserve3d,
            flat: p.transform.is_2d(),
            transformed: p.transform != ElementTransform::NONE,
            perspective: p.perspective.is_some(),
            backface_hidden: p.backface == BackfaceVisibility::Hidden,
            shader: p
                .background
                .as_ref()
                .is_some_and(|fill| matches!(fill, Fill::Layers(layers) if layers.iter().any(|l| matches!(l.image, FillImage::Shader(_))))),
        }
    }

    /// Whether the node's effects group its subtree (CSS: the grouping
    /// properties that flatten a preserve-3d context).
    pub(crate) fn grouping(&self) -> bool {
        self.clip
            || self.opacity < 1.0
            || self.filtered
            || self.masked
            || self.backdropped
            || self.grouped
            || self.isolated
    }

    /// Whether the node is a flat member of a preserve-3d context: its
    /// own plane is shared with its parent.
    pub(crate) fn plane_3d(&self) -> bool {
        self.preserves && !self.grouping()
    }
}

/// The facts of every drawn subtree of the layer roots (the tree and the
/// overlays and fixed boxes); only the nodes with something to say are kept.
pub(crate) fn layer_facts(view: &impl LayerView) -> HashMap<NodeId, SubtreeFacts> {
    let mut out = HashMap::new();
    for root in view.layer_roots() {
        subtree_facts(view, root, None, &mut out);
    }
    out
}

/// The facts of the subtree at `id` (its parent's look is `parent`),
/// recorded in `out` for the nodes that have any.
pub(crate) fn subtree_facts(
    view: &impl LayerView,
    id: NodeId,
    parent: Option<&Look>,
    out: &mut HashMap<NodeId, SubtreeFacts>,
) -> SubtreeFacts {
    let Some(look) = view.drawn_look(id) else {
        return SubtreeFacts::default();
    };
    let mut below = SubtreeFacts::default();
    for child in view.flat_children(id) {
        if view.is_layer_node(child) {
            continue;
        }
        let f = subtree_facts(view, child, Some(&look), out);
        below.has_layer |= f.has_layer;
        below.has_item |= f.has_item;
        below.has_backdrop |= f.has_backdrop;
        below.has_blend |= f.has_blend;
    }
    let (scene_layer, scene_item) = view.scene_facts(id);
    below.has_layer |= scene_layer;
    // Its own background shader is an item inside its group too.
    below.has_item |= scene_item || look.shader;
    let layer = own_layer(view, id, &look, parent) || promoted(&look, &below);
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
fn own_layer(view: &impl LayerView, id: NodeId, look: &Look, parent: Option<&Look>) -> bool {
    let parent_plane_3d = parent.is_some_and(Look::plane_3d);
    let parent_perspective = parent.is_some_and(|p| p.perspective);
    let overlay = view.is_layer_node(id);
    let moves = view.moves_layer(id);
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
