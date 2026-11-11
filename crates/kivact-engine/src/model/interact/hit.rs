//! Role: Find the node under a window point: the layers top first, then the
//! root's tree, the deepest shown hit-testable box first.
//!
//! Invariants:
//! - A node is hit inside its box's shape as painted (`paint::shape`): its
//!   rounded corners and its clip path, which also clips its subtree, as
//!   does a clipping node's padding box (rounded by its inner radii). The
//!   shapes are the node's base paint's (state styles do not move them).
//! - A node that is not hit testable (`hit_testable(false)`, CSS
//!   `pointer-events: none`) takes its whole subtree out of hit testing:
//!   the pointer reaches the nodes behind it, for hover, presses and the
//!   wheel alike. Keyboard focus and accessibility do not change.
//! - A child's subtree is looked at only when the point lies in the child's
//!   ink (`HitInk`: everything the child and its subtree can draw on, which
//!   holds every box of the subtree as drawn), so a point inside a long list
//!   visits the rows around it, not the list. A child whose ink does not
//!   bound its hits is always looked at: a sticky box (its shift is not in
//!   its ink), and one whose subtree holds a box drawn outside the ink
//!   (`HitReach::Anywhere`: a transform that is not 2D, which a perspective
//!   or a 3D rendering context projects, or a drawable, which its host's
//!   drawing places), with its ancestors up to a box that clips.
//! - The ink is the paint list's (kept until something in the subtree changes
//!   what or where it draws, computed when missing), never stale between
//!   syncs. The debug self-check compares every settled hit with the complete
//!   walk (`hit_reference`), drawables included.

use crate::layer::Look;
use crate::model::Model;
use crate::tree::node::HostNode;
use kivact_types::geometry3d::Matrix4;
use kivact_types::{NodeId, Point, Rect};
use std::collections::{HashMap, HashSet};

/// One plane of a 3D rendering context: its node and the map from its box
/// space to the box space of the context root's parent.
struct ContextPlane {
    id: NodeId,
    map: Matrix4,
}

/// The drawing's answer to where a subtree can be hit.
pub(crate) trait HitInk {
    /// Where the box `id` (`n`, laid out as `layout`) and its subtree can
    /// be hit.
    fn hit_ink(&self, id: NodeId, n: &HostNode, layout: &kivact_layout::NodeLayout) -> HitReach;
}

/// Where a subtree can be hit (see [`HitInk`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum HitReach {
    /// Nowhere: it draws nothing (it or an ancestor is hidden).
    Nothing,
    /// Inside its ink: relative to its layout location in its parent's
    /// unscrolled content space, mapped by its own transform.
    Within(Rect),
    /// Anywhere: a box of it is drawn outside its ink.
    Anywhere,
}

/// How a hit test walks the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HitWalk {
    /// Children are left out by their inks.
    Pruned,
    /// Every child is looked at: the debug self-check's reference.
    #[cfg(all(debug_assertions, feature = "testing"))]
    Complete,
}

/// Logical pixels an ink is grown by: it is computed relative to the
/// child's location and compared in the parent's space.
const HIT_SLACK: f64 = 0.5;

impl Model {
    /// Whether `p` (in the box space of the parent, which scrolls its children
    /// by `scroll`) can hit the subtree of its child `c` (see the module
    /// invariants).
    fn may_hit(&self, c: NodeId, cn: &HostNode, scroll: Point, p: Point) -> bool {
        if cn.place.positioning == crate::tree::node::Positioning::Sticky {
            return true;
        }
        let layout = self.computed_layout(c).unwrap_or_default();
        let ink = match self.hit_ink(c, cn, &layout) {
            HitReach::Nothing => return false, // not drawn, so not shown
            HitReach::Anywhere => return true,
            HitReach::Within(ink) => ink,
        };
        let x = ink.x + layout.location.x - scroll.x;
        let y = ink.y + layout.location.y - scroll.y;
        p.x >= x - HIT_SLACK
            && p.x <= x + ink.width + HIT_SLACK
            && p.y >= y - HIT_SLACK
            && p.y <= y + ink.height + HIT_SLACK
    }

