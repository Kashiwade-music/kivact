//! Role: The test engine on its own Vulkan device, drawing into a shared pool.
//!
//! Its own instance and device (Vulkan 1.2, the pool's device extensions)
//! on the physical device whose UUIDs the window reported; the pool
//! (`kivact_gpu_share::vulkan::Pool`) makes, exports and hands
//! over the images and semaphores. A frame: the CPU's pixels into the
//! slot's staging buffer, the pool's acquire barrier to
//! `TRANSFER_DST_OPTIMAL`, a copy into the image, the pool's release
//! barrier and submission.

#![allow(
    unsafe_code,
    reason = "a Vulkan engine of its own, as an application has"
)]

use crate::engine::{Engine, POOL, WAIT};
use ash::vk;
use kivact_gpu_share::vulkan::{self as interop, Pool};
use kivact_gpu_share::{FrameReady, PoolConfig};
use kivact_types::{GpuAdapter, GpuBackend, SHARED_PREPARED, SharedHandle, SharedTextures};
use std::time::Duration;

fn failed(what: &'static str) -> impl Fn(vk::Result) -> String {
    move |e| format!("{what} failed: {e}")
}

/// The preparation signal on the pool's `release` timeline semaphore,
/// sent as an importing window sends it: a timeline semaphore of
/// `device` imports the handle (the import keeps the payload), is
/// signaled to [`SHARED_PREPARED`] and is destroyed.
pub(crate) fn prepare(
    instance: &ash::Instance,
    device: &ash::Device,
    release: &SharedHandle,
) -> Result<(), String> {
    let mut kind = vk::SemaphoreTypeCreateInfo::default()
        .semaphore_type(vk::SemaphoreType::TIMELINE)
        .initial_value(0);
    let info = vk::SemaphoreCreateInfo::default().push_next(&mut kind);
    // SAFETY: valid create information on the engine's device, which has
    // timeline semaphores (API 1.2).
    let semaphore =
        unsafe { device.create_semaphore(&info, None) }.map_err(failed("vkCreateSemaphore"))?;
    let outcome = import(instance, device, semaphore, release).and_then(|()| {
        let signal = vk::SemaphoreSignalInfo::default()
            .semaphore(semaphore)
            .value(SHARED_PREPARED);
        // SAFETY: the imported timeline semaphore of the device, signaled
        // on the host to a value above its current one (0).
        unsafe { device.signal_semaphore(&signal) }.map_err(failed("vkSignalSemaphore"))
    });
    // SAFETY: the semaphore made above, used by no submission.
    unsafe { device.destroy_semaphore(semaphore, None) };
    outcome
}

/// Import the `release` handle into `semaphore` (a new timeline
/// semaphore of `device`), permanently.
fn import(
    instance: &ash::Instance,
    device: &ash::Device,
    semaphore: vk::Semaphore,
    release: &SharedHandle,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::io::{AsHandle, AsRawHandle};
        let ext = ash::khr::external_semaphore_win32::Device::new(instance, device);
        let info = vk::ImportSemaphoreWin32HandleInfoKHR::default()
            .semaphore(semaphore)
            .handle_type(vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32)
            .handle(release.as_handle().as_raw_handle() as isize);
        // SAFETY: a live handle of a timeline semaphore exported for this
        // adapter; `semaphore` is a new timeline semaphore of `device`.
        unsafe { ext.import_semaphore_win32_handle(&info) }
            .map_err(failed("vkImportSemaphoreWin32HandleKHR"))
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::fd::{AsFd, FromRawFd, IntoRawFd};
        let ext = ash::khr::external_semaphore_fd::Device::new(instance, device);
        // The driver takes a descriptor it imports, so the import gets a
        // duplicate and the pool keeps its own.
        let fd = release
            .as_fd()
            .try_clone_to_owned()
            .map_err(|e| format!("duplicating the shared semaphore failed: {e}"))?;
        let raw = fd.into_raw_fd();
        let info = vk::ImportSemaphoreFdInfoKHR::default()
            .semaphore(semaphore)
            .handle_type(vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD)
            .fd(raw);
        // SAFETY: as above, with a descriptor of this process.
        match unsafe { ext.import_semaphore_fd(&info) } {
            Ok(()) => Ok(()),
            Err(e) => {
                // SAFETY: the import failed, so the driver did not take the
                // duplicate: this process still owns it.
                drop(unsafe { std::os::fd::OwnedFd::from_raw_fd(raw) });
                Err(format!("vkImportSemaphoreFdKHR failed: {e}"))
            }
        }
    }
}

