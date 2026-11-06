//! Role: Transforms and z-order that decide where a node is drawn and hit.
//!
//! Where a node is drawn and hit: its layout box mapped by its element
//! transform and its ancestors' (CSS `transform`, inherited by the
//! subtree), and its order among its siblings (CSS `z-index`).
//!
//! An element transform (`ElementTransform`) is kivact's canonical form,
//! `translate · rotate · skewX · scale` (the translation in px plus a 2 × 2
//! of fractions of the box's width and height), with its origin as two
//! lengths of the box (CSS `transform-origin`, the centre by default).
//! [`resolve`] makes it a matrix for the box's size, about that origin, so
//! the box's text, borders, corners and children rotate, skew and scale
//! with it, non-uniformly too. A rotation by a multiple of 90° is exact (a
//! full turn is the identity). A state style's transform replaces the base
//! one whole: the core resolved it over the base already, keeping the axes
//! and functions it does not set.
//!
//! Invariants:
//! - A node's own space is the untransformed window space of the layout
//!   ([`Model::rect`]): its box and its children's boxes keep their
//!   layout positions there, and its transform maps that space (about its
//!   origin, resolved against its box) into its parent's. The window sees
//!   a node through its own transform and every ancestor's, nearest
//!   first, up to its overlay or fixed box, which starts afresh at its
//!   window origin (it ignores its ancestors' transforms).
//! - The transform a node is drawn with is the one it is hit, pointed at,
//!   reported (`LayoutChanged`, `NodeRef::rect`: the window bounds of the
//!   transformed box) and exposed with (G-07): its base paint's with its
//!   active state styles'. A pointer event's position is the point in the
//!   node's own space, relative to its untransformed box
//!   ([`Model::to_local`]). The same transform places the caret, the IME's
//!   character boxes and text area, overlay anchors, scroll targets and
//!   the test driver's points. A transform that flattens the box
//!   (determinant 0) draws nothing and hits nothing, nor does its subtree.
//! - Children are drawn and hit in z-index order, sticky boxes after the
//!   others of their z-index (stable: equal keys keep the tree order, so a
//!   container without z-indices or sticky children keeps it). Only a
//!   container that once had a child with a z-index, a state style that
//!   sets one, or a sticky position sorts (`Place::stacks`, never cleared);
//!   every other one walks its children in tree order at no cost. Every
//!   container isolates its children's z-indices (CSS `isolation:
//!   isolate`): a negative one is drawn after its parent's background,
//!   before its other children, and none is compared with nodes outside
//!   its parent. Tab order stays the tree order.
//! - A new transform or z-index (written, or by a state) computes no
//!   layout: it marks positions moved, so the reports and the paint list
//!   follow.
//!
//! Limits: only 2D affine transforms (no `perspective`, no `rotateX`). A
//! child cannot be raised above nodes outside its parent (that takes an
//! overlay). A transformed node is hit by its base paint's corners and
//! clip path, as every node is.

use kivact_types::{ElementTransform, NodeId, Point, Rect, Size};

use crate::model::Model;
use crate::tree::node::HostNode;

/// Iterate cached box order, allocating only when z-order needs sorting.
enum StackOrder<I> {
    Tree(I),
    Sorted(std::vec::IntoIter<NodeId>),
}
impl<I: Iterator<Item = NodeId>> Iterator for StackOrder<I> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        match self {
            Self::Tree(i) => i.next(),
            Self::Sorted(i) => i.next(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Tree(i) => i.size_hint(),
            Self::Sorted(i) => i.size_hint(),
        }
    }
}
impl<I: DoubleEndedIterator<Item = NodeId>> DoubleEndedIterator for StackOrder<I> {
    fn next_back(&mut self) -> Option<NodeId> {
        match self {
            Self::Tree(i) => i.next_back(),
            Self::Sorted(i) => i.next_back(),
        }
    }
}
impl<I: ExactSizeIterator<Item = NodeId>> ExactSizeIterator for StackOrder<I> {}

