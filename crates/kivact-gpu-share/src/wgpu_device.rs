//! Role: Opens wgpu Vulkan devices with the shared-texture extensions.

#![allow(
    unsafe_code,
    reason = "wgpu hal device creation with supported sharing extensions"
)]

use crate::common::{Error, api};
use crate::vulkan_contract::{API_VERSION, DEVICE_EXTENSIONS};
use ::wgpu::hal::api::Vulkan;

pub(crate) fn open(
    adapter: &::wgpu::Adapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Result<Option<(::wgpu::Device, ::wgpu::Queue)>, Error> {
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
