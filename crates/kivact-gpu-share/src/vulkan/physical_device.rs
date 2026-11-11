//! Role: Finds a GpuAdapter's Vulkan physical device and checks it can export.

use super::raw::FORMAT;
use crate::common::{Error, api};
use crate::pool::PoolConfig;
use crate::vulkan_contract::API_VERSION;
use crate::vulkan_contract::{MEMORY_HANDLE, usage};
use ash::vk;
use kivact_types::{AdapterIds, GpuAdapter};

/// Fail unless `physical` exports images of the contract's usage (with
/// storage or not) at `config`'s size.
///
/// # Safety
/// `physical` is a physical device of the live `instance`.
pub(super) unsafe fn check_export(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    config: &PoolConfig,
) -> Result<(), Error> {
    let mut external_info =
        vk::PhysicalDeviceExternalImageFormatInfo::default().handle_type(MEMORY_HANDLE);
    let info = vk::PhysicalDeviceImageFormatInfo2::default()
        .format(FORMAT)
        .ty(vk::ImageType::TYPE_2D)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage(config.storage))
        .push_next(&mut external_info);
    let mut external = vk::ExternalImageFormatProperties::default();
    let which = if config.storage { " with storage" } else { "" };
    let extent = {
        let mut properties = vk::ImageFormatProperties2::default().push_next(&mut external);
        // SAFETY: a physical device of the instance (the caller); valid
        // chains.
        let supported = unsafe {
            instance.get_physical_device_image_format_properties2(physical, &info, &mut properties)
        };
        match supported {
            Ok(()) => {}
            Err(vk::Result::ERROR_FORMAT_NOT_SUPPORTED) => {
                return Err(Error::Mismatch(format!(
                    "the adapter has no shared images{which}"
                )));
            }
            Err(e) => return Err(api("vkGetPhysicalDeviceImageFormatProperties2")(e)),
        }
        properties.image_format_properties.max_extent
    };
    let features = external.external_memory_properties.external_memory_features;
    if !features.contains(vk::ExternalMemoryFeatureFlags::EXPORTABLE) {
        return Err(Error::Mismatch(format!(
            "the adapter cannot export shared images{which}"
        )));
    }
    if config.width > extent.width || config.height > extent.height {
        return Err(Error::Mismatch(format!(
            "the adapter exports images up to {} x {}",
            extent.width, extent.height
        )));
    }
    Ok(())
}

/// The device and driver UUIDs and API version of `physical`.
///
/// # Safety
/// `physical` is a physical device of the live `instance`.
pub(crate) unsafe fn identity(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
) -> ([u8; 16], [u8; 16], u32) {
    // SAFETY: the caller provides a physical device of the live instance.
    let ids = unsafe { crate::vulkan_raw::identity(instance, physical) };
    (ids.device_uuid, ids.driver_uuid, ids.version)
}

/// The UUIDs `adapter` reports for Vulkan.
pub(crate) fn uuids(adapter: &GpuAdapter) -> Result<([u8; 16], [u8; 16]), Error> {
    match adapter.ids {
        AdapterIds::Vulkan {
            device_uuid,
            driver_uuid,
            ..
        } => Ok((device_uuid, driver_uuid)),
        AdapterIds::Dx12 { .. } => Err(Error::Mismatch(format!(
            "the window's device is {:?}, not Vulkan",
            adapter.backend()
        ))),
    }
}

/// The physical device of `instance` that is `adapter` (its device and
/// driver UUIDs) with Vulkan [`API_VERSION`] or later.
///
/// # Safety
/// `instance` is a live instance of API 1.1 or later.
pub unsafe fn find_physical_device(
    instance: &ash::Instance,
    adapter: &GpuAdapter,
) -> Result<vk::PhysicalDevice, Error> {
    let wanted = uuids(adapter)?;
    // SAFETY: a live instance (the caller).
    let physicals = unsafe { instance.enumerate_physical_devices() }
        .map_err(api("vkEnumeratePhysicalDevices"))?;
    for physical in physicals {
        // SAFETY: a physical device of the instance.
        let (device, driver, version) = unsafe { identity(instance, physical) };
        if (device, driver) == wanted && version >= API_VERSION {
            return Ok(physical);
        }
    }
    Err(Error::NoAdapter)
}
