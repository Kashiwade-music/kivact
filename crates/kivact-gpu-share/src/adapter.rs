//! Role: The identity of a wgpu adapter: its DXGI LUID, or its Vulkan UUIDs.
//!
//! Without the import or the export: an engine that only names the adapter
//! a window reports (`GpuPreference::adapter`) needs no pool or native
//! handle code, and the windows' default build compiles none of it.

#![allow(
    unsafe_code,
    reason = "reading an adapter's handles through wgpu's as_hal, below its safe API"
)]

#[cfg(windows)]
use ::wgpu::hal::api::Dx12;
#[cfg(any(windows, target_os = "linux"))]
use ::wgpu::hal::api::Vulkan;
use kivact_types::AdapterIds;

/// The identity of `adapter` (`None`: neither Direct3D 12 nor Vulkan, or
/// unreadable).
pub fn identity(adapter: &::wgpu::Adapter) -> Option<AdapterIds> {
    #[cfg(windows)]
    if let Some(identity) = dx12(adapter) {
        return Some(identity);
    }
    #[cfg(any(windows, target_os = "linux"))]
    if let Some(identity) = vulkan(adapter) {
        return Some(identity);
    }
    let _ = adapter;
    None
}

/// The identity of a Direct3D 12 adapter: its LUID.
#[cfg(windows)]
fn dx12(adapter: &::wgpu::Adapter) -> Option<AdapterIds> {
    // SAFETY: only reads the adapter's description while the guard lives.
    let hal = unsafe { adapter.as_hal::<Dx12>() }?;
    // SAFETY: a live DXGI adapter.
    let desc = unsafe { hal.raw_adapter().GetDesc2() }.ok()?;
    let luid = crate::d3d12::luid_bytes(desc.AdapterLuid);
    Some(AdapterIds::Dx12 { luid })
}

/// The identity of a Vulkan adapter: its device and driver UUIDs and LUID.
#[cfg(any(windows, target_os = "linux"))]
fn vulkan(adapter: &::wgpu::Adapter) -> Option<AdapterIds> {
    // SAFETY: only reads the adapter's handles while the guard lives.
    let hal = unsafe { adapter.as_hal::<Vulkan>() }?;
    let instance = hal.shared_instance().raw_instance();
    // SAFETY: the physical device belongs to the live instance retained by the guard.
    let ids = unsafe { crate::vulkan::raw::identity(instance, hal.raw_physical_device()) };
    Some(AdapterIds::Vulkan {
        device_uuid: ids.device_uuid,
        driver_uuid: ids.driver_uuid,
        luid: ids.luid,
    })
}
