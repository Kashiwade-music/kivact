//! Role: Resolve pointer hits, canvas regions and current hover geometry.
//!
//! Geometry of the node tree: the layout pass (with the host's text
//! measurement), window rects, overlay origins, hit testing, scroll
//! offsets (wheel chains, scroll-into-view) and the `ScrollChanged` /
//! `LayoutChanged` reports (the drawn bounds and the layout size).
//!
//! Invariants:
//! - Rects are layout boxes in the untransformed window space (borders do
//!   not inset the children). Hit testing maps the point into each
//!   transformed node's own space (`model::transform`), so a node is hit
//!   where it is drawn; children are hit in reverse paint order (z-index,
//!   then tree order).
//! - A node is hit inside its box's shape as painted (`paint::shape`): its
//!   rounded corners and its clip path, which also clips its subtree, as
//!   does a clipping node's padding box (rounded by its inner radii). The
//!   shapes are the node's base paint's (state styles do not move them).
//! - A node that is not hit testable (`hit_testable(false)`, CSS
//!   `pointer-events: none`) takes its whole subtree out of hit testing:
//!   the pointer reaches the nodes behind it, for hover, presses and the
//!   wheel alike. Keyboard focus and accessibility do not change.
//! - Fixed boxes and overlays are layers over the root's tree
//!   ([`Model::layers`]): a layer leaves its parent's hit testing, and
//!   takes none of its
//!   ancestors' scroll or transforms (`rect` and `transform_between` stop
//!   at it). They are hit top first, before the root's tree; scrolling
//!   into view stops at a fixed box.
//! - A scroll offset stays within `[0, content - viewport]` on a scrolling
//!   axis and is 0 on the other; a scroll moves no layout box, so it needs
//!   no new layout computation. A native wheel scroll is laid out at once
//!   (the host runs the pass right after the wheel), so its `ScrollChanged`
//!   goes out without waiting for the next paint.
//! - Overlays are laid out as roots of their own and placed by
//!   [`kivact_layout::place_overlay`] at their anchor (centered without one, and
//!   with an anchor whose position depends on the overlay's own: inside
//!   it, or inside an overlay anchored inside it). Hit
//!   testing and the paint pass take the overlay layers from
//!   [`Model::shown_overlays`]: an overlay under a hidden ancestor is
//!   neither hit nor drawn.
//! - Reports go out in id order: every changed scroll container's
//!   `ScrollChanged`, then every changed observed node's `LayoutChanged`.

use crate::contract::{HostEvent, NodeType};
use kivact_types::{NodeId, OverlayAt, Point, Rect};

use crate::measure::leaf::Measure;
use crate::model::Model;

impl Model {
    /// Lay the tree out if anything changed since the last pass, scroll
    /// pending nodes into view and queue the geometry reports. Returns
    /// whether a pass ran. Taffy computes only when the layout tree was
    /// edited (structure, layout props, a measured size, the window);
    /// the recorded changes say whether positions may have moved.
    pub(crate) fn layout(&mut self, engine: &mut arvane::Engine) -> bool {
        let Some(remeasure) = self.changes.begin_layout() else {
            return false;
        };
        let previous = self.capture_layout_motion();
        let mut measure = Measure::new(engine, self.assets.get_mut(), &self.measure, &self.editors);
        for id in remeasure {
            if measure.measures_same(&self.tree, &self.layout, id) {
                continue;
            }
            self.layout.invalidate_chain(&self.tree, id);
        }
        // Scrolls alone keep the main layout, but may change a tooltip's available room.
        let computed = {
            let mut leaves = crate::measure::leaf::EngineMeasure {
                tree: &self.tree,
                measure: &mut measure,
            };
            self.layout.compute(&self.tree, self.viewport, &mut leaves)
        };
        let reveal = self.focus.take_reveal();
        if !reveal.is_empty() {
            self.stick();
        }
        for id in reveal {
            self.scroll_into_view(id);
        }
        let fitted = self.refit_tooltips(engine, !computed);
        if computed || fitted {
            let geometry: Vec<_> = self.layout.drain_geometry_changes().collect();
            for change in &geometry {
                self.a11y.subtree(change.id);
            }
            let tree = &self.tree;
            let repaint = geometry.into_iter().filter_map(|change| match change.kind {
                kivact_layout::GeometryChangeKind::Local => Some(change.id),
                kivact_layout::GeometryChangeKind::PositionOnly => {
                    let node = tree.get(change.id)?;
                    (!node.is_layer()).then(|| tree.flat_parent(node)).flatten()
                }
            });
            // Set pass flags even when no local ink dependency changed.
            self.changes.laid_out(repaint);
        }
        self.retarget_layout_motion(previous);
        self.report_geometry(engine);
        true
    }

