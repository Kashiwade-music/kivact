//! Role: The exports and GPU pools of drawables, and the reconcile of their roots.
//!
//! Invariants:
//! - A GPU export waits for its pool of the same generation before it is
//!   composed; the pool is sized from the box at the scale the app asked.

use kivact_types::{DrawableId, NodeId};

use crate::contract::{DrawableMode, ExportRequestMode};
use crate::model::Model;
use crate::model::drawables::state::ExportAsk;

/// A GPU export pool a drawable wants this frame.
pub(crate) struct WantedPool {
    /// The drawable.
    pub(crate) id: DrawableId,
    /// The app's request generation the pool serves.
    pub(crate) generation: u64,
    /// The window's adapter the textures are shared on.
    pub(crate) adapter: kivact_types::GpuAdapter,
    /// The pool's size in device px.
    pub(crate) size: (u32, u32),
}

/// The export of an exported drawable's layer `layer`, sized from its box
/// (`bounds`, logical px) at `scale` device px per logical px.
pub(crate) fn export_of(
    sink: DrawableId,
    layer: u32,
    bounds: kivact_types::Rect,
    scale: f64,
    ask: &ExportAsk,
) -> kivact_types::render::Export {
    let px = |v: f64| (v * scale).ceil().max(1.0) as u32;
    kivact_types::render::Export {
        layer,
        sink,
        size: (px(bounds.width), px(bounds.height)),
        scale,
        generation: ask.generation,
        mode: match ask.mode {
            ExportRequestMode::Gpu(_) => kivact_types::render::ExportMode::Gpu,
            _ => kivact_types::render::ExportMode::Pixels,
        },
    }
}

impl Model {
    /// The exports to compose this frame: each exported drawable the app asked
    /// frames of, with its root node and request. A GPU request waits for its
    /// pool (`pool_of`) of the same generation.
    pub(crate) fn export_requests(&self) -> Vec<(DrawableId, NodeId, ExportAsk)> {
        self.drawables
            .iter()
            .filter(|(_, entry)| entry.mode == DrawableMode::Export)
            .filter_map(|(id, entry)| {
                let ask = self.drawables.ask_of(id)?.clone();
                let ready = match ask.mode {
                    ExportRequestMode::None => false,
                    ExportRequestMode::Pixels => true,
                    ExportRequestMode::Gpu(_) => self
                        .drawables
                        .pool_of(id)
                        .is_some_and(|(generation, _)| generation == ask.generation),
                };
                ready.then_some((id, entry.root, ask))
            })
            .collect()
    }

    /// The GPU export pools wanted now: every mounted exported drawable whose
    /// app asked for textures, sized from its laid-out box in device px.
    pub(crate) fn wanted_pools(&self) -> Vec<WantedPool> {
        self.drawables
            .iter()
            .filter(|(_, entry)| entry.mode == DrawableMode::Export)
            .filter_map(|(id, entry)| {
                let ask = self.drawables.ask_of(id)?;
                let ExportRequestMode::Gpu(adapter) = &ask.mode else {
                    return None;
                };
                let rect = self.rect(entry.root)?;
                let scale = ask.scale_at(self.paint.scale());
                let px = |v: f64| (v * scale).ceil().max(1.0) as u32;
                Some(WantedPool {
                    id,
                    generation: ask.generation,
                    adapter: adapter.clone(),
                    size: (px(rect.width), px(rect.height)),
                })
            })
            .collect()
    }

    /// Make the entries follow the roles written since the last commit, and
    /// drop those whose root left the tree.
    pub(crate) fn reconcile_drawables(&mut self) {
        for node in self.drawables.take_noted() {
            let role = self
                .tree
                .get(node)
                .and_then(|n| n.properties.drawable.map(|r| (r.id, r.mode)));
            self.drawables.forget_root(node);
            if let Some((id, mode)) = role {
                self.drawables.follow(id, node, mode);
            }
        }
        let tree = &self.tree;
        self.drawables.retain_roots(|root| tree.get(root).is_some());
    }
}