    /// The deepest shown node under a window point (the layers first, the
    /// top one first). While a modal overlay is shown only its subtree can
    /// be hit; everything else is an inert backdrop.
    pub(crate) fn hit(&self, p: Point) -> Option<NodeId> {
        self.hit_with(p, HitWalk::Pruned)
    }

    /// [`Model::hit`] by the complete walk: the debug self-check's reference.
    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) fn hit_reference(&self, p: Point) -> Option<NodeId> {
        self.hit_with(p, HitWalk::Complete)
    }

    fn hit_with(&self, p: Point, walk: HitWalk) -> Option<NodeId> {
        let modal = self.modal();
        for (o, _) in self.layers().into_iter().rev() {
            if modal.is_some_and(|m| !self.tree.is_in_subtree(o, m)) {
                continue;
            }
            if let Some(h) = self.hit_node(o, p, walk) {
                return Some(h);
            }
        }
        if modal.is_some() {
            return None;
        }
        self.hit_node(self.tree.root?, p, walk)
    }

    /// The deepest shown node under a window point, ignoring modals.
    pub(crate) fn hit_any(&self, p: Point) -> Option<NodeId> {
        for (layer, _) in self.layers().into_iter().rev() {
            if let Some(hit) = self.hit_node(layer, p, HitWalk::Pruned) {
                return Some(hit);
            }
        }
        self.tree
            .root
            .and_then(|root| self.hit_node(root, p, HitWalk::Pruned))
    }

    /// The deepest node under `p` in `id`'s subtree. `p` is in the box space
    /// of `id`'s parent (the window for a root or a layer), where
    /// [`Model::to_parent`] maps the box of `id`. A node whose map is not
    /// invertible, or puts the point behind the viewer, is not hit; nor is
    /// a back face that is hidden. A 3D rendering context is hit by
    /// depth ([`Model::hit_context`]). `walk` is the one of the hit test that
    /// reaches `id` (a drawable root is reached from its host).
    pub(crate) fn hit_node(&self, id: NodeId, p: Point, walk: HitWalk) -> Option<NodeId> {
        self.hit_in(id, p, None, walk)
    }

    /// [`Model::hit_node`], with the nodes in `skip` left out: the planes of
    /// an enclosing 3D context, which are hit in its depth order instead.
    fn hit_in(
        &self,
        id: NodeId,
        p: Point,
        skip: Option<&HashSet<NodeId>>,
        walk: HitWalk,
    ) -> Option<NodeId> {
        let n = self.tree.get(id)?;
        if !self.tree.shown(id) || !n.hit_testable() {
            return None;
        }
        let map = self.to_parent(id)?;
        if self.backface_hidden_at(id, &map) {
            return None;
        }
        if Look::now(self, id, n).plane_3d() {
            return self.hit_context(id, map, p, walk);
        }
        // Into its own box space.
        let p = map.flatten().invert()?.apply(p)?;
        self.hit_box(id, p, skip, walk)
    }

    /// Whether the node `id`, under `map` (its box space to its parent's),
    /// shows a back face that is hidden.
    fn backface_hidden_at(&self, id: NodeId, map: &Matrix4) -> bool {
        map.shows_back()
            && self.effective_paint(id).backface == kivact_types::paint::BackfaceVisibility::Hidden
    }

    /// The deepest node under `p` (in `id`'s box space) among `id` and its
    /// subtree, leaving out the children in `skip`.
    fn hit_box(
        &self,
        id: NodeId,
        p: Point,
        skip: Option<&HashSet<NodeId>>,
        walk: HitWalk,
    ) -> Option<NodeId> {
        let n = self.tree.get(id)?;
        let size = self.computed_layout(id)?.size;
        let r = Rect {
            x: 0.0,
            y: 0.0,
            width: size.width,
            height: size.height,
        };
        let paint = n.paint();
        // A clip path clips the node and its subtree.
        if paint.is_some_and(|paint| !crate::lower::shape::in_clip_path(paint, r, p)) {
            return None;
        }
        let inside = crate::lower::shape::hits(paint, r, p);
        if n.clips() && !crate::lower::shape::in_child_clip(paint, r, p) {
            return inside.then_some(id);
        }
        if self.scrollbar_at(id, p).is_some() {
            return Some(id);
        }
        // Drawables drawn over the children are hit first.
        if let Some(h) = self.drawable_hit(id, 1, p, walk) {
            return Some(h);
        }
        let scroll = n.scroll();
        for c in self.stacked(n).rev() {
            if skip.is_some_and(|skip| skip.contains(&c)) {
                continue;
            }
            let Some(cn) = self.tree.get(c) else {
                continue;
            };
            if cn.is_layer() || cn.properties.drawable.is_some() {
                continue;
            }
            if walk == HitWalk::Pruned && !self.may_hit(c, cn, scroll, p) {
                continue;
            }
            if let Some(h) = self.hit_in(c, p, skip, walk) {
                return Some(h);
            }
        }
        if let Some(h) = self
            .drawable_hit(id, 0, p, walk)
            .or_else(|| self.exported_hit(id, p, walk))
        {
            return Some(h);
        }
        inside.then_some(id)
    }

    /// The deepest node under `p` (in the box space of the root `root`'s
    /// parent) of the 3D rendering context rooted at `root`, whose map is
    /// `root_map`. Its planes are hit in depth order, nearer first
    /// (ties: the later one in paint order first). Past
    /// `MAX_CONTEXT_PLANES` planes, the context is hit flat: the later plane
    /// in paint order first, without depth.
    fn hit_context(
        &self,
        root: NodeId,
        root_map: Matrix4,
        p: Point,
        walk: HitWalk,
    ) -> Option<NodeId> {
        let root_look = Look::now(self, root, self.tree.get(root)?);
        let mut planes = vec![ContextPlane {
            id: root,
            map: root_map,
        }];
        self.collect_planes(root, root_map, &root_look, &mut planes);
        let skip: HashSet<NodeId> = planes.iter().map(|plane| plane.id).collect();
        let flat = planes.len() > kivact_types::render::MAX_CONTEXT_PLANES;
        let mut hits: Vec<(usize, f64, Point)> = planes
            .iter()
            .enumerate()
            .filter_map(|(order, plane)| {
                let (depth, local) = self.plane_point(plane, p)?;
                Some((order, depth, local))
            })
            .collect();
        if flat {
            hits.sort_by_key(|hit| std::cmp::Reverse(hit.0));
        } else {
            hits.sort_by(|a, b| b.1.total_cmp(&a.1).then(b.0.cmp(&a.0)));
        }
        hits.into_iter()
            .find_map(|(order, _, local)| self.hit_box(planes[order].id, local, Some(&skip), walk))
    }

    /// The planes of a 3D context below `parent`, whose map to the context
    /// root's parent is `parent_map`: each layer child of
    /// `parent` (a child with a clip is not one, since the clip flattens it, and neither is
    /// a fixed box or an overlay), in paint order, and the layers under the
    /// ones that preserve 3D. Each plane is listed before its own children.
    fn collect_planes(
        &self,
        parent: NodeId,
        parent_map: Matrix4,
        parent_look: &Look,
        out: &mut Vec<ContextPlane>,
    ) {
        let Some(pn) = self.tree.get(parent) else {
            return;
        };
        for child in self.stacked(pn) {
            let Some(cn) = self.tree.get(child) else {
                continue;
            };
            if cn.is_layer() || !self.tree.shown(child) || !cn.hit_testable() {
                continue;
            }
            let mut facts = HashMap::new();
            if !self
                .subtree_facts(child, Some(parent_look), &mut facts)
                .layer
            {
                continue;
            }
            let Some(step) = self.to_parent(child) else {
                continue;
            };
            let map = step.then(parent_map);
            out.push(ContextPlane { id: child, map });
            let look = Look::now(self, child, cn);
            if look.plane_3d() {
                self.collect_planes(child, map, &look, out);
            }
        }
    }

    /// The point `p` (in the root's parent's box space) in the plane's box
    /// space, and the plane's depth at it, or `None` when the plane is not
    /// hit there: it does not invert, the point is behind the viewer, or it
    /// shows a hidden back face.
    fn plane_point(&self, plane: &ContextPlane, p: Point) -> Option<(f64, Point)> {
        if self.backface_hidden_at(plane.id, &plane.map) {
            return None;
        }
        let local = plane.map.flatten().invert()?.apply(p)?;
        let q = plane.map.transform_point([local.x, local.y, 0.0, 1.0]);
        let depth = q[2] / q[3];
        depth.is_finite().then_some((depth, local))
    }
}
