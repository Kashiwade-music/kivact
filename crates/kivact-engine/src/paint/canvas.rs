//! Role: The canvas drawables of a frame: their hosts, placements and composed layers.
//!
//! Invariants:
//! - The plan is built before the hosts are composed and replaced each
//!   frame; a drawable's layers are composed from its own subtree at the
//!   window's scale, as an export's are (§6.1 R6, §10.3, §10.4).
//! - A drawing in a node other than a drawable's host is reported once per
//!   node and drawable (§10.2).

use kivact_types::render::{CompositeLayer, LayerKey};
use kivact_types::{DrawableId, NodeId, Rect, Size};

use crate::contract::{DrawableMode, HostEvent, SceneLayer};
use crate::lower::drawables::DrawablePlan;
use crate::model::Model;
use crate::model::drawables::place::drawable_placements;
use crate::paint::state::PaintPass;

impl Model {
    /// Build the frame's plan of the canvas drawables: each one's host, the
    /// drawing that places it and its composed layers.
    pub(super) fn prepare_canvas(&self, engine: &mut arvane::Engine) {
        let mut plan = DrawablePlan::default();
        for (id, entry) in self.drawables.iter() {
            if entry.mode != DrawableMode::Canvas {
                continue;
            }
            let Some(host) = self
                .tree()
                .get(entry.root)
                .and_then(|n| self.tree().parent_id(n))
            else {
                continue;
            };
            plan.hosts.insert(id, host);
            let Some(placed) = self.placement_in(host, id) else {
                continue;
            };
            plan.winners.insert(id, placed);
            if let Some(layers) = self.subtree_layers(engine, id, entry.root, self.paint.scale()) {
                plan.layers.insert(id, layers);
            }
        }
        *self.paint.drawables.borrow_mut() = plan;
    }

    /// The slot and command index of the drawing that places drawable `id`
    /// in `host`'s scenes (its first drawing).
    fn placement_in(&self, host: NodeId, id: DrawableId) -> Option<(u8, u32)> {
        let node = self.tree().get(host)?;
        let scenes = [
            node.scene(SceneLayer::Draw).map(|s| &**s),
            node.scene(SceneLayer::DrawOver).map(|s| &**s),
        ];
        let placed = drawable_placements(scenes).by_id.remove(&id)?;
        Some((placed.slot, placed.order))
    }

    /// The layers of drawable `id`'s subtree at `root`, composed at `scale`
    /// device px per logical px: the root layer (index 0) is keyed by the
    /// drawable, and the layer indices are local to the list.
    pub(super) fn subtree_layers(
        &self,
        engine: &mut arvane::Engine,
        id: DrawableId,
        root: NodeId,
        scale: f64,
    ) -> Option<Vec<CompositeLayer>> {
        let bounds = self.rect(root)?;
        // The subtree is walked at its own origin: its clip is its local box.
        let local = Rect {
            x: 0.0,
            y: 0.0,
            width: bounds.width,
            height: bounds.height,
        };
        let list = self.subtree_paint_list(root, local);
        let pass = PaintPass {
            scale,
            size: Size::new(bounds.width, bounds.height),
        };
        let mut layers = self.compose_list(engine, &list, pass).layers;
        if let Some(first) = layers.first_mut() {
            first.key = LayerKey::Drawable(id);
        }
        Some(layers)
    }

    /// Queue one report per node and drawable for the drawings of drawables
    /// that their hosts do not mount (§10.2), and for the second drawings of
    /// a drawable its host draws in both slots (§10.3).
    pub(super) fn report_misplaced(&mut self) {
        let misplaced = std::mem::take(&mut self.paint.drawables.get_mut().misplaced);
        for (node, drawable) in misplaced {
            if self
                .paint
                .misplaced_reported
                .get_mut()
                .insert((node, drawable))
            {
                self.push(HostEvent::DrawableNotHosted { node, drawable });
            }
        }
        let twice = std::mem::take(&mut self.paint.drawables.get_mut().twice);
        for (node, drawable) in twice {
            if self.paint.twice_reported.get_mut().insert((node, drawable)) {
                self.push(HostEvent::DrawableDrawnTwice { node, drawable });
            }
        }
    }
}