    pub(crate) fn refit_tooltips(&mut self, engine: &mut arvane::Engine, reset: bool) -> bool {
        let tooltips: Vec<_> = self
            .tree
            .overlays
            .iter()
            .filter_map(|&id| {
                let props = self.tree.get(id)?.overlay()?;
                if !props.fit.is_some_and(|fit| fit.keep_anchor_clear) {
                    return None;
                }
                Some((id, props))
            })
            .collect();
        let mut computed = false;
        for (id, props) in tooltips {
            if reset {
                self.layout_tooltip(engine, id, None);
                computed = true;
            }
            let Some(anchor) = self.overlay_anchor(id, props.at) else {
                continue;
            };
            let Some(size) = self.computed_layout(id).map(|layout| layout.size) else {
                continue;
            };
            if let Some(limit) =
                kivact_layout::anchor_clear_size(&props, anchor, size, self.viewport)
            {
                self.layout_tooltip(engine, id, Some(limit));
                computed = true;
            }
        }
        computed
    }

    fn layout_tooltip(
        &mut self,
        engine: &mut arvane::Engine,
        id: NodeId,
        limit: Option<kivact_types::Size>,
    ) {
        let mut measure = Measure::new(engine, self.assets.get_mut(), &self.measure, &self.editors);
        let mut leaves = crate::measure::leaf::EngineMeasure {
            tree: &self.tree,
            measure: &mut measure,
        };
        self.layout
            .compute_overlay(&self.tree, id, self.viewport, limit, &mut leaves);
    }

    /// Window-space border box of `id` (after the last layout pass), a
    /// sticky box (or one below it) where it is stuck.
    pub(crate) fn rect(&self, id: NodeId) -> Option<Rect> {
        if self.tree.get(id)?.is_transparent() {
            return None;
        }
        let size = self.computed_layout(id)?.size;
        let (mut x, mut y) = (0.0, 0.0);
        let mut cur = id;
        loop {
            let node = self.tree.get(cur)?;
            if node.kind() == NodeType::Overlay {
                let o = self.overlay_origin(cur);
                x += o.x;
                y += o.y;
                break;
            }
            let loc = self.location(node);
            x += loc.x;
            y += loc.y;
            if node.place.fixed {
                // At its window position.
                break;
            }
            match self.tree.flat_parent(node) {
                Some(p) => {
                    let s = self.tree.get(p)?.scroll;
                    x -= s.x;
                    y -= s.y;
                    cur = p;
                }
                None => break,
            }
        }
        Some(Rect {
            x,
            y,
            width: size.width,
            height: size.height,
        })
    }

    /// Where node `n`'s box is in its parent's content
    /// (unscrolled; a fixed box's in the window): its layout location, a
    /// sticky box's moved by its shift.
    pub(crate) fn location(&self, n: &crate::tree::node::HostNode) -> Point {
        let loc = self
            .layout_at(n.index)
            .map(|l| l.location)
            .unwrap_or_default();
        if n.place.sticky {
            Point {
                x: loc.x + n.shift.x,
                y: loc.y + n.shift.y,
            }
        } else {
            loc
        }
    }

    /// Window-space origin of the overlay `id`: the placement at its
    /// anchor (flip and clamp; centered in the window without an anchor,
    /// or with one whose position depends on the overlay's own).
    pub(crate) fn overlay_origin(&self, id: NodeId) -> Point {
        let Some(o) = self.tree.get(id).and_then(|n| n.overlay()) else {
            return Point::default();
        };
        let anchor = self.overlay_anchor(id, o.at);
        let size = self.computed_layout(id).map(|l| l.size).unwrap_or_default();
        kivact_layout::place_overlay(&o, anchor, size, self.viewport)
    }

