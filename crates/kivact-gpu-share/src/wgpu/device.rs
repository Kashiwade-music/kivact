//! Role: Opens a wgpu device on the adapter a window names, able to share textures.

use crate::common::Error;
use kivact_types::{GpuAdapter, GpuBackend};

/// A device and queue on the adapter of `instance` that is `adapter` (its
/// LUID, or its Vulkan device and driver UUIDs), with the window's
/// graphics API and `desc`'s features and limits, able to share textures:
/// a Vulkan device also enables [`crate::vulkan::DEVICE_EXTENSIONS`]
/// (wgpu does not enable the external semaphore ones) and needs Vulkan
/// [`crate::vulkan::API_VERSION`]; a Direct3D 12 device is wgpu's own
/// `request_device` (shared heaps and fences need nothing more).
///
/// In a process where Kivact's window also runs wgpu on Direct3D 12,
/// `D3D12CreateDevice` returns the process's one device of the adapter:
/// both wgpu devices wrap the same `ID3D12Device`, each with its own
/// queue. Sharing works on it (the window opens the handles on the device
/// that made them); a lost device is lost for both. Make `instance`
/// without `wgpu::InstanceFlags::DEBUG` and `VALIDATION` there (as Arvane
/// does: `InstanceFlags::empty().with_env()`): enabling Direct3D 12's
/// debug layer once a device exists removes that device, the window's
/// too. `InstanceDescriptor`'s defaults turn them on in a debug build.
pub fn request_device(
    instance: &::wgpu::Instance,
    adapter: &GpuAdapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Result<(::wgpu::Adapter, ::wgpu::Device, ::wgpu::Queue), Error> {
    let backends = match adapter.backend {
        GpuBackend::Dx12 => ::wgpu::Backends::DX12,
        GpuBackend::Vulkan => ::wgpu::Backends::VULKAN,
    };
    for found in pollster::block_on(instance.enumerate_adapters(backends)) {
        match adapter.backend {
            #[cfg(windows)]
            GpuBackend::Dx12 if super::d3d12::is_adapter(&found, adapter)? => {
                return super::d3d12::open(found, desc);
            }
            GpuBackend::Vulkan if super::vulkan::is_adapter(&found, adapter)? => {
                return super::vulkan::open(found, desc);
            }
            _ => {}
        }
    }
    Err(Error::NoAdapter)
}
