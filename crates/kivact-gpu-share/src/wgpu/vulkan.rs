//! Role: The Vulkan half of the wgpu helpers: device, wrapped images, hand-off.
//!
//! Devices: wgpu-hal enables the external memory extensions itself but not
//! the external semaphore ones, so the device is opened through
//! `Adapter::as_hal` → `open_with_callback` (adding
//! [`crate::vulkan::DEVICE_EXTENSIONS`]) → `create_device_from_hal`. wgpu
//! enables timeline semaphores on a Vulkan 1.2 device.
//!
//! Pools: the backend's pool ([`crate::vulkan::Pool`]) makes and exports
//! the images on wgpu's `VkDevice`, then hands them and their dedicated
//! memory to wgpu-hal (`texture_from_raw`, which destroys them when wgpu
//! drops the texture). Two command buffers per texture, recorded each
//! frame once the texture's last frame finished (the pool's `acquire`
//! waited for its `ready`), carry the pool's acquire and release barriers,
//! both keeping the hand-off layout (the layout wgpu's tracker holds for
//! `TextureUses::RESOURCE`).

use crate::common::{Error, api};
use crate::pool::{FrameReady, PoolConfig};
use crate::vulkan::{self as backend, DEVICE_EXTENSIONS, HAND_OFF_LAYOUT};
use ::wgpu::hal::api::Vulkan;
use ash::vk;
use kivact_types::{GpuAdapter, SharedTextures};
use std::mem::ManuallyDrop;
use std::time::Duration;

/// Whether the Vulkan adapter `found` is `adapter` (its UUIDs).
pub(super) fn is_adapter(found: &::wgpu::Adapter, adapter: &GpuAdapter) -> Result<bool, Error> {
    let wanted = backend::uuids(adapter)?;
    // SAFETY: only reads the adapter's handles while the guard lives.
    let Some(hal) = (unsafe { found.as_hal::<Vulkan>() }) else {
        return Ok(false);
    };
    let instance = hal.shared_instance().raw_instance();
    // SAFETY: the adapter's physical device, of its live instance.
    let (device, driver, _) = unsafe { backend::identity(instance, hal.raw_physical_device()) };
    Ok((device, driver) == wanted)
}

