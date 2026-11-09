//! Role: Transforms and z-order that decide where a node is drawn and hit.
//!
//! Where a node is drawn and hit: its layout box mapped by its element
//! transform and its ancestors' (CSS `transform`, inherited by the
//! subtree), and its order among its siblings (CSS `z-index`).
//!
//! An element transform (`ElementTransform`) is kivact's canonical 3D form
//! (`ElementTransform::matrix`: a 4 × 4 for the box's size about its
//! origin, `transform-origin` with the centre by default). A node's own
//! 4 × 4 is [`Model::local_matrix`] (its element transform, then the
//! layout motion's affine); [`Model::to_parent`] maps its box into its
//! parent's, and [`Model::window_map`] folds the steps up to the window as
//! CSS flattens them (a projective map). The paint list draws a node
//! through the affine plane map of its local matrix
//! ([`Model::local_transform`]), so the box's text, borders, corners and
//! children rotate, skew and scale with it, non-uniformly too. A state
//! style's transform replaces the base one whole: the core resolved it
//! over the base already, keeping the axes and functions it does not set.
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
//!   node's box space ([`Model::to_local`], [`Model::pointer_local`]). The same transform places the caret, the IME's
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
//! Limits: the paint list draws only affine plane maps; a node whose map
//! has a perspective is drawn without its transform until compositor
//! layers draw it. A child cannot be raised above nodes outside its parent (that takes an
//! overlay). A transformed node is hit by its base paint's corners and
//! clip path, as every node is.

use kivact_types::{
    DrawableId, ElementTransform, Matrix4, NodeId, Perspective, Point, Projective, Rect, Size,
    TransformStyle,
};

use crate::contract::{DrawableMode, NodeType};
use crate::model::Model;
use crate::tree::node::{HostNode, Place};

/// The determinant below which a node's map counts as singular.
const SINGULAR: f64 = 1e-12;

/// One step of a node up its parent chain.
struct Step {
    /// Node box space to parent box space.
    matrix: Matrix4,
    /// Where the step leads.
    up: Up,
}

