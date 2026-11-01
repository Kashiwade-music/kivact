//! Role: Vulkan devices that import shared images and timeline semaphores.
//!
//! Devices: wgpu-hal enables the external memory extensions itself but not
//! the external semaphore ones, so an interop device is opened through
//! `Adapter::as_hal` → `open_with_callback` (adding them) →
//! `create_device_from_hal`. It needs Vulkan 1.2 (timeline semaphores).
//!
//! Imports: an image made with the engine's create info
//! (`ui_types::SharedTextures`, its usage with `STORAGE` when the textures
//! say `storage`), first checked importable with that usage
//! (`vkGetPhysicalDeviceImageFormatProperties2`), bound to the imported memory (a dedicated
//! allocation of the image's required size, in the lowest device-local
//! memory type it allows: what the engine allocated), handed to wgpu-hal
//! (`texture_from_raw`, which then owns image and memory); the ready and
//! release timeline semaphores imported permanently into semaphores of
//! wgpu's device; a command pool of wgpu's queue family with one acquire
//! and one release buffer per texture, recorded once (`SIMULTANEOUS_USE`):
//! the ownership barriers between `VK_QUEUE_FAMILY_EXTERNAL` and wgpu's
//! family, both keeping `SHADER_READ_ONLY_OPTIMAL` (the layout wgpu's
//! tracker holds for a sampled texture). Handles are OPAQUE_WIN32 on
//! Windows (an import does not take the NT handle, which is closed after
//! it) and OPAQUE_FD on Linux (a successful import takes the descriptor).
//!
//! Submissions: raw `vkQueueSubmit`s on wgpu's `VkQueue` (see the import module
//! invariants on external synchronization). The acquire batch waits for
//! the ready value, so every later submission is ordered after it; a
//! signal's first scope is every command earlier in submission order. An
//! own timeline semaphore tells when the queue finished (bounded waits).

#![allow(unsafe_code, reason = "Vulkan calls below wgpu's safe API")]

use crate::common::{Error, Identity, same_backend};
use ::wgpu::hal::api::Vulkan;
use ash::vk;
use std::time::Duration;
use ui_types::{GpuBackend, SharedHandle, SharedTextures};

use crate::vulkan_contract::{FORMAT, MEMORY_HANDLE, SEMAPHORE_HANDLE, usage};

/// The identity of a Vulkan adapter (`None`: not Vulkan).
pub(crate) fn identity(adapter: &::wgpu::Adapter) -> Option<Identity> {
    // SAFETY: only reads the adapter's handles while the guard lives.
    let hal = unsafe { adapter.as_hal::<Vulkan>() }?;
    let instance = hal.shared_instance().raw_instance();
    // SAFETY: the physical device belongs to the live instance retained by the guard.
    Some(unsafe { crate::vulkan_raw::identity(instance, hal.raw_physical_device()) }.0)
}

/// An interop device and its queue on `adapter`; `None` when it is not a
/// Vulkan adapter or cannot import (the caller opens an ordinary device).
pub(crate) fn request_device(
    adapter: &::wgpu::Adapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Option<Result<(::wgpu::Device, ::wgpu::Queue), Error>> {
    crate::wgpu_device::open(adapter, desc).transpose()
}

/// Whether `device` is a Vulkan device.
pub(crate) fn is_vulkan(device: &::wgpu::Device) -> bool {
    // SAFETY: only asks for the backend; the guard is dropped at once.
    unsafe { device.as_hal::<Vulkan>() }.is_some()
}

/// The shared image's create info, the external memory chain added by
/// the caller.
fn image_info(width: u32, height: u32, storage: bool) -> vk::ImageCreateInfo<'static> {
    vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(FORMAT)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage(storage))
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
}

/// What an import made before it is complete: destroyed when it fails
/// halfway.
struct Partial {
    device: ash::Device,
    images: Vec<(vk::Image, vk::DeviceMemory)>,
    semaphores: Vec<vk::Semaphore>,
    pool: vk::CommandPool,
}

