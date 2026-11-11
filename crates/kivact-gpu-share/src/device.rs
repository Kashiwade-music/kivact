//! Role: Opens the devices that import and export shared textures.
//!
//! The import side (`import::Import`) and the export side (the wgpu pools)
//! both need the same device: the Vulkan extensions both sides need are
//! asked for here, so this module is the one path both sides name.

use crate::common::{Error, api};
use kivact_types::AdapterIds;

/// A device opened with the shared-texture extensions when available.
pub struct Opened {
    /// The device.
    pub device: ::wgpu::Device,
    /// Its queue.
    pub queue: ::wgpu::Queue,
    /// Whether shared textures can be imported.
    pub interop: bool,
}

/// The identity of `adapter` (`None`: neither Direct3D 12 nor Vulkan, or
/// unreadable).
pub fn identity(adapter: &::wgpu::Adapter) -> Option<AdapterIds> {
    #[cfg(windows)]
    if let Some(identity) = crate::import::dx12::identity(adapter) {
        return Some(identity);
    }
    #[cfg(any(windows, target_os = "linux"))]
    if let Some(identity) = crate::import::vulkan::identity(adapter) {
        return Some(identity);
    }
    let _ = adapter;
    None
}

/// `adapter.request_device(desc)`, able to import shared textures where
/// the adapter can: a Vulkan device gets the external memory and
/// semaphore extensions (and needs Vulkan 1.2's timeline semaphores); an
/// adapter without them gets an ordinary device that cannot import.
pub fn request_device(
    adapter: &::wgpu::Adapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Result<Opened, Error> {
    #[cfg(any(windows, target_os = "linux"))]
    if let Some(opened) = crate::import::vulkan::request_device(adapter, desc) {
        let (device, queue) = opened?;
        return Ok(Opened {
            device,
            queue,
            interop: true,
        });
    }
    let (device, queue) =
        pollster::block_on(adapter.request_device(desc)).map_err(api("request_device"))?;
    let interop = cfg!(windows) && adapter.get_info().backend == ::wgpu::Backend::Dx12;
    Ok(Opened {
        device,
        queue,
        interop,
    })
}
