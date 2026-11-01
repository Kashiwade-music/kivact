//! Role: Bounded retained local scene payloads and resource dependency data.
use crate::effects::Tint;
use arvane::asset::{PixelSize, Region};
use kivact_wire::{Asset, ComputedText, Rgba, Scene, Size};
use std::sync::{Arc, Weak};

/// Rebuildable local-scene lowering owned by one host or offscreen renderer.
/// It retains at most 128 successful local payloads and never shares between hosts.
/// Clear it after directly registering a font with the borrowed Arvane engine;
/// registration through `ArvaneAssets` invalidates it automatically.
#[derive(Default)]
pub struct SceneCache {
    entries: Vec<Arc<Entry>>,
    revision: u64,
    cursor: usize,
    open: usize,
}
impl SceneCache {
    /// Create an empty local scene cache.
    pub fn new() -> Self {
        Self::default()
    }
    /// Start a fresh append-only output scene while retaining reusable entries.
    /// Call before using a new, replaced or mutated output with Context.
    pub fn begin_frame(&mut self) {
        self.cursor = 0;
        self.open = 0;
    }

    pub(crate) fn prefix_depth(&mut self, out: &ui_types::Scene) -> usize {
        for cmd in &out.commands[self.cursor..] {
            step(cmd, &mut self.open);
        }
        self.cursor = out.commands.len();
        self.open
    }

    /// Release rebuildable payloads without changing live resource owners.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub(crate) fn revision(&mut self, revision: u64) {
        if self.revision != revision {
            self.clear();
            self.revision = revision;
        }
    }
    pub(crate) fn get(
        &mut self,
        scene: &Arc<Scene>,
        inherited: Option<&ComputedText>,
        raster: u32,
        tint: &Tint,
    ) -> Option<Arc<Entry>> {
        self.entries.retain(|entry| entry.scene.strong_count() != 0);
        let index = self.entries.iter().position(|entry| {
            entry.scene.ptr_eq(&Arc::downgrade(scene))
                && entry.len == scene.cmds.len()
                && entry.inherited.as_ref() == inherited
                && entry.raster == raster
                && entry.tint == *tint
        })?;
        let entry = self.entries.remove(index);
        self.entries.push(Arc::clone(&entry));
        Some(entry)
    }
    pub(crate) fn insert(&mut self, entry: Entry) {
        self.entries.retain(|old| {
            old.scene.strong_count() != 0
                && !(old.scene.ptr_eq(&entry.scene)
                    && old.len == entry.len
                    && old.inherited == entry.inherited
                    && old.raster == entry.raster
                    && old.tint == entry.tint)
        });
        if self.entries.len() == 128 {
            self.entries.remove(0);
        }
        self.entries.push(Arc::new(entry));
    }
}
pub(crate) struct Entry {
    pub(crate) scene: Weak<Scene>,
    pub(crate) len: usize,
    pub(crate) inherited: Option<ComputedText>,
    pub(crate) raster: u32,
    pub(crate) tint: Tint,
    pub(crate) lowered: ui_types::Scene,
    pub(crate) max_depth: usize,
    pub(crate) dependencies: Vec<Dependency>,
}
#[derive(Clone)]
pub(crate) enum Dependency {
    Natural {
        asset: Asset,
        size: Size,
    },
    Pixels {
        asset: Asset,
        device: PixelSize,
        region: Region,
        tint: Option<Rgba>,
        width: u32,
        height: u32,
        pixels: Weak<[u8]>,
    },
}

fn step(command: &ui_types::DrawCommand, depth: &mut usize) {
    use ui_types::DrawCommand as D;
    match command {
        D::PushClip(_) | D::PushTransform(_) | D::PushLayer(_) => *depth += 1,
        D::PopClip | D::PopTransform | D::PopLayer => *depth = depth.saturating_sub(1),
        _ => {}
    }
}
pub(crate) fn payload_depth(scene: &ui_types::Scene) -> usize {
    let (mut depth, mut max) = (0, 0);
    for cmd in &scene.commands {
        step(cmd, &mut depth);
        max = max.max(depth);
    }
    max
}
