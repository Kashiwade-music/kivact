//! Role: The GPU device's imports of externally shared images.

use super::device::GpuDevice;
use crate::RenderError;

impl GpuDevice {
    /// Import the pool of textures `shared` as the image `id` (replacing an image imported as
    /// `id` before), for frames drawn on this device. Fails when the textures do not fit the
    /// device (another adapter or graphics API, a size beyond its limits, handles it refuses) or
    /// it cannot import.
    #[cfg(feature = "gpu-interop")]
    pub fn import_external(
        &self,
        id: kivact_types::ExternalImageId,
        shared: kivact_types::SharedTextures,
    ) -> Result<(), RenderError> {
        self.check()?;
        let Some(adapter) = &self.interop else {
            return Err(RenderError::Backend(
                "the GPU device cannot import shared textures".into(),
            ));
        };
        self.external
            .borrow_mut()
            .import(&self.device, &self.queue, adapter, id, shared)?;
        self.check()
    }

    /// Drop the image `id` (nothing when it is not imported): its frames draw nothing from now
    /// on, and its release fence is signaled past every frame of it.
    #[cfg(feature = "gpu-interop")]
    pub fn drop_external(&self, id: kivact_types::ExternalImageId) {
        self.external.borrow_mut().drop_image(id);
    }

    /// Without the `gpu-interop` feature no image can be imported.
    #[cfg(not(feature = "gpu-interop"))]
    pub fn import_external(
        &self,
        _: kivact_types::ExternalImageId,
        _: kivact_types::SharedTextures,
    ) -> Result<(), RenderError> {
        Err(RenderError::Backend(
            "built without the gpu-interop feature".into(),
        ))
    }

    /// Without the `gpu-interop` feature there is no image to drop.
    #[cfg(not(feature = "gpu-interop"))]
    pub fn drop_external(&self, _: kivact_types::ExternalImageId) {}

    /// The external frames that frames drew but could not draw since the last call (each noted
    /// once): not ready within the frame's wait, or no longer held.
    pub fn take_missed_external(&self) -> Vec<kivact_types::ExternalFrame> {
        self.external.borrow_mut().take_missed()
    }
}
