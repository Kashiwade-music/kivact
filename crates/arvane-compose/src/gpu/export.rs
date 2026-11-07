//! Role: The GPU export pools of drawables (design §8.12). Not built yet: an export is read back as pixels.

use super::state::GpuContext;
use crate::ComposeError;
use kivact_types::ids::DrawableId;
use kivact_types::{GpuAdapter, SharedTextures};

impl GpuContext {
    /// The export pool of `sink` for request `generation` on `adapter`, as shared textures.
    /// The pool is not made by this executor yet, so it always fails.
    pub fn export_drawable(
        &mut self,
        sink: DrawableId,
        generation: u64,
        adapter: &GpuAdapter,
        size: (u32, u32),
    ) -> Result<SharedTextures, ComposeError> {
        let _ = (sink, generation, adapter, size);
        Err(ComposeError::Backend(
            "drawable export pools are not built by this executor".into(),
        ))
    }

    /// Drop the export pool of `sink`; there is none, so this does nothing.
    pub fn drop_export(&mut self, sink: DrawableId) {
        let _ = sink;
    }
}
