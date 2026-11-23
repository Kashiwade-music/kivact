//! Role: One window's GPU context on the driver's shared device.
//!
//! The window's frame state starts empty; the device (adapter, pipelines,
//! imports) is shared by every window's context.

use super::export_pool::Exports;
use super::state::GpuContext;
use super::state::GpuDevice;
use super::targets::Pool;
use crate::cache::CacheIndex;
use std::collections::HashMap;
use std::rc::Rc;

impl GpuContext {
    /// A window's context on `gpu`, with no frame drawn yet.
    pub fn new(gpu: Rc<GpuDevice>) -> GpuContext {
        Self {
            gpu,
            cache: CacheIndex::default(),
            pool: Pool::default(),
            kept: HashMap::new(),
            chunks: HashMap::new(),
            shader_failures: Vec::new(),
            exports: Exports::default(),
        }
    }

    /// The device this context draws with.
    pub fn device(&self) -> &Rc<GpuDevice> {
        &self.gpu
    }

    /// Release rebuildable metadata: the previous frame's records, the free
    /// textures and the kept surfaces and chunks they describe. Imports stay.
    pub fn release_caches(&mut self) {
        self.cache = CacheIndex::default();
        self.pool.clear();
        self.chunks.clear();
        self.kept.clear();
    }
}