impl Drop for Partial {
    fn drop(&mut self) {
        // Nothing of these was submitted yet.
        if self.pool != vk::CommandPool::null() {
            // SAFETY: a pool of this device, its buffers never submitted.
            unsafe { self.device.destroy_command_pool(self.pool, None) };
        }
        for &semaphore in &self.semaphores {
            // SAFETY: a semaphore of this device, never used.
            unsafe { self.device.destroy_semaphore(semaphore, None) };
        }
        for &(image, memory) in &self.images {
            // SAFETY: an image of this device, never used.
            unsafe { self.device.destroy_image(image, None) };
            if memory != vk::DeviceMemory::null() {
                // SAFETY: its memory, no longer bound to a live image.
                unsafe { self.device.free_memory(memory, None) };
            }
        }
    }
}

/// Import `shared` into the Vulkan `device`.
pub(crate) fn import(
    device: &::wgpu::Device,
    shared: SharedTextures,
    desc: &::wgpu::TextureDescriptor<'_>,
) -> Result<(Vec<::wgpu::Texture>, Shared), Error> {
    same_backend(&shared, GpuBackend::Vulkan)?;
    let (width, height, storage) = (shared.width, shared.height, shared.storage);
    // SAFETY: the guard is dropped before wgpu makes textures from it.
    let hal = unsafe { device.as_hal::<Vulkan>() }.ok_or("not a Vulkan device")?;
    let raw = hal.raw_device().clone();
    let instance = hal.shared_instance().raw_instance().clone();
    let physical = hal.raw_physical_device();
    let family = hal.queue_family_index();
    check_format(&instance, physical, (width, height), storage)?;
    let mut partial = Partial {
        device: raw.clone(),
        images: Vec::new(),
        semaphores: Vec::new(),
        pool: vk::CommandPool::null(),
    };
    for handle in shared.textures {
        let mut external = vk::ExternalMemoryImageCreateInfo::default().handle_types(MEMORY_HANDLE);
        let info = image_info(width, height, storage).push_next(&mut external);
        // SAFETY: a valid create info of a device that has the extension.
        let image = unsafe { raw.create_image(&info, None) }
            .map_err(crate::common::api("vkCreateImage"))?;
        partial.images.push((image, vk::DeviceMemory::null()));
        // SAFETY: a live image of `raw`.
        let required = unsafe { raw.get_image_memory_requirements(image) };
        let memory_type = device_local(&instance, physical, required.memory_type_bits)
            .ok_or("the shared image has no device-local memory type")?;
        let memory = import_memory(&raw, image, handle, required.size, memory_type)?;
        if let Some(last) = partial.images.last_mut() {
            last.1 = memory;
        }
        // SAFETY: a dedicated allocation of the image's required size.
        unsafe { raw.bind_image_memory(image, memory, 0) }
            .map_err(crate::common::api("vkBindImageMemory"))?;
    }
    for handle in [shared.ready, shared.release] {
        let semaphore = import_semaphore(&instance, &raw, handle)?;
        partial.semaphores.push(semaphore);
    }
    let own = timeline(&raw)?;
    partial.semaphores.push(own);
    let pool_info = vk::CommandPoolCreateInfo::default().queue_family_index(family);
    // SAFETY: a valid create info for wgpu's queue family.
    partial.pool = unsafe { raw.create_command_pool(&pool_info, None) }
        .map_err(crate::common::api("vkCreateCommandPool"))?;
    let count = u32::try_from(2 * partial.images.len()).map_err(|_| "too many textures")?;
    let allocate = vk::CommandBufferAllocateInfo::default()
        .command_pool(partial.pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(count);
    // SAFETY: a live pool of `raw`.
    let buffers = unsafe { raw.allocate_command_buffers(&allocate) }
        .map_err(crate::common::api("vkAllocateCommandBuffers"))?;
    let mut slots = Vec::with_capacity(partial.images.len());
    for (&(image, _), pair) in partial.images.iter().zip(buffers.as_chunks::<2>().0) {
        let all = vk::PipelineStageFlags::ALL_COMMANDS;
        let ext = vk::QUEUE_FAMILY_EXTERNAL;
        record(&raw, pair[0], image, (ext, family), (all, all), true)?;
        let bottom = vk::PipelineStageFlags::BOTTOM_OF_PIPE;
        record(&raw, pair[1], image, (family, ext), (all, bottom), false)?;
        slots.push(Slot {
            acquire: pair[0],
            release: pair[1],
        });
    }
    let hal_desc = ::wgpu::hal::TextureDescriptor {
        label: desc.label,
        size: desc.size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: desc.dimension,
        format: desc.format,
        usage: ::wgpu::TextureUses::RESOURCE,
        memory_flags: ::wgpu::hal::MemoryFlags::empty(),
        view_formats: Vec::new(),
    };
    let hal_textures: Vec<_> = std::mem::take(&mut partial.images)
        .into_iter()
        .map(|(image, memory)| {
            // SAFETY: the image was made as `hal_desc` says and bound to
            // `memory`, a dedicated allocation; wgpu-hal owns both from here.
            unsafe {
                hal.texture_from_raw(
                    image,
                    &hal_desc,
                    None,
                    ::wgpu::hal::vulkan::TextureMemory::Dedicated(memory),
                )
            }
        })
        .collect();
    let queue = hal.raw_queue();
    drop(hal);
    let textures = hal_textures
        .into_iter()
        .map(|texture| {
            // SAFETY: a texture of this device, made as `desc` says.
            unsafe { device.create_texture_from_hal::<Vulkan>(texture, desc) }
        })
        .collect();
    let [ready, release, own] = std::mem::take(&mut partial.semaphores)[..] else {
        unreachable!("invariant: three semaphores were made");
    };
    let pool = std::mem::replace(&mut partial.pool, vk::CommandPool::null());
    Ok((
        textures,
        Shared {
            device: raw,
            queue,
            ready,
            release,
            own,
            own_value: 0,
            pool,
            slots,
            leaked: false,
            _keep: device.clone(),
        },
    ))
}

/// Fail unless the shared image (its usage with or without storage) can be
/// imported at its size.
fn check_format(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    (width, height): (u32, u32),
    storage: bool,
) -> Result<(), Error> {
    let mut external_info =
        vk::PhysicalDeviceExternalImageFormatInfo::default().handle_type(MEMORY_HANDLE);
    let info = vk::PhysicalDeviceImageFormatInfo2::default()
        .format(FORMAT)
        .ty(vk::ImageType::TYPE_2D)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage(storage))
        .push_next(&mut external_info);
    let mut external = vk::ExternalImageFormatProperties::default();
    let extent = {
        let mut properties = vk::ImageFormatProperties2::default().push_next(&mut external);
        // SAFETY: a physical device of `instance`; valid chains.
        let supported = unsafe {
            instance.get_physical_device_image_format_properties2(physical, &info, &mut properties)
        };
        match supported {
            Ok(()) => {}
            Err(vk::Result::ERROR_FORMAT_NOT_SUPPORTED) => {
                return Err(Error::Mismatch(format!(
                    "the adapter has no shared images of this usage{}",
                    if storage { " (with storage)" } else { "" }
                )));
            }
            Err(e) => {
                return Err(crate::common::api(
                    "vkGetPhysicalDeviceImageFormatProperties2",
                )(e));
            }
        }
        properties.image_format_properties.max_extent
    };
    let features = external.external_memory_properties.external_memory_features;
    if !features.contains(vk::ExternalMemoryFeatureFlags::IMPORTABLE) {
        return Err(Error::Mismatch(format!(
            "the adapter cannot import shared images of this usage{}",
            if storage { " (with storage)" } else { "" }
        )));
    }
    if width > extent.width || height > extent.height {
        return Err(Error::Mismatch(format!(
            "the adapter imports images up to {} x {}",
            extent.width, extent.height
        )));
    }
    Ok(())
}

