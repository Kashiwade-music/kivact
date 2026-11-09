//! Role: Bounded retained local scene payloads and resource dependency data.
use crate::lower::effects::Tint;
use arvane::asset::{PixelSize, Region};
use kivact_types::{Asset, ComputedTextStyle, DrawList, Rgba, Size};
use std::sync::{Arc, Weak};

/// Rebuildable local-scene lowering owned by one host or offscreen renderer.
/// It retains at most 128 successful local payloads and never shares between hosts.
/// Clear it after directly registering a font with the borrowed Arvane engine;
/// registration through `ArvaneAssets` invalidates it automatically.
#[derive(Default)]
pub(crate) struct SceneCache {
    entries: Vec<Arc<Entry>>,
    revision: kivact_types::cell::Generation,
}
impl SceneCache {
    /// Release rebuildable payloads without changing live resource owners.
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
    pub(crate) fn revision(&mut self, revision: kivact_types::cell::Generation) {
        if self.revision != revision {
            self.clear();
            self.revision = revision;
        }
    }
    pub(crate) fn get(
        &mut self,
        scene: &Arc<DrawList>,
        inherited: Option<&ComputedTextStyle>,
        raster: u64,
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
    pub(crate) scene: Weak<DrawList>,
    pub(crate) len: usize,
    pub(crate) inherited: Option<ComputedTextStyle>,
    pub(crate) raster: u64,
    pub(crate) tint: Tint,
    pub(crate) lowered: kivact_types::render::RenderScene,
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
        frame: u32,
        width: u32,
        height: u32,
        pixels: Weak<[u8]>,
    },
}