    fn overlay_anchor(&self, id: NodeId, at: OverlayAt) -> Option<Rect> {
        match at {
            OverlayAt::Point(p) => Some(Rect {
                x: p.x,
                y: p.y,
                width: 0.0,
                height: 0.0,
            }),
            OverlayAt::Anchor { node, .. } => node
                .filter(|a| !self.placed_by(*a, id))
                .and_then(|a| self.bounds(a)),
        }
    }

    /// Whether the position of `node` depends on the placement of the
    /// overlay `overlay`: `node` lies in it, or in an overlay anchored
    /// (directly or through further overlays) at a node that does. Such an
    /// anchor cannot place the overlay (it would recurse forever).
    fn placed_by(&self, node: NodeId, overlay: NodeId) -> bool {
        let mut seen = Vec::new();
        let mut cur = Some(node);
        while let Some(x) = cur {
            // The overlay whose placement moves `x`, if any (a fixed box
            // is placed by the window).
            let Some(o) = self
                .tree
                .chain(Some(x))
                .into_iter()
                .find(|c| self.tree.get(*c).is_some_and(|n| n.is_layer()))
                .filter(|c| self.tree.get(*c).is_some_and(|n| !n.place.fixed))
            else {
                return false;
            };
            if o == overlay {
                return true;
            }
            if seen.contains(&o) {
                // A cycle without `overlay`: those overlays are unanchored.
                return false;
            }
            seen.push(o);
            cur = self
                .tree
                .get(o)
                .and_then(|n| n.overlay())
                .and_then(|p| p.anchor());
        }
        false
    }