/// The lowest device-local memory type `bits` allows (the engine's
/// contract).
fn device_local(instance: &ash::Instance, physical: vk::PhysicalDevice, bits: u32) -> Option<u32> {
    // SAFETY: a physical device of the live instance, retained by wgpu.
    unsafe {
        crate::vulkan_raw::memory_type(
            instance,
            physical,
            bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
    }
}

/// The shared memory `handle` names, imported as `image`'s dedicated
/// allocation.
fn import_memory(
    device: &ash::Device,
    image: vk::Image,
    handle: SharedHandle,
    size: u64,
    memory_type: u32,
) -> Result<vk::DeviceMemory, Error> {
    let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(image);
    #[cfg(windows)]
    let mut import = {
        use std::os::windows::io::{AsHandle, AsRawHandle};
        vk::ImportMemoryWin32HandleInfoKHR::default()
            .handle_type(MEMORY_HANDLE)
            .handle(handle.as_handle().as_raw_handle() as isize)
    };
    #[cfg(target_os = "linux")]
    let mut import = {
        use std::os::fd::{AsFd, AsRawFd};
        vk::ImportMemoryFdInfoKHR::default()
            .handle_type(MEMORY_HANDLE)
            .fd(handle.as_fd().as_raw_fd())
    };
    let info = vk::MemoryAllocateInfo::default()
        .allocation_size(size)
        .memory_type_index(memory_type)
        .push_next(&mut dedicated)
        .push_next(&mut import);
    // SAFETY: `handle` is a live handle the caller says names memory
    // exported for this adapter with the same image and allocation info.
    let memory = unsafe { device.allocate_memory(&info, None) }
        .map_err(crate::common::api("importing the shared memory"))?;
    given(handle);
    Ok(memory)
}

/// A timeline semaphore of `device` importing the one `handle` names
/// (permanently).
fn import_semaphore(
    instance: &ash::Instance,
    device: &ash::Device,
    handle: SharedHandle,
) -> Result<vk::Semaphore, Error> {
    let semaphore = timeline(device)?;
    #[cfg(windows)]
    let imported = {
        use std::os::windows::io::{AsHandle, AsRawHandle};
        let ext = ash::khr::external_semaphore_win32::Device::new(instance, device);
        let info = vk::ImportSemaphoreWin32HandleInfoKHR::default()
            .semaphore(semaphore)
            .handle_type(SEMAPHORE_HANDLE)
            .handle(handle.as_handle().as_raw_handle() as isize);
        // SAFETY: a live handle of a timeline semaphore exported for this
        // adapter; `semaphore` is a new timeline semaphore of `device`.
        unsafe { ext.import_semaphore_win32_handle(&info) }
    };
    #[cfg(target_os = "linux")]
    let imported = {
        use std::os::fd::{AsFd, AsRawFd};
        let ext = ash::khr::external_semaphore_fd::Device::new(instance, device);
        let info = vk::ImportSemaphoreFdInfoKHR::default()
            .semaphore(semaphore)
            .handle_type(SEMAPHORE_HANDLE)
            .fd(handle.as_fd().as_raw_fd());
        // SAFETY: as above, with a descriptor.
        unsafe { ext.import_semaphore_fd(&info) }
    };
    match imported {
        Ok(()) => {
            given(handle);
            Ok(semaphore)
        }
        Err(e) => {
            // SAFETY: the new semaphore, never used.
            unsafe { device.destroy_semaphore(semaphore, None) };
            Err(crate::common::api("importing the shared semaphore")(e))
        }
    }
}

/// After a successful import: the NT handle is closed (an import does not
/// take it); the descriptor belongs to the driver now.
fn given(handle: SharedHandle) {
    #[cfg(windows)]
    drop(handle);
    #[cfg(target_os = "linux")]
    {
        use std::os::fd::IntoRawFd;
        let _ = std::os::fd::OwnedFd::from(handle).into_raw_fd();
    }
}

/// A new timeline semaphore at 0.
fn timeline(device: &ash::Device) -> Result<vk::Semaphore, Error> {
    // SAFETY: the import device was opened with Vulkan 1.2 and sharing extensions.
    unsafe { crate::vulkan_raw::timeline(device, false) }
}

/// Record an ownership barrier of `image` between `families`, keeping
/// `SHADER_READ_ONLY_OPTIMAL`.
fn record(
    device: &ash::Device,
    buffer: vk::CommandBuffer,
    image: vk::Image,
    (from, to): (u32, u32),
    (src_stage, dst_stage): (vk::PipelineStageFlags, vk::PipelineStageFlags),
    acquire: bool,
) -> Result<(), Error> {
    let begin =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
    // SAFETY: a new buffer of a pool of `device`.
    unsafe { device.begin_command_buffer(buffer, &begin) }
        .map_err(crate::common::api("vkBeginCommandBuffer"))?;
    let read = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    // SAFETY: the command buffer was begun above for the live imported image.
    unsafe {
        crate::vulkan_raw::barrier(
            device,
            buffer,
            image,
            (read, read),
            (from, to),
            (src_stage, vk::AccessFlags::empty()),
            (
                dst_stage,
                if acquire {
                    vk::AccessFlags::SHADER_READ
                } else {
                    vk::AccessFlags::empty()
                },
            ),
        )
    };
    // SAFETY: the buffer begun above.
    unsafe { device.end_command_buffer(buffer) }.map_err(crate::common::api("vkEndCommandBuffer"))
}

struct Slot {
    acquire: vk::CommandBuffer,
    release: vk::CommandBuffer,
}

/// An import's semaphores and ownership barriers on wgpu's queue.
pub(crate) struct Shared {
    device: ash::Device,
    queue: vk::Queue,
    ready: vk::Semaphore,
    release: vk::Semaphore,
    /// Tells when the queue finished (`idle`).
    own: vk::Semaphore,
    own_value: u64,
    pool: vk::CommandPool,
    slots: Vec<Slot>,
    /// The GPU may still use the objects: they are not destroyed.
    leaked: bool,
    /// Keeps the native device alive while the objects live.
    _keep: ::wgpu::Device,
}

impl Shared {
    fn submit(
        &self,
        buffer: Option<vk::CommandBuffer>,
        wait: Option<(vk::Semaphore, u64)>,
        signal: Option<(vk::Semaphore, u64)>,
    ) -> Result<(), Error> {
        let waits: Vec<_> = wait.iter().map(|w| w.0).collect();
        let wait_values: Vec<_> = wait.iter().map(|w| w.1).collect();
        let stages = vec![vk::PipelineStageFlags::ALL_COMMANDS; waits.len()];
        let signals: Vec<_> = signal.iter().map(|s| s.0).collect();
        let signal_values: Vec<_> = signal.iter().map(|s| s.1).collect();
        let buffers: Vec<_> = buffer.into_iter().collect();
        let mut values = vk::TimelineSemaphoreSubmitInfo::default()
            .wait_semaphore_values(&wait_values)
            .signal_semaphore_values(&signal_values);
        let info = vk::SubmitInfo::default()
            .command_buffers(&buffers)
            .wait_semaphores(&waits)
            .wait_dst_stage_mask(&stages)
            .signal_semaphores(&signals)
            .push_next(&mut values);
        // SAFETY: wgpu's queue, used only between wgpu's own calls on the
        // owning thread (external synchronization); the buffers are
        // `SIMULTANEOUS_USE` and their images live.
        unsafe {
            self.device
                .queue_submit(self.queue, &[info], vk::Fence::null())
        }
        .map_err(crate::common::api("vkQueueSubmit"))
    }

    pub(crate) fn ready_value(&self) -> u64 {
        // SAFETY: a timeline semaphore of this device.
        unsafe { self.device.get_semaphore_counter_value(self.ready) }.unwrap_or(0)
    }

    pub(crate) fn wait_ready(&self, value: u64, timeout: Duration) -> bool {
        self.wait(self.ready, value, timeout)
    }

    fn wait(&self, semaphore: vk::Semaphore, value: u64, timeout: Duration) -> bool {
        let semaphores = [semaphore];
        let values = [value];
        let info = vk::SemaphoreWaitInfo::default()
            .semaphores(&semaphores)
            .values(&values);
        let nanos = u64::try_from(timeout.as_nanos()).unwrap_or(u64::MAX);
        // SAFETY: a timeline semaphore of this device; the wait is bounded.
        unsafe { self.device.wait_semaphores(&info, nanos) }.is_ok()
    }

    pub(crate) fn acquire(&mut self, slot: usize, value: u64) -> Result<(), Error> {
        let buffer = self.slots.get(slot).ok_or("no such texture")?.acquire;
        self.submit(Some(buffer), Some((self.ready, value)), None)
    }

    pub(crate) fn release(&mut self, slot: usize) -> Result<(), Error> {
        let buffer = self.slots.get(slot).ok_or("no such texture")?.release;
        self.submit(Some(buffer), None, None)
    }

    pub(crate) fn signal(&mut self, value: u64) -> Result<(), Error> {
        self.submit(None, None, Some((self.release, value)))
    }

    pub(crate) fn idle(&mut self, timeout: Duration) -> bool {
        self.own_value += 1;
        self.submit(None, None, Some((self.own, self.own_value)))
            .is_ok()
            && self.wait(self.own, self.own_value, timeout)
    }

    pub(crate) fn leak(&mut self) {
        self.leaked = true;
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        if self.leaked {
            return;
        }
        // SAFETY: the queue finished every submission that used the pool's
        // buffers (`Import`'s drop waited).
        unsafe { self.device.destroy_command_pool(self.pool, None) };
        for semaphore in [self.ready, self.release, self.own] {
            // SAFETY: as above, for the semaphores.
            unsafe { self.device.destroy_semaphore(semaphore, None) };
        }
    }
}
