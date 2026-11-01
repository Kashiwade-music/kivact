//! Role: GpuContext's imports of the images external render engines share.

use super::state::GpuContext;
#[cfg(feature = "gpu-interop")]
use crate::error::RenderError;

impl GpuContext {
    /// Import the pool of textures `shared` as the image `id` (replacing
    /// an image imported as `id` before), for scenes drawn with this
    /// context. Fails with [`RenderError::External`] when the textures do
    /// not fit the device (another adapter or graphics API, a size beyond
    /// its limits, handles it refuses) or it cannot import.
    #[cfg(feature = "gpu-interop")]
    pub fn import_external(
        &mut self,
        id: kivact_types::ExternalImageId,
        shared: kivact_types::SharedTextures,
    ) -> Result<(), RenderError> {
        self.check()?;
        let Some(adapter) = &self.interop else {
            return Err(RenderError::External(
                "the GPU device cannot import shared textures".into(),
            ));
        };
        self.external
            .import(&self.device, &self.queue, adapter, id, shared)?;
        self.check()
    }

    /// Drop the image `id` (nothing when it is not imported): its frames
    /// draw nothing from now on, and its release fence is signaled past
    /// every frame of it.
    #[cfg(feature = "gpu-interop")]
    pub fn drop_external(&mut self, id: kivact_types::ExternalImageId) {
        self.external.drop_image(id);
    }

    /// The external frames scenes named that were not drawn since the last
    /// call: not ready within the frame's wait, or no longer held (each
    /// noted once).
    pub fn take_missed_external(&mut self) -> Vec<kivact_types::ExternalFrame> {
        self.external.take_missed()
    }
}
