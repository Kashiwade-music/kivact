//! Role: Paint one node's scene: reuse its cached lowering or lower each command.
//!
//! Invariants: a scene with a child layer or an item is not cached: what it
//! lowers to is in the composition's layers, not a plain command list (§6.7).

use super::state::State;
use crate::lower::SceneOut;
use crate::lower::context::Context;
use crate::lower::effects::Tint;
use crate::lower::scene_cache::Entry;
use kivact_types::render::RenderCmd as D;
use kivact_types::{ComputedTextStyle, DrawList, Rect};
use std::sync::Arc;

impl Context<'_> {
    /// Paint a local scene at its canvas origin, inheriting missing text fields.
    pub(crate) fn paint_scene(
        &self,
        scene: &Arc<DrawList>,
        inherited: Option<&ComputedTextStyle>,
        origin: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut SceneOut,
    ) {
        let (layer, item) = super::scene_layers(scene);
        let cacheable = !layer && !item;

        if !cacheable || !origin.x.is_finite() || !origin.y.is_finite() {
            self.lower_scene(scene, inherited, origin, tint, engine, out);
            return;
        }
        let raster = self.raster.get().to_bits();
        // Drop the cache borrow before refreshing resource usage.
        let prefix = out.open();
        let cached = self
            .scene_cache
            .borrow_mut()
            .get(scene, inherited, raster, tint);
        if let Some(entry) = cached {
            if prefix + entry.max_depth + 1 > arvane::MAX_SCENE_DEPTH {
                self.lower_scene(scene, inherited, origin, tint, engine, out);
                return;
            }
            if self.resources_match(&entry.dependencies) {
                append(out, &entry.lowered.commands, origin);
                return;
            }
        }
        let checkpoint = self.checkpoint();
        let mut lowered = SceneOut::default();
        let mark = self.dependencies.borrow().len();
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
        let dependencies = self.dependencies.borrow()[mark..].to_vec();
        let max_depth = lowered.deepest();
        let lowered = lowered.into_scene();
        if prefix + max_depth + 1 > arvane::MAX_SCENE_DEPTH {
            self.restore(checkpoint);
            self.lower_scene(scene, inherited, origin, tint, engine, out);
            return;
        }
        append(out, &lowered.commands, origin);
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
        scene: &DrawList,
        inherited: Option<&ComputedTextStyle>,
        origin: Rect,
        tint: &Tint,
        engine: &mut arvane::Engine,
        out: &mut crate::lower::SceneOut,
    ) {
        let raster = self.raster.get();
        let mut state = State::new(inherited, engine, out, origin, *tint);
        for (index, cmd) in scene.cmds.iter().enumerate() {
            let rest = &scene.cmds[index + 1..];
            if let kivact_types::DrawCmd::Drawable { id, at } = cmd {
                self.paint_drawable(*id, *at, index as u32, &mut state);
                continue;
            }
            let _ = self.paint_raster(cmd, &mut state)
                || self.paint_layers(cmd, rest, &mut state)
                || self.paint_geometry(cmd, &mut state)
                || self.paint_text(cmd, &mut state);
        }
        state.finish();
        self.raster.set(raster);
    }
}

fn append(out: &mut SceneOut, local: &[D], origin: Rect) {
    out.push(D::PushTransform(kivact_types::Transform {
        e: origin.x,
        f: origin.y,
        ..kivact_types::Transform::IDENTITY
    }));
    out.extend(local.iter().cloned());
    out.push(D::PopTransform);
}
