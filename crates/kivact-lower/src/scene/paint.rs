//! Role: Paint one node's scene: reuse its cached lowering or lower each command.

use super::state::State;
use crate::context::Context;
use crate::effects::Tint;
use crate::scene_cache::Entry;
use kivact_types::DrawCommand as D;
use kivact_wire::{ComputedText, Rect, Scene};
use std::sync::Arc;

impl Context<'_> {
    /// Paint a local scene at its canvas origin, inheriting missing text fields.
    pub fn paint_scene(
        &self,
        scene: &Arc<Scene>,
        inherited: Option<&ComputedText>,
        origin: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut kivact_types::Scene,
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
        let mut lowered = kivact_types::Scene::default();
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
        out: &mut kivact_types::Scene,
    ) {
        let raster = self.raster.get();
        let mut state = State::new(inherited, engine, out, origin, *tint);
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

fn append(out: &mut kivact_types::Scene, local: &kivact_types::Scene, origin: Rect) {
    out.commands.push(D::PushTransform(kivact_types::Transform {
        e: origin.x,
        f: origin.y,
        ..kivact_types::Transform::IDENTITY
    }));
    out.commands.extend(local.commands.iter().cloned());
    out.commands.push(D::PopTransform);
}
