//! Role: Opens the devices that import and export shared textures.
//!
//! The import side (`import::Import`) and the export side (the wgpu pools)
//! both need the same device: the Vulkan extensions both sides need are
//! asked for here, so this module is the one path both sides name.

#![allow(
    unsafe_code,
    reason = "wgpu hal device creation with supported sharing extensions"
)]

use crate::common::{Error, api};
#[cfg(any(windows, target_os = "linux"))]
use ::wgpu::hal::api::Vulkan;
#[cfg(feature = "wgpu")]
use kivact_types::{GpuAdapter, GpuBackend};

/// A device opened with the shared-texture extensions when available.
pub struct Opened {
    /// The device.
    pub device: ::wgpu::Device,
    /// Its queue.
    pub queue: ::wgpu::Queue,
    /// Whether shared textures can be imported.
    pub interop: bool,
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
    if let Some(opened) = open_vulkan(adapter, desc).transpose() {
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
#[cfg(all(feature = "wgpu", any(windows, target_os = "linux")))]
pub fn request_device_for(
    instance: &::wgpu::Instance,
    adapter: &GpuAdapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Result<(::wgpu::Adapter, ::wgpu::Device, ::wgpu::Queue), Error> {
    let backends = match adapter.backend() {
        GpuBackend::Dx12 => ::wgpu::Backends::DX12,
        GpuBackend::Vulkan => ::wgpu::Backends::VULKAN,
    };
    for found in pollster::block_on(instance.enumerate_adapters(backends)) {
        match adapter.backend() {
            #[cfg(windows)]
            GpuBackend::Dx12 if crate::wgpu::d3d12::is_adapter(&found, adapter)? => {
                return crate::wgpu::d3d12::open(found, desc);
            }
            GpuBackend::Vulkan if crate::wgpu::vulkan::is_adapter(&found, adapter)? => {
                return crate::wgpu::vulkan::open(found, desc);
            }
            _ => {}
        }
    }
    Err(Error::NoAdapter)
}

/// A Vulkan device of `adapter` with the sharing's extensions and `desc`'s
/// features and limits, or `None` when the adapter cannot share (the
/// caller opens an ordinary device).
#[cfg(any(windows, target_os = "linux"))]
pub(crate) fn open_vulkan(
    adapter: &::wgpu::Adapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Result<Option<(::wgpu::Device, ::wgpu::Queue)>, Error> {
    use crate::vulkan::{API_VERSION, DEVICE_EXTENSIONS};
    let open = {
        // SAFETY: the guard is dropped before the device is made from it.
        let Some(hal) = (unsafe { adapter.as_hal::<Vulkan>() }) else {
            return Ok(None);
        };
        let capabilities = hal.physical_device_capabilities();
        if capabilities.properties().api_version < API_VERSION
            || hal.shared_instance().instance_api_version() < API_VERSION
        {
            return Ok(None);
        }
        if !DEVICE_EXTENSIONS
            .iter()
            .all(|extension| capabilities.supports_extension(extension))
        {
            return Ok(None);
        }
        let callback = Box::new(
            |args: ::wgpu::hal::vulkan::CreateDeviceCallbackArgs<'_, '_, '_>| {
                for extension in DEVICE_EXTENSIONS {
                    if !args.extensions.contains(&extension) {
                        args.extensions.push(extension);
                    }
                }
            },
        );
        // SAFETY: the callback only adds extensions the adapter supports
        // (checked above) and removes nothing.
        unsafe {
            hal.open_with_callback(
                desc.required_features,
                &desc.required_limits,
                &desc.memory_hints,
                Some(callback),
            )
        }
        .map_err(api("open_with_callback"))?
    };
    // SAFETY: `open` was opened from this adapter with `desc`'s features and
    // limits.
    let (device, queue) = unsafe { adapter.create_device_from_hal::<Vulkan>(open, desc) }
        .map_err(api("create_device_from_hal"))?;
    Ok(Some((device, queue)))
}