/// Where a node's step leads.
enum Up {
    /// Into the window: the node is the root or a layer.
    Window,
    /// Into the flat parent `parent`.
    Parent {
        parent: NodeId,
        /// The parent shares a 3D context with its children.
        preserves_3d: bool,
    },
}

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

    /// Whether the running state motion of `id` has a current transform
    /// that is not none.
    pub(crate) fn motion_moves(&self, id: NodeId) -> bool {
        self.motion
            .get(id)
            .is_some_and(|a| !a.current_paint().transform.is_none())
    }

    /// Derive the place of `id` again and return the one it had. The only
    /// caller of `NodeTree::update_place`, so `Place::transformed` has one
    /// writer. `moving` is [`Model::motion_moves`] as the caller
    /// sees it. A node whose parent has a perspective, or whose own
    /// perspective moves its children, keeps them in step.
    pub(crate) fn refresh_place(&mut self, id: NodeId, moving: bool) -> Place {
        let Some(node) = self.tree.get(id) else {
            return Place::default();
        };
        let Some(index) = self.tree.nodes.index_of(id) else {
            return Place::default();
        };
        let parent = self.tree.flat_parent(node);
        let under_perspective =
            parent.is_some_and(|p| self.effective_paint(p).perspective.is_some());
        let old = self.tree.update_place(index, moving, under_perspective);
        if self.effective_paint(id).perspective.is_some() {
            let children: Vec<NodeId> = self.tree.flat_children(id).collect();
            for child in children {
                if self.tree.raise_under_perspective(child) {
                    self.changes.restack = true;
                    self.stale_fact(child);
                    self.repaint(child);
                }
            }
        }
        old
    }

    /// The 2D transform of `id` relative to its box's top-left corner, for a
    /// box of `size`, as the paint list draws it: the affine plane map of
    /// [`Model::local_matrix`] (`None`: none, or not affine).
    pub(crate) fn local_transform(
        &self,
        id: NodeId,
        size: Size,
    ) -> Option<kivact_types::Transform> {
        let m = self.local_matrix(id, size)?.flatten().as_affine()?;
        (!m.is_identity()).then_some(m)
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

    /// The node's own 4x4 about its box: its element transform for a box of
    /// `size`, then the layout motion's affine (`None`: it has neither; a
    /// value that is not finite is none).
    pub(crate) fn local_matrix(&self, id: NodeId, size: Size) -> Option<Matrix4> {
        let element = self
            .element_transform(id)
            .map(|t| t.matrix(size))
            .filter(Matrix4::is_finite);
        let layout = self.layout_motion_transform(id).map(Matrix4::from_affine);
        match (element, layout) {
            (Some(element), Some(layout)) => Some(element.then(layout)),
            (element, layout) => element.or(layout),
        }
    }

    /// Node box space to parent box space: the local matrix, then the move
    /// to the node's place in its parent (its layout location less the
    /// parent's scroll, a sticky box's shift), then the parent's perspective.
    /// A layer (overlay or fixed box) is placed in the window instead.
    /// `None` for a result that is not finite or is singular: the node and
    /// its subtree are neither drawn nor hit.
    pub(crate) fn to_parent(&self, id: NodeId) -> Option<Matrix4> {
        self.step(id).map(|step| step.matrix)
    }

    /// The step of `id` up its parent chain (see [`Model::to_parent`]).
    fn step(&self, id: NodeId) -> Option<Step> {
        self.step_in(id, true)
    }

    /// The step of `id`. Without `scrolled` it leaves out its parent's
    /// scroll, so the box stays where its layout puts it (content space).
    fn step_in(&self, id: NodeId, scrolled: bool) -> Option<Step> {
        let n = self.tree.get(id)?;
        if n.is_transparent() {
            return None;
        }
        if let Some(role) = n.properties.drawable {
            return match role.mode {
                DrawableMode::Canvas => self.canvas_step(id, role.id),
                DrawableMode::Export => self.export_step(id, role.id),
            };
        }
        let size = self.computed_layout(id)?.size;
        let parent = if n.is_layer() {
            None
        } else {
            self.tree.flat_parent(n)
        };
        let place = match parent {
            Some(p) => {
                let scroll = if scrolled {
                    self.tree.get(p)?.scroll
                } else {
                    Point::default()
                };
                let at = self.location(n);
                Point {
                    x: at.x - scroll.x,
                    y: at.y - scroll.y,
                }
            }
            None if n.kind() == NodeType::Overlay => self.overlay_origin(id),
            None => self.location(n),
        };
        let local = self.local_matrix(id, size);
        let mut matrix = local.unwrap_or(Matrix4::IDENTITY);
        matrix = matrix.then(Matrix4::translate(place.x, place.y, 0.0));
        let mut up = Up::Window;
        if let Some(parent) = parent {
            let paint = self.effective_paint(parent);
            // The perspective moves nothing on the plane z = 0.
            if local.is_some()
                && let Some(perspective) = paint.perspective
            {
                matrix = matrix.then(self.perspective_matrix(parent, perspective));
            }
            up = Up::Parent {
                parent,
                preserves_3d: paint.transform_style == TransformStyle::Preserve3d,
            };
        }
        let regular = matrix.is_finite() && matrix.determinant().abs() >= SINGULAR;
        regular.then_some(Step { matrix, up })
    }

    /// The step of the root `id` of the canvas drawable `drawable`: its
    /// box into its host's border box, where the host's drawing places it.
    /// `None` while no drawing places it or its back is hidden.
    fn canvas_step(&self, id: NodeId, drawable: DrawableId) -> Option<Step> {
        let host = self.tree.parent_id(self.tree.get(id)?)?;
        let matrix = self.drawn_matrix(host, drawable)?;
        let regular = matrix.is_finite() && matrix.determinant().abs() >= SINGULAR;
        regular.then_some(Step {
            matrix,
            up: Up::Parent {
                parent: host,
                preserves_3d: false,
            },
        })
    }

    /// The step of the root `id` of the exported drawable `drawable`:
    /// its box into its host's border box, through the geometry the app set
    /// for it. `None` without geometry, the way a canvas root without a
    /// drawing has none.
    fn export_step(&self, id: NodeId, drawable: DrawableId) -> Option<Step> {
        let host = self.tree.parent_id(self.tree.get(id)?)?;
        let geometry = self.drawables.geometry_of(drawable)?;
        let origin = self.padding_origin(host)?;
        let matrix =
            Matrix4::from_projective(geometry).then(Matrix4::translate(origin.x, origin.y, 0.0));
        let regular = matrix.is_finite() && matrix.determinant().abs() >= SINGULAR;
        regular.then_some(Step {
            matrix,
            up: Up::Parent {
                parent: host,
                preserves_3d: false,
            },
        })
    }

    /// The perspective that the children of `parent` are in, when the parent's
    /// box origin is at `origin` in the walk's space: `T(q) P(d) T(-q)` with
    /// `q` the perspective origin in that space. `None` without one.
    pub(crate) fn perspective_at(&self, parent: NodeId, origin: Point) -> Option<Matrix4> {
        let perspective = self.effective_paint(parent).perspective?;
        Some(
            Matrix4::translate(-origin.x, -origin.y, 0.0)
                .then(self.perspective_matrix(parent, perspective))
                .then(Matrix4::translate(origin.x, origin.y, 0.0)),
        )
    }

    /// `T(o) * P(d) * T(-o)`: the perspective of the node `parent`, with its
    /// origin resolved against its border box.
    fn perspective_matrix(&self, parent: NodeId, perspective: Perspective) -> Matrix4 {
        let size = self
            .computed_layout(parent)
            .map(|l| l.size)
            .unwrap_or_default();
        let (ox, oy) = (
            perspective.origin[0].of(size.width),
            perspective.origin[1].of(size.height),
        );
        Matrix4::translate(-ox, -oy, 0.0)
            .then(Matrix4::perspective(perspective.distance))
            .then(Matrix4::translate(ox, oy, 0.0))
    }

    /// Node box space to window logical px, flattened as CSS says
    /// (accumulated 3D matrices inside a preserved-3d context, flattened
    /// where it ends). `None` for a result that is not finite or singular.
    pub(crate) fn window_map(&self, id: NodeId) -> Option<Projective> {
        self.map_between(id, None)
    }

    /// The same up to the node `upto` (exclusive; the window with `None`):
    /// node box space to the box space of `upto`.
    pub(crate) fn map_between(&self, id: NodeId, upto: Option<NodeId>) -> Option<Projective> {
        self.map_walk(id, upto, true)
    }

    /// [`Model::map_between`] in content space: no step leaves its parent's
    /// scroll in, so the boxes are where their layouts put them.
    pub(crate) fn content_map_between(&self, id: NodeId, upto: NodeId) -> Option<Projective> {
        self.map_walk(id, Some(upto), false)
    }

    fn map_walk(&self, id: NodeId, upto: Option<NodeId>, scrolled: bool) -> Option<Projective> {
        let mut flat = Projective::IDENTITY;
        let mut deep = Matrix4::IDENTITY;
        let mut cur = Some(id);
        while let Some(x) = cur.filter(|x| Some(*x) != upto) {
            let step = self.step_in(x, scrolled)?;
            let (next, preserves_3d) = match step.up {
                Up::Window => (None, false),
                Up::Parent {
                    parent,
                    preserves_3d,
                } => (Some(parent), preserves_3d),
            };
            if preserves_3d {
                deep = deep.then(step.matrix);
            } else {
                flat = flat.then(deep.then(step.matrix).flatten());
                deep = Matrix4::IDENTITY;
            }
            cur = next;
        }
        let flat = flat.then(deep.flatten());
        flat.m
            .iter()
            .flatten()
            .all(|v| v.is_finite())
            .then_some(flat)
    }

    /// The window bounds of the node's border box as drawn.
    pub(crate) fn bounds(&self, id: NodeId) -> Option<Rect> {
        let size = self.computed_layout(id)?.size;
        let map = self.window_map(id)?;
        map.bounds(Rect {
            x: 0.0,
            y: 0.0,
            width: size.width,
            height: size.height,
        })
    }

    /// The window point `at` in the node's box space (`None`: its map has
    /// no inverse, or the point is not in front of the viewer).
    pub(crate) fn to_local(&self, id: NodeId, at: Point) -> Option<Point> {
        self.window_map(id)?.invert()?.apply(at)
    }

    /// The window point `at` relative to `id`'s box for a pointer event
    /// (under a map without an inverse, which no pointer hits but a capture
    /// may still follow, the point as if untransformed).
    pub(crate) fn pointer_local(&self, id: NodeId, at: Point) -> Point {
        self.to_local(id, at).unwrap_or_else(|| {
            let r = self.rect(id).unwrap_or_default();
            Point {
                x: at.x - r.x,
                y: at.y - r.y,
            }
        })
    }

    /// The window bounds of `rect`, given in the untransformed layout space
    /// of the window (a box position plus offsets within the box of `id`),
    /// as drawn through the node's map (`rect` itself without one).
    pub(crate) fn drawn_rect(&self, id: NodeId, rect: Rect) -> Rect {
        let Some(origin) = self.rect(id) else {
            return rect;
        };
        let local = Rect {
            x: rect.x - origin.x,
            y: rect.y - origin.y,
            ..rect
        };
        self.window_map(id)
            .and_then(|map| map.bounds(local))
            .unwrap_or(rect)
    }

    /// The bounds of `rect` (the node `id`'s untransformed layout box, in the
    /// window's layout space) as drawn in the layout space of its ancestor
    /// `within`, whose box is `view` there (`rect` itself without a map).
    pub(crate) fn drawn_within(&self, id: NodeId, within: NodeId, rect: Rect, view: Rect) -> Rect {
        let local = Rect {
            x: 0.0,
            y: 0.0,
            width: rect.width,
            height: rect.height,
        };
        self.map_between(id, Some(within))
            .and_then(|map| map.bounds(local))
            .map_or(rect, |b| Rect {
                x: view.x + b.x,
                y: view.y + b.y,
                ..b
            })
    }

    /// The window point of `local`, relative to `id`'s untransformed box
    /// (the test driver acts on points of an element).
    #[cfg(feature = "testing")]
    pub(crate) fn local_to_window(&self, id: NodeId, local: Point) -> Point {
        let r = self.rect(id).unwrap_or_default();
        self.window_map(id)
            .and_then(|map| map.apply(local))
            .unwrap_or(Point {
                x: r.x + local.x,
                y: r.y + local.y,
            })
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