/// What draws into one texture of the pool.
#[derive(Default)]
struct Slot {
    staging: vk::Buffer,
    staging_memory: vk::DeviceMemory,
    mapped: Option<std::ptr::NonNull<u8>>,
    command: vk::CommandBuffer,
    /// Signaled when the slot's last submission finished.
    fence: vk::Fence,
}

// xtask-allow(state-fields): Owned Vulkan resources; optional platform resources reflect independent enabled device extensions.
pub(crate) struct Vulkan {
    _entry: ash::Entry,
    instance: ash::Instance,
    physical: vk::PhysicalDevice,
    device: Option<ash::Device>,
    family: u32,
    queue: vk::Queue,
    width: u32,
    height: u32,
    slots: Vec<Slot>,
    commands: vk::CommandPool,
    pool: Option<Pool>,
}

/// A Vulkan 1.2 instance of the system's loader.
fn instance() -> Result<(ash::Entry, ash::Instance), String> {
    // SAFETY: loading the system's Vulkan loader.
    let entry = unsafe { ash::Entry::load() }.map_err(|e| format!("no Vulkan loader: {e}"))?;
    let app = vk::ApplicationInfo::default()
        .application_name(c"gpu-interop-producer")
        .api_version(interop::API_VERSION);
    let info = vk::InstanceCreateInfo::default().application_info(&app);
    // SAFETY: a valid create info.
    let instance =
        unsafe { entry.create_instance(&info, None) }.map_err(failed("vkCreateInstance"))?;
    Ok((entry, instance))
}

/// This machine's Vulkan 1.2 hardware adapters, as a window reports them.
pub(crate) fn adapters() -> Vec<GpuAdapter> {
    let Ok((_entry, instance)) = instance() else {
        return Vec::new();
    };
    // SAFETY: a live instance.
    let physicals = unsafe { instance.enumerate_physical_devices() }.unwrap_or_default();
    let mut found = Vec::new();
    for physical in physicals {
        let mut id = vk::PhysicalDeviceIDProperties::default();
        let mut properties = vk::PhysicalDeviceProperties2::default().push_next(&mut id);
        // SAFETY: a physical device of the instance; a valid chain.
        unsafe { instance.get_physical_device_properties2(physical, &mut properties) };
        let base = properties.properties;
        if base.api_version < interop::API_VERSION
            || base.device_type == vk::PhysicalDeviceType::CPU
        {
            continue;
        }
        found.push(GpuAdapter {
            backend: GpuBackend::Vulkan,
            name: base
                .device_name_as_c_str()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            luid: (id.device_luid_valid == vk::TRUE).then_some(id.device_luid),
            device_uuid: Some(id.device_uuid),
            driver_uuid: Some(id.driver_uuid),
            generation: 1,
        });
    }
    // SAFETY: the instance made no device.
    unsafe { instance.destroy_instance(None) };
    found
}

impl Vulkan {
    /// An engine of a pool of `width` × `height` textures, made for
    /// storage writes when `storage`.
    pub(crate) fn new(
        adapter: &GpuAdapter,
        (width, height): (u32, u32),
        storage: bool,
    ) -> Result<(Self, SharedTextures), String> {
        let (entry, instance) = instance()?;
        let mut engine = Self {
            _entry: entry,
            instance,
            physical: vk::PhysicalDevice::null(),
            device: None,
            family: 0,
            queue: vk::Queue::null(),
            width,
            height,
            slots: Vec::new(),
            commands: vk::CommandPool::null(),
            pool: None,
        };
        // SAFETY: a live instance of API 1.2.
        engine.physical = unsafe { interop::find_physical_device(&engine.instance, adapter) }
            .map_err(|e| e.to_string())?;
        engine.open()?;
        for _ in 0..POOL {
            engine.slots.push(Slot::default());
            engine.make_slot()?;
        }
        let config = PoolConfig {
            slots: POOL as u32,
            storage,
            ..PoolConfig::new(width, height)
        };
        // SAFETY: the engine's live instance, physical device and device
        // (made with the pool's extensions, API 1.2 and timeline
        // semaphores), which outlives the pool (dropped first).
        let (pool, shared) = unsafe {
            Pool::new(
                &engine.instance,
                engine.physical,
                engine.device(),
                engine.family,
                adapter,
                config,
            )
        }
        .map_err(|e| e.to_string())?;
        engine.pool = Some(pool);
        Ok((engine, shared))
    }

