//! Role: Hit tests of drawables: canvas drawables and exports, in §10.4 order.
//!
//! A host asks at two points of its walk: a slot-1 (`draw_over`) drawable
//! comes before the host's children, a slot-0 (`draw`) drawable and an
//! exported drawable after them. A drawable is hit through its placement:
//! the clips must contain the point, the back face must show, and the
//! point is then mapped into the drawable's box, where its root is hit.
//!
//! Invariants:
//! - A drawable is hit only by its own host (the parent of its root).
//! - A hidden back face and an unplaced or ungeometried drawable are never hit.

use kivact_types::{DrawableId, Matrix4, NodeId, Point, Rect};

use crate::contract::{DrawableMode, SceneLayer};
use crate::model::Model;
use crate::model::interact::visual;

use super::place::{DrawnPlacement, drawable_placements};

impl Model {
    /// The deepest drawable of `host` drawn in `slot` (0 or 1) under `p`,
    /// where `p` is in `host`'s box space. The latest drawn answers first.
    pub(crate) fn drawable_hit(&self, host: NodeId, slot: u8, p: Point) -> Option<NodeId> {
        let canvas = self.canvas_point(host, p)?;
        let mut drawn: Vec<(DrawableId, DrawnPlacement)> = self
            .host_placements(host)
            .into_iter()
            .filter(|(_, placed)| placed.slot == slot)
            .collect();
        drawn.sort_by_key(|(_, placed)| std::cmp::Reverse(placed.order));
        drawn
            .iter()
            .find_map(|(id, placed)| self.hit_placed(*id, placed, canvas, p))
    }

    /// The placement of canvas drawable `drawable` by `host`'s two scenes:
    /// its first drawing (slot 0 before slot 1, then command order).
    pub(crate) fn host_placement(
        &self,
        host: NodeId,
        drawable: DrawableId,
    ) -> Option<DrawnPlacement> {
        let n = self.tree.get(host)?;
        let scenes = [
            n.scene(SceneLayer::Draw).map(|s| &**s),
            n.scene(SceneLayer::DrawOver).map(|s| &**s),
        ];
        drawable_placements(scenes).by_id.remove(&drawable)
    }

    /// The box of the canvas drawable `drawable` mapped into `host`'s border
    /// box, where the host's drawing places it. `None` when no drawing does,
    /// or the back of the drawing is hidden (§10.4).
    pub(crate) fn drawn_matrix(&self, host: NodeId, drawable: DrawableId) -> Option<Matrix4> {
        let placed = self.host_placement(host, drawable)?;
        if placed.back_hidden {
            return None;
        }
        let origin = self.padding_origin(host)?;
        Some(Matrix4::from_projective(placed.map).then(Matrix4::translate(origin.x, origin.y, 0.0)))
    }

    /// The exported drawable of `host` under `p` (`p` in `host`'s box space).
    /// Its root's own map (`export_step`) takes the point into its box.
    pub(crate) fn exported_hit(&self, host: NodeId, p: Point) -> Option<NodeId> {
        self.drawables
            .iter()
            .filter(|(_, entry)| entry.mode == DrawableMode::Export)
            .filter(|(_, entry)| self.parent_of(entry.root) == Some(host))
            .find_map(|(_, entry)| self.hit_node(entry.root, p))
    }

    /// The origin of `host`'s padding box in its border box: the canvas that
    /// its drawings and exported drawables are mapped into.
    pub(crate) fn padding_origin(&self, host: NodeId) -> Option<Point> {
        let n = self.tree.get(host)?;
        let size = self.computed_layout(host)?.size;
        let paint = n.paint().cloned().unwrap_or_default();
        let padding = visual::padding_box(
            Rect {
                x: 0.0,
                y: 0.0,
                width: size.width,
                height: size.height,
            },
            &paint,
        );
        Some(Point {
            x: padding.x,
            y: padding.y,
        })
    }

    /// The point `p` of `host`'s box space, in its canvas (padding box) space.
    fn canvas_point(&self, host: NodeId, p: Point) -> Option<Point> {
        let origin = self.padding_origin(host)?;
        Some(Point {
            x: p.x - origin.x,
            y: p.y - origin.y,
        })
    }

    /// The canvas drawables of `host`, with their placements from its two scenes.
    fn host_placements(&self, host: NodeId) -> Vec<(DrawableId, DrawnPlacement)> {
        let hosted = self
            .drawables
            .iter()
            .any(|(id, _)| self.hosts_canvas_drawable(id, host));
        let Some(n) = self.tree.get(host).filter(|_| hosted) else {
            return Vec::new();
        };
        let scenes = [
            n.scene(SceneLayer::Draw).map(|s| &**s),
            n.scene(SceneLayer::DrawOver).map(|s| &**s),
        ];
        drawable_placements(scenes)
            .by_id
            .into_iter()
            .filter(|(id, _)| self.hosts_canvas_drawable(*id, host))
            .collect()
    }

    /// Whether `id` is a canvas drawable mounted under `host`.
    fn hosts_canvas_drawable(&self, id: DrawableId, host: NodeId) -> bool {
        self.drawables.get(id).is_some_and(|entry| {
            entry.mode == DrawableMode::Canvas && self.parent_of(entry.root) == Some(host)
        })
    }

    /// The parent of a node in the tree.
    fn parent_of(&self, node: NodeId) -> Option<NodeId> {
        let n = self.tree.get(node)?;
        self.tree.parent_id(n)
    }

    /// Hit a placed drawable: `canvas` is the point in the host's padding box
    /// and `at` the same point in its border box, where the root's own map
    /// (`to_parent`) takes it into the drawable's box.
    fn hit_placed(
        &self,
        id: DrawableId,
        placed: &DrawnPlacement,
        canvas: Point,
        at: Point,
    ) -> Option<NodeId> {
        if placed.back_hidden {
            return None;
        }
        for (map, shape) in &placed.clips {
            let inside = map.apply(canvas)?;
            if !shape.contains(inside) {
                return None;
            }
        }
        let root = self.drawables.get(id)?.root;
        self.hit_node(root, at)
    }
}
