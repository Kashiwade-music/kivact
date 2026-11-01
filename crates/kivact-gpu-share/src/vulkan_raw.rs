//! Role: Shared Vulkan barriers, memory selection and timeline construction.

#![allow(
    unsafe_code,
    reason = "recording barriers and reading memory types through ash"
)]

use ash::vk;

pub(crate) const COLOR: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::COLOR,
    base_mip_level: 0,
    level_count: 1,
    base_array_layer: 0,
    layer_count: 1,
};

/// The memory type for `bits` with `flags`: the lowest index.
///
/// # Safety
/// `physical` is a physical device of the live `instance`.
pub(crate) unsafe fn memory_type(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    bits: u32,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    // SAFETY: a physical device of the instance (the caller).
    let properties = unsafe { instance.get_physical_device_memory_properties(physical) };
    (0..properties.memory_type_count).find(|&i| {
        bits & (1 << i) != 0
            && properties.memory_types[i as usize]
                .property_flags
                .contains(flags)
    })
}

/// Record one image barrier into `command`.
///
/// # Safety
/// `command` is a recording command buffer of `device`; `image` is an
/// image of `device`.
pub(crate) unsafe fn barrier(
    device: &ash::Device,
    command: vk::CommandBuffer,
    image: vk::Image,
    (old, new): (vk::ImageLayout, vk::ImageLayout),
    (from, to): (u32, u32),
    (src_stage, src_access): (vk::PipelineStageFlags, vk::AccessFlags),
    (dst_stage, dst_access): (vk::PipelineStageFlags, vk::AccessFlags),
) {
    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old)
        .new_layout(new)
        .src_queue_family_index(from)
        .dst_queue_family_index(to)
        .src_access_mask(src_access)
        .dst_access_mask(dst_access)
        .image(image)
        .subresource_range(COLOR);
    // SAFETY: recording into a recording buffer (the caller).
    unsafe {
        device.cmd_pipeline_barrier(
            command,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }
}

/// Make a timeline at zero, optionally exportable with the contract handle type.
///
/// # Safety
/// The device supports Vulkan 1.2 timeline semaphores and sharing extensions.
pub(crate) unsafe fn timeline(
    device: &ash::Device,
    external: bool,
) -> Result<vk::Semaphore, crate::common::Error> {
    let mut kind = vk::SemaphoreTypeCreateInfo::default()
        .semaphore_type(vk::SemaphoreType::TIMELINE)
        .initial_value(0);
    let mut export = vk::ExportSemaphoreCreateInfo::default()
        .handle_types(crate::vulkan_contract::SEMAPHORE_HANDLE);
    let mut info = vk::SemaphoreCreateInfo::default().push_next(&mut kind);
    if external {
        info = info.push_next(&mut export);
    }
    // SAFETY: valid create information on the caller's Vulkan 1.2 device.
    unsafe { device.create_semaphore(&info, None) }.map_err(crate::common::api("vkCreateSemaphore"))
}

/// Read an adapter's identity and Vulkan version.
///
/// # Safety
/// The physical device belongs to the live instance.
pub(crate) unsafe fn identity(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
) -> (crate::common::Identity, u32) {
    let mut id = vk::PhysicalDeviceIDProperties::default();
    let version = {
        let mut properties = vk::PhysicalDeviceProperties2::default().push_next(&mut id);
        // SAFETY: the caller supplies a physical device of the live instance.
        unsafe { instance.get_physical_device_properties2(physical, &mut properties) };
        properties.properties.api_version
    };
    (
        crate::common::Identity {
            backend: kivact_types::GpuBackend::Vulkan,
            luid: (id.device_luid_valid == vk::TRUE).then_some(id.device_luid),
            device_uuid: Some(id.device_uuid),
            driver_uuid: Some(id.driver_uuid),
        },
        version,
    )
}