/// The matrix of `t` for a box of `size`, relative to the box's top-left
/// corner (`None`: it moves nothing, or a value is not finite).
pub(crate) fn resolve(t: &ElementTransform, size: Size) -> Option<kivact_types::Transform> {
    if t.is_none() {
        return None;
    }
    let (w, h) = (size.width, size.height);
    let p = t.per_size;
    let (sin, cos) = sin_cos_deg(t.rotate);
    let k = if t.skew_x == 0.0 {
        0.0
    } else {
        t.skew_x.to_radians().tan()
    };
    let (sx, sy) = (t.scale.x, t.scale.y);
    // translate · rotate · skewX · scale.
    let m = kivact_types::Transform {
        a: cos * sx,
        b: sin * sx,
        c: (cos * k - sin) * sy,
        d: (sin * k + cos) * sy,
        e: t.translate.x + p[0] * w + p[1] * h,
        f: t.translate.y + p[2] * w + p[3] * h,
    };
    let origin = Point {
        x: t.origin[0].of(w),
        y: t.origin[1].of(h),
    };
    let m = m.about(origin);
    let finite = [m.a, m.b, m.c, m.d, m.e, m.f].iter().all(|v| v.is_finite());
    (finite && !m.is_identity()).then_some(m)
}

/// `sin` and `cos` of `deg` degrees, exact at the quarter turns (a
/// rotation by 360° is the identity, its text hinted again).
fn sin_cos_deg(deg: f64) -> (f64, f64) {
    let turn = deg.rem_euclid(360.0);
    if turn == 0.0 {
        (0.0, 1.0)
    } else if turn == 90.0 {
        (1.0, 0.0)
    } else if turn == 180.0 {
        (0.0, -1.0)
    } else if turn == 270.0 {
        (-1.0, 0.0)
    } else {
        deg.to_radians().sin_cos()
    }
}

/// `t` moved to a box at `at` (its matrix is relative to the box's
/// top-left corner).
pub(crate) fn placed(t: kivact_types::Transform, at: Point) -> kivact_types::Transform {
    kivact_types::Transform::translate(-at.x, -at.y)
        .then(t)
        .then(kivact_types::Transform::translate(at.x, at.y))
}

/// The axis-aligned bounds of `r` mapped by `t` (`r` itself without one).
pub(crate) fn map_rect(t: Option<&kivact_types::Transform>, r: Rect) -> Rect {
    match t {
        Some(t) => t.bounds(r),
        None => r,
    }
}

impl Model {
    /// The element transform of `id` now: its base paint's with its active
    /// state styles' (`None`: it has none).
    pub(crate) fn element_transform(&self, id: NodeId) -> Option<ElementTransform> {
        let n = self.tree.get(id)?;
        if !n.place.transformed {
            return None;
        }
        if let Some(a) = self.motion.get(id) {
            let t = a.current_paint().transform;
            return (!t.is_none()).then_some(t);
        }
        let mut t = n.paint().map(|p| p.transform).unwrap_or_default();
        if n.states().is_some() {
            for d in self.active_deltas(id) {
                if let Some(x) = d.transform {
                    t = x;
                }
            }
        }
        (!t.is_none()).then_some(t)
    }

    /// The transform of `id` relative to its box's top-left corner, for a
    /// box of `size` (`None`: none).
    pub(crate) fn local_transform(
        &self,
        id: NodeId,
        size: Size,
    ) -> Option<kivact_types::Transform> {
        let element = self.element_transform(id).and_then(|t| resolve(&t, size));
        match (element, self.layout_motion_transform(id)) {
            (Some(element), Some(layout)) => Some(element.then(layout)),
            (element, layout) => element.or(layout),
        }
    }

    /// The transform of `id` from its own space to its parent's: about its
    /// box `rect` there (`None`: none).
    pub(crate) fn transform_at(&self, id: NodeId, rect: Rect) -> Option<kivact_types::Transform> {
        let size = Size {
            width: rect.width,
            height: rect.height,
        };
        let t = self.local_transform(id, size)?;
        Some(placed(
            t,
            Point {
                x: rect.x,
                y: rect.y,
            },
        ))
    }

