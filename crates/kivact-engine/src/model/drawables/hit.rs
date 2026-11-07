//! Role: Hit tests of drawables: a canvas's drawables and its hosted exports, in the order §10.4 gives.
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

use kivact_types::{DrawableId, NodeId, Point, Rect};

use crate::contract::{DrawableMode, SceneLayer};
use crate::model::Model;
use crate::model::drawables::place::{DrawnPlacement, drawable_placements};
use crate::model::interact::visual;

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
            .find_map(|(id, placed)| self.hit_placed(*id, placed, canvas))
    }

    /// The exported drawable of `host` under `p` (`p` in `host`'s box space),
    /// through the geometry the app set for it.
    pub(crate) fn exported_hit(&self, host: NodeId, p: Point) -> Option<NodeId> {
        let canvas = self.canvas_point(host, p)?;
        self.drawables
            .iter()
            .filter(|(_, entry)| entry.mode == DrawableMode::Export)
            .filter(|(_, entry)| self.parent_of(entry.root) == Some(host))
            .find_map(|(id, entry)| {
                let map = self.drawables.geometry_of(id)?;
                let at = map.invert()?.apply(canvas)?;
                self.hit_node(entry.root, at)
            })
    }

    /// The point `p` of `host`'s box space, in its canvas (padding box) space.
    fn canvas_point(&self, host: NodeId, p: Point) -> Option<Point> {
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
            x: p.x - padding.x,
            y: p.y - padding.y,
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

    /// Hit a placed drawable at the canvas point `canvas`.
    fn hit_placed(&self, id: DrawableId, placed: &DrawnPlacement, canvas: Point) -> Option<NodeId> {
        if placed.back_hidden {
            return None;
        }
        for (map, shape) in &placed.clips {
            let at = map.apply(canvas)?;
            if !shape.contains(at) {
                return None;
            }
        }
        let at = placed.map.invert()?.apply(canvas)?;
        let root = self.drawables.get(id)?.root;
        self.hit_node(root, at)
    }
}