    /// The overlays drawn and hit-tested as layers over the root, in the
    /// order they were attached (the last on top): the attached overlays
    /// that are drawn, so an overlay under a hidden ancestor is neither
    /// drawn nor hit, as it confines nothing ([`Model::modal`]) and
    /// holds no tab stop. A leaving one is drawn but takes no hit.
    pub(crate) fn shown_overlays(&self) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        self.tree
            .overlays
            .iter()
            .copied()
            .filter(|o| self.tree.drawn(*o))
    }

    /// The layers drawn and hit over the root's tree, bottom first, with
    /// their window origins: the fixed boxes outside every overlay (by
    /// z-index, then in tree order), then each overlay in attach order
    /// followed by the fixed boxes inside it; only the drawn ones (one
    /// under a hidden ancestor is neither drawn nor hit, a leaving one is
    /// drawn but takes no hit).
    pub(crate) fn layers(&self) -> Vec<(NodeId, Point)> {
        let mut out = Vec::new();
        let fixed: Vec<(Option<NodeId>, NodeId)> = self
            .tree
            .fixed
            .iter()
            .copied()
            .filter(|f| self.tree.drawn(*f))
            .map(|f| (self.overlay_of(f), f))
            .collect();
        let group = |owner: Option<NodeId>, out: &mut Vec<(NodeId, Point)>| {
            let mut ids: Vec<NodeId> = fixed
                .iter()
                .filter(|(o, _)| *o == owner)
                .map(|(_, f)| *f)
                .collect();
            if ids.len() > 1 {
                ids.sort_by_cached_key(|f| (self.z_index(*f), self.tree.path(*f)));
            }
            for f in ids {
                let at = self.computed_layout(f).map(|l| l.location);
                out.push((f, at.unwrap_or_default()));
            }
        };
        group(None, &mut out);
        for o in self.shown_overlays() {
            out.push((o, self.overlay_origin(o)));
            group(Some(o), &mut out);
        }
        out
    }

    /// The nearest overlay at or above `id`.
    fn overlay_of(&self, id: NodeId) -> Option<NodeId> {
        self.tree.chain(Some(id)).into_iter().find(|c| {
            self.tree
                .get(*c)
                .is_some_and(|n| n.kind() == NodeType::Overlay)
        })
    }

    /// The deepest shown node under a window point (the layers first, the
    /// top one first). While a modal overlay is shown only its subtree can
    /// be hit; everything else is an inert backdrop.
    pub(crate) fn hit(&self, p: Point) -> Option<NodeId> {
        let modal = self.modal();
        for (o, origin) in self.layers().into_iter().rev() {
            if modal.is_some_and(|m| !self.tree.is_in_subtree(o, m)) {
                continue;
            }
            if let Some(h) = self.hit_node(o, origin, p) {
                return Some(h);
            }
        }
        if modal.is_some() {
            return None;
        }
        self.hit_node(self.tree.root?, Point::default(), p)
    }

    /// The deepest node under `p` in `id`'s subtree; `origin` is its
    /// untransformed box's origin and `p` the point, both in its parent's
    /// own space.
    fn hit_node(&self, id: NodeId, origin: Point, p: Point) -> Option<NodeId> {
        let n = self.tree.get(id)?;
        if !self.tree.shown(id) || !n.hit_testable() {
            return None;
        }
        let size = self.computed_layout(id)?.size;
        let r = Rect {
            x: origin.x,
            y: origin.y,
            width: size.width,
            height: size.height,
        };
        // Into its own space (a flattening transform hits nothing).
        let p = match n.place.transformed || self.layout_motion_transform(id).is_some() {
            true => match self.transform_at(id, r) {
                Some(t) => t.invert()?.apply(p),
                None => p,
            },
            false => p,
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
        if self
            .scrollbar_at(
                id,
                Point {
                    x: p.x - origin.x,
                    y: p.y - origin.y,
                },
            )
            .is_some()
        {
            return Some(id);
        }
        for c in self.stacked(n).rev() {
            let Some(cn) = self.tree.get(c).filter(|cn| !cn.is_layer()) else {
                continue;
            };
            let loc = self.location(cn);
            let o = Point {
                x: origin.x - n.scroll.x + loc.x,
                y: origin.y - n.scroll.y + loc.y,
            };
            if let Some(h) = self.hit_node(c, o, p) {
                return Some(h);
            }
        }
        inside.then_some(id)
    }

    /// Set the scroll offset of `id` (clamped). Returns whether it moved.
    pub(crate) fn set_scroll(&mut self, id: NodeId, offset: Point) -> bool {
        let Some(o) = self.clamped_scroll(id, offset) else {
            return false;
        };
        let current = self
            .tree
            .get(id)
            .expect("invariant: clamped scroll found a live node")
            .scroll;
        if o == current {
            return false;
        }
        if let Some(n) = self.tree.get_mut(id) {
            n.scroll = o;
        }
        self.changes.scrolled(id);
        self.a11y.subtree(id);
        true
    }

    fn clamped_scroll(&self, id: NodeId, offset: Point) -> Option<Point> {
        let n = self.tree.get(id)?;
        let l = self.computed_layout(id).unwrap_or_default();
        let (ax, ay) = n.scroll_axes();
        let max_x = (l.content_size.width - l.size.width).max(0.0);
        let max_y = (l.content_size.height - l.size.height).max(0.0);
        Some(Point {
            x: if ax { offset.x.clamp(0.0, max_x) } else { 0.0 },
            y: if ay { offset.y.clamp(0.0, max_y) } else { 0.0 },
        })
    }

    /// Scroll the nearest container at or above `from` that can still move
    /// by `delta` (edges propagate upward), stopping before `stop` and
    /// after the active modal when `from` lies in it. Returns whether
    /// something scrolled.
    pub(crate) fn scroll_from(&mut self, from: NodeId, delta: Point, stop: Option<NodeId>) -> bool {
        self.scroll_target(from, delta, stop)
            .is_some_and(|(id, target)| self.set_scroll(id, target))
    }

    /// Read the same nested/modal-bounded scroll chain without mutating it.
    pub(crate) fn scroll_target(
        &self,
        from: NodeId,
        delta: Point,
        stop: Option<NodeId>,
    ) -> Option<(NodeId, Point)> {
        let mut chain = self.tree.chain(Some(from));
        if let Some(m) = self.modal_boundary(from)
            && let Some(at) = chain.iter().position(|id| *id == m)
        {
            chain.truncate(at + 1);
        }
        for id in chain {
            if Some(id) == stop {
                return None;
            }
            let Some(n) = self.tree.get(id) else { break };
            if n.scroll_axes() == (false, false) {
                continue;
            }
            let s = n.scroll;
            let target = Point {
                x: s.x + delta.x,
                y: s.y + delta.y,
            };
            if let Some(target) = self.clamped_scroll(id, target)
                && target != s
            {
                return Some((id, target));
            }
        }
        None
    }

    /// Scroll every scroll container above `id` so that it shows `id`
    /// (using the current layout), up to a fixed box (the scrolls above it
    /// do not move it). Inner containers scroll first; each outer one
    /// reveals the target where the inner scrolls left it.
    pub(crate) fn scroll_into_view(&mut self, id: NodeId) {
        // The scroll delta that brings [t, t + tl) into [v, v + vl).
        let reveal = |t: f64, tl: f64, v: f64, vl: f64| {
            if t < v {
                t - v
            } else {
                (t + tl - v - vl).clamp(0.0, (t - v).max(0.0))
            }
        };
        let mut cur = self
            .tree
            .get(id)
            .filter(|n| !n.place.fixed)
            .and_then(|n| self.tree.flat_parent(n));
        while let Some(p) = cur {
            let Some(n) = self.tree.get(p) else { break };
            let parent = self.tree.flat_parent(n).filter(|_| !n.place.fixed);
            let axes = n.scroll_axes();
            if axes != (false, false)
                && let Some(view) = self.rect(p)
                && let Some(target) = self.rect(id)
            {
                // Respect the container's padding when revealing a target.
                // Its gutter stays available for focus rings and scrollbars.
                let view = self.computed_layout(p).map_or(view, |l| {
                    crate::model::interact::visual::content_at(view, l.content_box)
                });
                // The target as drawn in the container's own space.
                let target = crate::model::transform::map_rect(
                    self.transform_between(id, Some(p)).as_ref(),
                    target,
                );
                let mut s = n.scroll;
                if axes.1 {
                    s.y += reveal(target.y, target.height, view.y, view.height);
                }
                if axes.0 {
                    s.x += reveal(target.x, target.width, view.x, view.width);
                }
                self.set_scroll(p, s);
            }
            cur = parent;
        }
    }

    fn report_geometry(&mut self, engine: &mut arvane::Engine) {
        let scrollers: Vec<NodeId> = self
            .tree
            .scrollers
            .union(&self.tree.observed)
            .copied()
            .collect();
        for id in scrollers {
            let Some(l) = self.computed_layout(id) else {
                continue;
            };
            let scroll = self.tree.get(id).map(|n| n.scroll).unwrap_or_default();
            self.set_scroll(id, scroll);
            let Some(n) = self.tree.get_mut(id) else {
                continue;
            };
            let state =
                crate::tree::node::ScrollReport(n.scroll, l.size, l.content_size, n.scroll_axes());
            if n.sent_scroll.update(state).is_some() {
                self.push(HostEvent::ScrollChanged {
                    node: id,
                    offset: state.0,
                    viewport: l.size,
                    content: l.content_size,
                });
            }
        }
        // Clamping only settles offsets; it needs no new pass.
        self.changes.settled();
        // Sticky boxes follow the settled offsets.
        self.stick();
        let observed: Vec<NodeId> = self.tree.observed.iter().copied().collect();
        for id in observed {
            let (Some(rect), Some(l)) = (self.bounds(id), self.computed_layout(id)) else {
                continue;
            };
            let size = l.size;
            let text_truncated = super::text_geometry::text_truncated(self, engine, id);
            if let Some(n) = self.tree.get_mut(id)
                && n.sent_layout
                    .update(crate::tree::node::LayoutReport(
                        rect,
                        size,
                        if text_truncated {
                            crate::tree::node::Clipping::Yes
                        } else {
                            crate::tree::node::Clipping::No
                        },
                    ))
                    .is_some()
            {
                self.push(HostEvent::LayoutChanged {
                    node: id,
                    rect,
                    size,
                    text_truncated,
                });
            }
        }
    }
}