    /// The transform from the own space of `id` to the own space of its
    /// ancestor `upto` (the window with `None`): the transforms of `id`
    /// and its ancestors below `upto`, nearest first, up to its overlay
    /// (`None`: none of them is transformed).
    pub(crate) fn transform_between(
        &self,
        id: NodeId,
        upto: Option<NodeId>,
    ) -> Option<kivact_types::Transform> {
        let mut out: Option<kivact_types::Transform> = None;
        let mut cur = Some(id);
        while let Some(x) = cur {
            if Some(x) == upto {
                break;
            }
            let Some(n) = self.tree.get(x) else { break };
            if (n.place.transformed || self.layout_motion_transform(x).is_some())
                && let Some(r) = self.rect(x)
                && let Some(t) = self.transform_at(x, r)
            {
                out = Some(out.map_or(t, |o| o.then(t)));
            }
            if n.is_layer() {
                // Overlays and fixed boxes are placed in the window.
                break;
            }
            cur = self.tree.flat_parent(n);
        }
        out
    }

    /// The window bounds of `id`'s box as drawn: its layout box mapped by
    /// its transforms (`NodeRef::rect`, an overlay's anchor).
    pub(crate) fn bounds(&self, id: NodeId) -> Option<Rect> {
        let r = self.rect(id)?;
        Some(map_rect(self.transform_between(id, None).as_ref(), r))
    }

    /// The window point `at` in the own space of `id` (`None`: its
    /// transforms flatten it, nothing there is hit).
    pub(crate) fn to_own(&self, id: NodeId, at: Point) -> Option<Point> {
        match self.transform_between(id, None) {
            None => Some(at),
            Some(t) => Some(t.invert()?.apply(at)),
        }
    }

    /// The window point `at` relative to `id`'s untransformed box (the
    /// position its pointer events carry; under a flattening transform,
    /// which no pointer hits but a capture may still follow, the point as
    /// if untransformed).
    pub(crate) fn to_local(&self, id: NodeId, at: Point) -> Point {
        let r = self.rect(id).unwrap_or_default();
        let own = self.to_own(id, at).unwrap_or(at);
        Point {
            x: own.x - r.x,
            y: own.y - r.y,
        }
    }

    /// The window point of `local`, relative to `id`'s untransformed box
    /// (the test driver acts on points of an element).
    #[cfg(feature = "testing")]
    pub(crate) fn local_to_window(&self, id: NodeId, local: Point) -> Point {
        let r = self.rect(id).unwrap_or_default();
        let own = Point {
            x: r.x + local.x,
            y: r.y + local.y,
        };
        match self.transform_between(id, None) {
            None => own,
            Some(t) => t.apply(own),
        }
    }

    /// The z-index of `id` now (its state styles applied).
    pub(crate) fn z_index(&self, id: NodeId) -> i32 {
        let Some(n) = self.tree.get(id) else {
            return 0;
        };
        let mut z = n.place.z;
        if n.place.z_states {
            for d in self.active_deltas(id) {
                if let Some(x) = d.z_index {
                    z = x;
                }
            }
        }
        z
    }

    /// The order of `id` among its siblings: its z-index, then whether it
    /// is sticky (CSS paints positioned boxes over the in-flow ones of the
    /// same z-index).
    pub(crate) fn stack_key(&self, id: NodeId) -> (i32, bool) {
        let sticky = self.tree.get(id).is_some_and(|n| n.place.sticky);
        (self.z_index(id), sticky)
    }

    /// Whether the children of the node `n` paint in [`stack_key`] order:
    /// it stacks, or one of its flat children has a z-index (one under a
    /// Contents node marks that node, not `n`, as stacking).
    ///
    /// [`stack_key`]: Model::stack_key
    pub(crate) fn sorts_children(&self, n: &HostNode) -> bool {
        n.place.stacks == crate::tree::node::Stacking::Stacks
            || self
                .tree
                .flat_children(self.tree.nodes.id_at(n.index))
                .any(|id| self.tree.get(id).is_some_and(|child| child.place.has_z()))
    }

    /// The children of the node `n` in paint order: by [`stack_key`], ties
    /// in tree order.
    ///
    /// [`stack_key`]: Model::stack_key
    pub(crate) fn stacked<'a>(
        &'a self,
        n: &'a HostNode,
    ) -> impl DoubleEndedIterator<Item = NodeId> + ExactSizeIterator + 'a {
        let children = self.tree.flat_children(self.tree.nodes.id_at(n.index));
        if self.sorts_children(n) {
            let mut children: Vec<_> = children.collect();
            children.sort_by_key(|id| self.stack_key(*id));
            StackOrder::Sorted(children.into_iter())
        } else {
            StackOrder::Tree(children)
        }
    }
}
