//! Role: GpuContext's imports of the images external render engines share.

use super::state::GpuContext;
use crate::ComposeError;

impl GpuContext {
    /// Import the pool of textures `shared` as the image `id` (replacing an image imported as
    /// `id` before), for frames drawn with this context. Fails when the textures do not fit the
    /// device (another adapter or graphics API, a size beyond its limits, handles it refuses) or
    /// it cannot import.
    #[cfg(feature = "gpu-interop")]
    pub fn import_external(
        &mut self,
        id: kivact_types::ExternalImageId,
        shared: kivact_types::SharedTextures,
    ) -> Result<(), ComposeError> {
        self.check()?;
        let Some(adapter) = &self.interop else {
            return Err(ComposeError::Backend(
                "the GPU device cannot import shared textures".into(),
            ));
        };
        self.external
            .import(&self.device, &self.queue, adapter, id, shared)?;
        self.check()
    }

    /// Drop the image `id` (nothing when it is not imported): its frames draw nothing from now
    /// on, and its release fence is signaled past every frame of it.
    #[cfg(feature = "gpu-interop")]
    pub fn drop_external(&mut self, id: kivact_types::ExternalImageId) {
        self.external.drop_image(id);
    }

    /// Without the `gpu-interop` feature no image can be imported.
    #[cfg(not(feature = "gpu-interop"))]
    pub fn import_external(
        &mut self,
        _: kivact_types::ExternalImageId,
        _: kivact_types::SharedTextures,
    ) -> Result<(), ComposeError> {
        Err(ComposeError::Backend(
            "built without the gpu-interop feature".into(),
        ))
    }

    /// Without the `gpu-interop` feature there is no image to drop.
    #[cfg(not(feature = "gpu-interop"))]
    pub fn drop_external(&mut self, _: kivact_types::ExternalImageId) {}

    /// The external frames that frames drew but could not draw since the last call (each noted
    /// once): not ready within the frame's wait, or no longer held.
    pub fn take_missed_external(&mut self) -> Vec<kivact_types::ExternalFrame> {
        self.external.take_missed()
    }
}
