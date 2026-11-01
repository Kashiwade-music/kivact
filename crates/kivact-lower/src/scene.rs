//! Role: Flattening of node-local canvas scenes into global draw commands.
//!
//! Flatten node-local canvas scenes into global commands at their node's
//! canvas origin, preserving paint order and alpha (scenes keep their size
//! under a transform).
//!
//! Invalid commands are dropped and clips are balanced within each node's scene.
//! A text run uses its own family and line height, else its node's inherited
//! ones (resolved like a text's), its italic, letter spacing, features and decoration
//! lines (drawn like a text's: `glyphs`); an image fills its rect with the asset
//! layer's pixels for the rect × the frame's scale (nothing while loading
//! or failed; the failure goes to the painting [`Context`], which a window
//! ignores).
//!
//! Invariants:
//! - A transform (kivact's `Layer::transform`, already a matrix for the
//!   canvas's size) is pushed as Arvane's `PushTransform` about the canvas
//!   origin, never flattened here, so it applies alike to text, images,
//!   clips, fills, strokes and shadows; under it images are rasterized for
//!   its zoom too (`raster_zoom`). A layer opens as the node's own do
//!   (`effects`: clip, mask, opacity, blend, filters); its color filter
//!   tints what it holds.
//! - A rect clip (`SceneCmd::PushClip`, kivact's `Layer::clip_rect`) opens
//!   no group: it is Arvane's `PushClip`. A layer's clip shape (kivact's
//!   `Layer::clip(&Shape)`, placed in its area) is the clip of the layer's
//!   group.
//! - Pixels (`SceneCmd::Pixels`: straight-alpha RGBA8) are drawn as they
//!   are, through neither the asset store nor its conversion. A part of an
//!   image (`SceneCmd::ImagePart`) is drawn from its source rect, copying no
//!   pixel, rasterized for the destination's device pixels enlarged by how
//!   much the source rect is (an SVG stays sharp, within the store's cache
//!   limit); both are drawn at their opacity, sampled nearest-neighbour
//!   when pixelated.
//! - An external frame (`SceneCmd::External`) is, with feature
//!   `gpu-interop`, Arvane's `DrawCommand::External` at its rect and
//!   opacity: drawn only by the GPU of the window its image was imported
//!   into (without the feature nothing is drawn and no command counts as
//!   dropped). A layer's
//!   color filter does not reach it (Arvane draws it without a color
//!   matrix); one with a rect or source that is not finite, or an opacity
//!   that is not, is dropped.
//! - A scene passes its neutral asset sources directly to the asset store.
//! - A text run with a size or origin that is not finite, or a negative
//!   size, is dropped; one of size 0 draws nothing.
//! - A paragraph (`SceneCmd::Paragraph`) draws the glyph runs and decoration
//!   lines it was shaped with, in its color, at its origin; one with an
//!   origin that is not finite is dropped.

mod geometry;
mod layers;
mod raster;
mod state;
mod text;
use crate::context::Context;
use crate::effects::Tint;
use crate::scene_cache::Entry;
use kivact_wire::{ComputedText, Rect, Scene};
use std::sync::Arc;
use ui_types::DrawCommand as D;
impl Context<'_> {
    /// Paint a local scene at its canvas origin, inheriting missing text fields.
    pub fn paint_scene(
        &self,
        scene: &Arc<Scene>,
        inherited: Option<&ComputedText>,
        origin: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut ui_types::Scene,
    ) {
        #[cfg(feature = "gpu-interop")]
        let cacheable = !scene
            .cmds
            .iter()
            .any(|cmd| matches!(cmd, kivact_wire::SceneCmd::External { .. }));
        #[cfg(not(feature = "gpu-interop"))]
        let cacheable = true;
        if !cacheable || !origin.x.is_finite() || !origin.y.is_finite() {
            self.lower_scene(scene, inherited, origin, tint, engine, out);
            return;
        }
        let raster = self.raster.get().to_bits();
        // Drop the cache borrow before refreshing resource usage.
        let (cached, prefix) = {
            let mut cache = self.scene_cache.borrow_mut();
            let prefix = cache.prefix_depth(out);
            (cache.get(scene, inherited, raster, tint), prefix)
        };
        if let Some(entry) = cached {
            if prefix + entry.max_depth + 1 > arvane::MAX_SCENE_DEPTH {
                self.lower_scene(scene, inherited, origin, tint, engine, out);
                return;
            }
            if self.resources_match(&entry.dependencies) {
                append(out, &entry.lowered, origin);
                return;
            }
        }
        let checkpoint = self.checkpoint();
        let mut lowered = ui_types::Scene::default();
        *self.dependencies.borrow_mut() = Some(Vec::new());
        self.lower_scene(
            scene,
            inherited,
            Rect {
                x: 0.0,
                y: 0.0,
                ..origin
            },
            tint,
            engine,
            &mut lowered,
        );
        let dependencies = self
            .dependencies
            .borrow_mut()
            .take()
            .expect("invariant: scene lowering records dependencies");
        let max_depth = crate::scene_cache::payload_depth(&lowered);
        if prefix + max_depth + 1 > arvane::MAX_SCENE_DEPTH {
            self.restore(checkpoint);
            self.lower_scene(scene, inherited, origin, tint, engine, out);
            return;
        }
        append(out, &lowered, origin);
        if self.succeeded_since(&checkpoint) {
            self.scene_cache.borrow_mut().insert(Entry {
                scene: Arc::downgrade(scene),
                len: scene.cmds.len(),
                inherited: inherited.cloned(),
                raster,
                tint: *tint,
                lowered,
                max_depth,
                dependencies,
            });
        }
    }

    fn lower_scene(
        &self,
        scene: &Scene,
        inherited: Option<&ComputedText>,
        origin: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut ui_types::Scene,
    ) {
        let raster = self.raster.get();
        let mut state = state::State::new(inherited, engine, out, origin, *tint);
        for cmd in &scene.cmds {
            let _ = self.paint_raster(cmd, &mut state)
                || self.paint_layers(cmd, &mut state)
                || self.paint_geometry(cmd, &mut state)
                || self.paint_text(cmd, &mut state);
        }
        state.finish();
        self.raster.set(raster);
    }
}

fn append(out: &mut ui_types::Scene, local: &ui_types::Scene, origin: Rect) {
    out.commands.push(D::PushTransform(ui_types::Transform {
        e: origin.x,
        f: origin.y,
        ..ui_types::Transform::IDENTITY
    }));
    out.commands.extend(local.commands.iter().cloned());
    out.commands.push(D::PopTransform);
}

/// The zoom images are rasterized for under transforms enlarging them by
/// `z`: `z` rounded up to a quarter step of powers of two (exactly 1
/// without a transform), so a zooming subtree rasterizes its SVGs again
/// only every 19 %, never blurrier than drawn.
pub fn raster_zoom(z: f32) -> f32 {
    if z == 1.0 || !z.is_finite() || z <= 0.0 {
        return 1.0;
    }
    ((z.log2() * 4.0).ceil() / 4.0).exp2()
}