/// A device of `adapter` with the sharing's extensions and `desc`'s
/// features and limits.
pub(super) fn open(
    adapter: ::wgpu::Adapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Result<(::wgpu::Adapter, ::wgpu::Device, ::wgpu::Queue), Error> {
    let (device, queue) = crate::wgpu_device::open(&adapter, desc)?
        .ok_or_else(|| Error::Mismatch("the adapter cannot share Vulkan textures".into()))?;
    Ok((adapter, device, queue))
}

/// The command pool of the hand-off's barriers, destroyed when dropped
/// (unless leaked).
struct Commands {
    device: ash::Device,
    pool: vk::CommandPool,
    /// Per texture: the acquire and the release buffer.
    buffers: Vec<[vk::CommandBuffer; 2]>,
    leaked: bool,
}

impl Drop for Commands {
    fn drop(&mut self) {
        if !self.leaked {
            // SAFETY: the buffers finished (the pool's last `ready`, or never
            // submitted); destroying a null pool is a no-op.
            unsafe { self.device.destroy_command_pool(self.pool, None) };
        }
    }
}

/// A pool's Vulkan half: the backend's pool and the hand-off on wgpu's
/// queue.
pub(super) struct Raw {
    /// Dropped by [`Raw`]'s drop unless leaked.
    pool: ManuallyDrop<backend::Pool>,
    queue: vk::Queue,
    commands: Commands,
}

impl Drop for Raw {
    fn drop(&mut self) {
        if !self.commands.leaked {
            // SAFETY: dropped once, here; not used after.
            unsafe { ManuallyDrop::drop(&mut self.pool) };
        }
    }
}

impl Raw {
    /// A pool on the Vulkan `device`, its textures wrapped as `desc`
    /// says, and its handles.
    pub(super) fn new(
        device: &::wgpu::Device,
        adapter: &GpuAdapter,
        config: PoolConfig,
        desc: &::wgpu::TextureDescriptor<'_>,
    ) -> Result<(Self, Vec<::wgpu::Texture>, SharedTextures), Error> {
        // SAFETY: the guard is dropped before wgpu makes textures from it.
        let hal = unsafe { device.as_hal::<Vulkan>() }
            .ok_or_else(|| Error::Mismatch("not a Vulkan device".into()))?;
        let instance = hal.shared_instance().raw_instance().clone();
        let physical = hal.raw_physical_device();
        let raw = hal.raw_device().clone();
        let family = hal.queue_family_index();
        if hal.shared_instance().instance_api_version() < backend::API_VERSION
            || !DEVICE_EXTENSIONS
                .iter()
                .all(|e| hal.enabled_device_extensions().contains(e))
        {
            return Err(Error::Mismatch(
                "the device cannot share (open it with `wgpu::request_device`)".into(),
            ));
        }
        let mut commands = Commands {
            device: raw.clone(),
            pool: vk::CommandPool::null(),
            buffers: Vec::new(),
            leaked: false,
        };
        let info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(family)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        // SAFETY: a valid create info for wgpu's queue family.
        commands.pool =
            unsafe { raw.create_command_pool(&info, None) }.map_err(api("vkCreateCommandPool"))?;
        let allocate = vk::CommandBufferAllocateInfo::default()
            .command_pool(commands.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(2 * config.slots);
        // SAFETY: a live pool of the device.
        let buffers = unsafe { raw.allocate_command_buffers(&allocate) }
            .map_err(api("vkAllocateCommandBuffers"))?;
        commands.buffers = buffers.as_chunks::<2>().0.to_vec();
        // SAFETY: wgpu's live instance, physical device and device, made by
        // `request_device` (checked above: Vulkan 1.2, so timeline
        // semaphores, and the extensions); the pool keeps a clone of the
        // `wgpu::Device`, so the device outlives it.
        let (mut pool, shared) =
            unsafe { backend::Pool::new(&instance, physical, &raw, family, adapter, config) }?;
        let hal_desc = ::wgpu::hal::TextureDescriptor {
            label: desc.label,
            size: desc.size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: desc.dimension,
            format: desc.format,
            usage: hal_usage(config.storage),
            memory_flags: ::wgpu::hal::MemoryFlags::empty(),
            view_formats: Vec::new(),
        };
        let hal_textures: Vec<_> = pool
            .hand_over_images()
            .into_iter()
            .map(|(image, memory)| {
                // SAFETY: an image of this device made as `hal_desc` says (no
                // mutable format), bound to `memory`, its dedicated
                // allocation; wgpu-hal owns both from here.
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
        Ok((
            Self {
                pool: ManuallyDrop::new(pool),
                queue,
                commands,
            },
            textures,
            shared,
        ))
    }

    pub(super) fn pool(&self) -> &backend::Pool {
        &self.pool
    }

    pub(super) fn pool_mut(&mut self) -> &mut backend::Pool {
        &mut self.pool
    }

    /// `buffer` begun for one submission.
    fn begin(&self, buffer: vk::CommandBuffer) -> Result<(), Error> {
        let begin = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        // SAFETY: a buffer of the pool's command pool (resettable), whose
        // last submission finished (the texture's last `ready`).
        unsafe { self.commands.device.begin_command_buffer(buffer, &begin) }
            .map_err(api("vkBeginCommandBuffer"))
    }

    /// Step 1: wait for `release` and acquire `target`'s image from
    /// `VK_QUEUE_FAMILY_EXTERNAL` keeping the hand-off layout.
    ///
    /// # Safety
    /// No other thread uses wgpu's queue during the call.
    pub(super) unsafe fn take(&mut self, target: &backend::Target) -> Result<(), Error> {
        let device = &self.commands.device;
        let buffer = self.commands.buffers[target.slot() as usize][0];
        self.begin(buffer)?;
        let all = (
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE,
        );
        // SAFETY: the begun buffer of the pool's device and family.
        unsafe {
            self.pool
                .record_acquire(buffer, target, HAND_OFF_LAYOUT, all)
        };
        // SAFETY: the begun buffer.
        unsafe { device.end_command_buffer(buffer) }.map_err(api("vkEndCommandBuffer"))?;
        let wait = target.release_wait();
        let (waits, values) = if wait > 0 {
            (vec![self.pool.release_semaphore()], vec![wait])
        } else {
            (Vec::new(), Vec::new())
        };
        let stages = vec![vk::PipelineStageFlags::ALL_COMMANDS; waits.len()];
        let buffers = [buffer];
        let mut timeline =
            vk::TimelineSemaphoreSubmitInfo::default().wait_semaphore_values(&values);
        let info = vk::SubmitInfo::default()
            .command_buffers(&buffers)
            .wait_semaphores(&waits)
            .wait_dst_stage_mask(&stages)
            .push_next(&mut timeline);
        // SAFETY: wgpu's queue, not used by another thread meanwhile (the
        // caller); the recorded buffer of its family.
        unsafe { device.queue_submit(self.queue, &[info], vk::Fence::null()) }
            .map_err(api("vkQueueSubmit"))
    }

    /// Step 4: release `target`'s image to `VK_QUEUE_FAMILY_EXTERNAL`
    /// keeping the hand-off layout and signal `ready`.
    ///
    /// # Safety
    /// As [`Raw::take`].
    pub(super) unsafe fn give(&mut self, target: backend::Target) -> Result<FrameReady, Error> {
        let buffer = self.commands.buffers[target.slot() as usize][1];
        let recorded = self.begin(buffer).and_then(|()| {
            let written = (
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::AccessFlags::MEMORY_WRITE,
            );
            // SAFETY: the begun buffer of the pool's device and family.
            unsafe {
                self.pool
                    .record_release(buffer, &target, HAND_OFF_LAYOUT, written)
            };
            // SAFETY: the begun buffer.
            unsafe { self.commands.device.end_command_buffer(buffer) }
                .map_err(api("vkEndCommandBuffer"))
        });
        if let Err(error) = recorded {
            self.pool.cancel(target);
            return Err(error);
        }
        // SAFETY: wgpu's queue of the pool's device and family, not used by
        // another thread meanwhile (the caller); the recorded buffer.
        unsafe {
            self.pool
                .submit(self.queue, target, &[buffer], vk::Fence::null())
        }
    }

    /// Texture `slot`'s last frame, read back on wgpu's queue.
    ///
    /// # Safety
    /// As [`crate::wgpu::Pool::read_back`].
    pub(super) unsafe fn read_back(&self, slot: u32, timeout: Duration) -> Result<Vec<u8>, Error> {
        // SAFETY: wgpu's queue of the pool's family; the rest as the caller
        // says.
        unsafe { self.pool.read_back(self.queue, slot, timeout) }
    }

    pub(super) fn leak(&mut self) {
        // The backend's pool is never dropped: its semaphores stay.
        self.commands.leaked = true;
    }
}

/// The hal usage matching [`crate::wgpu::usage`].
fn hal_usage(storage: bool) -> ::wgpu::TextureUses {
    let usage = ::wgpu::TextureUses::COLOR_TARGET
        | ::wgpu::TextureUses::RESOURCE
        | ::wgpu::TextureUses::COPY_SRC
        | ::wgpu::TextureUses::COPY_DST;
    if storage {
        usage | ::wgpu::TextureUses::STORAGE_READ_WRITE | ::wgpu::TextureUses::STORAGE_WRITE_ONLY
    } else {
        usage
    }
}