    fn device(&self) -> &ash::Device {
        self.device
            .as_ref()
            .expect("invariant: the device is made in `new`")
    }

    fn pool(&self) -> &Pool {
        self.pool
            .as_ref()
            .expect("invariant: the pool is made in `new`")
    }

    /// The device, its graphics queue and command pool.
    fn open(&mut self) -> Result<(), String> {
        // SAFETY: a physical device of the instance.
        let families = unsafe {
            self.instance
                .get_physical_device_queue_family_properties(self.physical)
        };
        self.family = families
            .iter()
            .position(|f| f.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .ok_or("no graphics queue")? as u32;
        let names: Vec<_> = interop::DEVICE_EXTENSIONS
            .iter()
            .map(|e| e.as_ptr())
            .collect();
        let mut v12 = vk::PhysicalDeviceVulkan12Features::default().timeline_semaphore(true);
        let queues = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(self.family)
            .queue_priorities(&[1.0])];
        let info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queues)
            .enabled_extension_names(&names)
            .push_next(&mut v12);
        // SAFETY: a valid create info for the physical device.
        let device = unsafe { self.instance.create_device(self.physical, &info, None) }
            .map_err(failed("vkCreateDevice"))?;
        // SAFETY: the one queue of the family made above.
        self.queue = unsafe { device.get_device_queue(self.family, 0) };
        self.device = Some(device);
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(self.family)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        // SAFETY: a valid create info.
        self.commands = unsafe { self.device().create_command_pool(&pool_info, None) }
            .map_err(failed("vkCreateCommandPool"))?;
        Ok(())
    }

    /// The last slot's staging buffer, command buffer and fence.
    fn make_slot(&mut self) -> Result<(), String> {
        let device = self.device().clone();
        let slot = self.slots.len() - 1;
        let size = u64::from(self.width) * u64::from(self.height) * 4;
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC);
        // SAFETY: a valid create info.
        let staging = unsafe { device.create_buffer(&buffer_info, None) }
            .map_err(failed("vkCreateBuffer"))?;
        self.slots[slot].staging = staging;
        // SAFETY: a live buffer.
        let required = unsafe { device.get_buffer_memory_requirements(staging) };
        let host = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        // SAFETY: a physical device of the instance.
        let properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical)
        };
        let index = (0..properties.memory_type_count)
            .find(|&i| {
                required.memory_type_bits & (1 << i) != 0
                    && properties.memory_types[i as usize]
                        .property_flags
                        .contains(host)
            })
            .ok_or("no host-visible memory")?;
        let allocate = vk::MemoryAllocateInfo::default()
            .allocation_size(required.size)
            .memory_type_index(index);
        // SAFETY: a valid allocate info.
        let staging_memory = unsafe { device.allocate_memory(&allocate, None) }
            .map_err(failed("vkAllocateMemory"))?;
        self.slots[slot].staging_memory = staging_memory;
        // SAFETY: a fresh allocation for the buffer.
        unsafe { device.bind_buffer_memory(staging, staging_memory, 0) }
            .map_err(failed("vkBindBufferMemory"))?;
        // SAFETY: host-visible memory, mapped once for the slot's life.
        let mapped = unsafe {
            device.map_memory(
                staging_memory,
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::empty(),
            )
        }
        .map_err(failed("vkMapMemory"))?;
        self.slots[slot].mapped = std::ptr::NonNull::new(mapped.cast());
        let allocate = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.commands)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        // SAFETY: a live pool.
        let commands = unsafe { device.allocate_command_buffers(&allocate) }
            .map_err(failed("vkAllocateCommandBuffers"))?;
        self.slots[slot].command = commands[0];
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        // SAFETY: a valid create info.
        self.slots[slot].fence =
            unsafe { device.create_fence(&fence_info, None) }.map_err(failed("vkCreateFence"))?;
        Ok(())
    }
}

impl Engine for Vulkan {
    fn pitch(&self) -> usize {
        self.width as usize * 4
    }

    fn draw(&mut self, rgba: &[u8]) -> Result<FrameReady, String> {
        let device = self.device().clone();
        let mut pool = self
            .pool
            .take()
            .expect("invariant: the pool is made in `new`");
        let drawn = self.draw_with(&device, &mut pool, rgba);
        self.pool = Some(pool);
        drawn
    }

    fn release_value(&self) -> u64 {
        self.pool().release_value().unwrap_or(0)
    }

    fn wait_release(&self, value: u64, timeout: Duration) -> bool {
        self.pool().wait_release(value, timeout).is_ok()
    }

    fn read_back(&self, slot: u32) -> Result<Vec<u8>, String> {
        // SAFETY: the engine's own queue of the pool's family, used by this
        // thread alone; the caller's window does not use the texture.
        unsafe { self.pool().read_back(self.queue, slot, WAIT) }.map_err(|e| e.to_string())
    }

    fn prepare(&self, release: &SharedHandle) -> Result<(), String> {
        prepare(&self.instance, self.device(), release)
    }
}

impl Vulkan {
    fn draw_with(
        &self,
        device: &ash::Device,
        pool: &mut Pool,
        rgba: &[u8],
    ) -> Result<FrameReady, String> {
        let target = pool.acquire(WAIT).map_err(|e| e.to_string())?;
        let slot = &self.slots[target.slot() as usize];
        let (command, fence, staging) = (slot.command, slot.fence, slot.staging);
        let nanos = u64::try_from(WAIT.as_nanos()).unwrap_or(u64::MAX);
        // SAFETY: a fence of the device; a bounded wait (already signaled:
        // the pool waited for the slot's `ready`).
        if let Err(e) = unsafe { device.wait_for_fences(&[fence], true, nanos) } {
            return Err(format!("the texture's last frame did not finish: {e}"));
        }
        // SAFETY: the fence's submission finished.
        unsafe { device.reset_fences(&[fence]) }.map_err(failed("vkResetFences"))?;
        let mapped = slot.mapped.ok_or("the staging buffer is not mapped")?;
        let len = rgba
            .len()
            .min(self.width as usize * self.height as usize * 4);
        // SAFETY: the mapped staging buffer holds `width * height * 4` bytes
        // and the GPU no longer reads it (the fence).
        unsafe { std::ptr::copy_nonoverlapping(rgba.as_ptr(), mapped.as_ptr(), len) };
        let begin = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        // SAFETY: the buffer's last submission finished (the fence).
        unsafe { device.begin_command_buffer(command, &begin) }
            .map_err(failed("vkBeginCommandBuffer"))?;
        let dst = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        let write = (
            vk::PipelineStageFlags::TRANSFER,
            vk::AccessFlags::TRANSFER_WRITE,
        );
        // SAFETY: the begun buffer of the pool's device and family.
        unsafe { target.record_acquire(command, dst, write) };
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D::default(),
            image_extent: target.extent().into(),
        };
        // SAFETY: recording a copy into the begun buffer.
        unsafe {
            device.cmd_copy_buffer_to_image(command, staging, target.image(), dst, &[region])
        };
        // SAFETY: as above.
        unsafe { target.record_release(command, dst, write) };
        // SAFETY: the begun buffer.
        unsafe { device.end_command_buffer(command) }.map_err(failed("vkEndCommandBuffer"))?;
        // SAFETY: the engine's own queue of the pool's family; the recorded
        // buffer; the slot's reset fence.
        unsafe { target.submit(self.queue, &[command], fence) }.map_err(|e| e.to_string())
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        if let Some(device) = self.device.clone() {
            let fences: Vec<_> = self
                .slots
                .iter()
                .map(|s| s.fence)
                .filter(|f| *f != vk::Fence::null())
                .collect();
            let nanos = u64::try_from(WAIT.as_nanos()).unwrap_or(u64::MAX);
            // SAFETY: fences of the device; a bounded wait.
            let finished = fences.is_empty()
                || unsafe { device.wait_for_fences(&fences, true, nanos) }.is_ok();
            if !finished {
                // The GPU may still wait for a release: keep everything.
                std::mem::forget(self.pool.take());
                return;
            }
            // The pool's images and semaphores, while the device lives.
            drop(self.pool.take());
            for slot in &self.slots {
                // SAFETY: nothing of the slot is in use (the fences).
                unsafe { device.destroy_fence(slot.fence, None) };
                // SAFETY: as above.
                unsafe { device.destroy_buffer(slot.staging, None) };
                // SAFETY: as above (freeing unmaps).
                unsafe { device.free_memory(slot.staging_memory, None) };
            }
            // SAFETY: the pool's buffers finished (the fences).
            unsafe { device.destroy_command_pool(self.commands, None) };
            // SAFETY: every object of the device is gone.
            unsafe { device.destroy_device(None) };
        }
        // SAFETY: the instance's device is gone (or was never made).
        unsafe { self.instance.destroy_instance(None) };
    }
}
