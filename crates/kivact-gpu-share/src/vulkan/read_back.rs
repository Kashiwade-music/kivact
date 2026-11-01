//! Role: Reads an engine's Vulkan image back to the CPU as straight RGBA8.
//!
//! For a window that cannot import (it draws on the CPU, or its GPU is not
//! up yet): the UI shows the frame as `Pixels`. That needs no shared pool:
//! the engine draws into an image of its own and reads that back.

#![allow(
    unsafe_code,
    reason = "the Vulkan calls of a read back, below any safe API"
)]

use super::raw::{COLOR, barrier, memory_type};
use crate::common::{Error, api};
use crate::pool::straighten;
use ash::vk;
use std::time::Duration;

/// An engine's image to read back ([`read_back`]): a 2D image of
/// `vulkan::FORMAT` (premultiplied alpha), 1 mip level, 1 layer, created with
/// `TRANSFER_SRC` usage.
/// Produced by the framework; read its fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReadImage {
    /// The image.
    pub image: vk::Image,
    /// Its layout; the read back leaves it in this layout.
    pub layout: vk::ImageLayout,
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// The queue family that owns it: the reading queue's family (or
    /// `VK_QUEUE_FAMILY_IGNORED`) reads it directly; another family
    /// (`VK_QUEUE_FAMILY_EXTERNAL` for a pool's texture) has released it
    /// keeping `layout`, and gets it back released the same way.
    pub owner: u32,
}

/// What a read back made, destroyed when it is dropped (unless leaked).
struct Scratch<'a> {
    device: &'a ash::Device,
    pool: vk::CommandPool,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    fence: vk::Fence,
}

impl Drop for Scratch<'_> {
    fn drop(&mut self) {
        // SAFETY: the command buffer finished or was never submitted (the
        // fence; a leaked scratch is forgotten); null handles are no-ops.
        unsafe { self.device.destroy_fence(self.fence, None) };
        // SAFETY: as above.
        unsafe { self.device.destroy_buffer(self.buffer, None) };
        // SAFETY: as above, after its buffer (freeing unmaps).
        unsafe { self.device.free_memory(self.memory, None) };
        // SAFETY: as above (destroying the pool frees its buffers).
        unsafe { self.device.destroy_command_pool(self.pool, None) };
    }
}

/// `image`'s pixels as straight-alpha RGBA8 (`width` × `height` × 4 bytes,
/// rows top to bottom), copied on `queue` (of the family given with it)
/// after its earlier work there and waited for at most `timeout`.
///
/// # Safety
/// - `instance`, `physical` and `device` are live, `device` made from
///   `physical`; `queue` is a queue of `device` of the given family, not
///   used by another thread during the call.
/// - `image` is as [`ReadImage`] says, an image of `device`, not used
///   meanwhile by other queues (their work on it finished or is ordered
///   before this call's submission).
pub unsafe fn read_back(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    device: &ash::Device,
    (queue, family): (vk::Queue, u32),
    image: &ReadImage,
    timeout: Duration,
) -> Result<Vec<u8>, Error> {
    if image.width == 0 || image.height == 0 {
        return Err(Error::Mismatch("an empty image".into()));
    }
    let size = u64::from(image.width) * u64::from(image.height) * 4;
    let mut scratch = Scratch {
        device,
        pool: vk::CommandPool::null(),
        buffer: vk::Buffer::null(),
        memory: vk::DeviceMemory::null(),
        fence: vk::Fence::null(),
    };
    let info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(vk::BufferUsageFlags::TRANSFER_DST);
    // SAFETY: a valid create info for the device.
    scratch.buffer = unsafe { device.create_buffer(&info, None) }.map_err(api("vkCreateBuffer"))?;
    // SAFETY: a live buffer.
    let required = unsafe { device.get_buffer_memory_requirements(scratch.buffer) };
    let host = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    // SAFETY: the live physical device of the instance (the caller).
    let index = unsafe { memory_type(instance, physical, required.memory_type_bits, host) }
        .ok_or_else(|| Error::Mismatch("no host-visible memory".into()))?;
    let allocate = vk::MemoryAllocateInfo::default()
        .allocation_size(required.size)
        .memory_type_index(index);
    // SAFETY: a valid allocate info.
    scratch.memory =
        unsafe { device.allocate_memory(&allocate, None) }.map_err(api("vkAllocateMemory"))?;
    // SAFETY: a fresh allocation for the buffer.
    unsafe { device.bind_buffer_memory(scratch.buffer, scratch.memory, 0) }
        .map_err(api("vkBindBufferMemory"))?;
    let pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(family)
        .flags(vk::CommandPoolCreateFlags::TRANSIENT);
    // SAFETY: a valid create info.
    scratch.pool = unsafe { device.create_command_pool(&pool_info, None) }
        .map_err(api("vkCreateCommandPool"))?;
    let allocate = vk::CommandBufferAllocateInfo::default()
        .command_pool(scratch.pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    // SAFETY: a live pool.
    let command = unsafe { device.allocate_command_buffers(&allocate) }
        .map_err(api("vkAllocateCommandBuffers"))?[0];
    // SAFETY: a valid create info.
    scratch.fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }
        .map_err(api("vkCreateFence"))?;
    let begin =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    // SAFETY: a new buffer of the pool.
    unsafe { device.begin_command_buffer(command, &begin) }.map_err(api("vkBeginCommandBuffer"))?;
    // SAFETY: the begun buffer; the image as the caller says.
    unsafe { record(device, command, scratch.buffer, image, family) };
    // SAFETY: the begun buffer.
    unsafe { device.end_command_buffer(command) }.map_err(api("vkEndCommandBuffer"))?;
    let commands = [command];
    let submit = vk::SubmitInfo::default().command_buffers(&commands);
    // SAFETY: the caller's queue; the recorded buffer.
    unsafe { device.queue_submit(queue, &[submit], scratch.fence) }
        .map_err(api("vkQueueSubmit"))?;
    let nanos = u64::try_from(timeout.as_nanos()).unwrap_or(u64::MAX);
    // SAFETY: a fence of the device; a bounded wait.
    match unsafe { device.wait_for_fences(&[scratch.fence], true, nanos) } {
        Ok(()) => {}
        Err(error) => {
            // The copy may still run: keep what it uses.
            std::mem::forget(scratch);
            return Err(match error {
                vk::Result::TIMEOUT => Error::Timeout("the read back"),
                error => api("vkWaitForFences")(error),
            });
        }
    }
    // SAFETY: host-visible memory the finished copy wrote.
    let mapped = unsafe { device.map_memory(scratch.memory, 0, size, vk::MemoryMapFlags::empty()) }
        .map_err(api("vkMapMemory"))?;
    let len = usize::try_from(size).map_err(|_| Error::Mismatch("an image too large".into()))?;
    let mut rgba = vec![0; len];
    // SAFETY: the mapping holds `size` bytes, written and made visible to
    // the host (the barrier and the fence).
    unsafe { std::ptr::copy_nonoverlapping(mapped.cast::<u8>(), rgba.as_mut_ptr(), len) };
    straighten(&mut rgba);
    Ok(rgba)
}

/// Record the copy of `image` into `buffer`, with its ownership and
/// layout moves around it.
///
/// # Safety
/// `command` is a recording buffer of `device` for a queue of `family`;
/// `buffer` holds the image's pixels; `image` as [`read_back`] says.
unsafe fn record(
    device: &ash::Device,
    command: vk::CommandBuffer,
    buffer: vk::Buffer,
    image: &ReadImage,
    family: u32,
) {
    let all = vk::PipelineStageFlags::ALL_COMMANDS;
    let transfer = vk::PipelineStageFlags::TRANSFER;
    let none = vk::AccessFlags::empty();
    let memory = vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE;
    let read = vk::AccessFlags::TRANSFER_READ;
    let source = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
    let foreign = image.owner != family && image.owner != vk::QUEUE_FAMILY_IGNORED;
    let ignored = (vk::QUEUE_FAMILY_IGNORED, vk::QUEUE_FAMILY_IGNORED);
    if foreign {
        // SAFETY: the recording buffer (the caller).
        unsafe {
            barrier(
                device,
                command,
                image.image,
                (image.layout, image.layout),
                (image.owner, family),
                (all, none),
                (all, memory),
            );
        }
    }
    // SAFETY: as above.
    unsafe {
        barrier(
            device,
            command,
            image.image,
            (image.layout, source),
            ignored,
            (all, vk::AccessFlags::MEMORY_WRITE),
            (transfer, read),
        );
    }
    let region = vk::BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length: 0,
        buffer_image_height: 0,
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: COLOR.aspect_mask,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        image_offset: vk::Offset3D::default(),
        image_extent: vk::Extent3D {
            width: image.width,
            height: image.height,
            depth: 1,
        },
    };
    // SAFETY: as above, with a buffer of the image's size.
    unsafe { device.cmd_copy_image_to_buffer(command, image.image, source, buffer, &[region]) };
    let host = vk::BufferMemoryBarrier::default()
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::HOST_READ)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .buffer(buffer)
        .size(vk::WHOLE_SIZE);
    // SAFETY: as above.
    unsafe {
        device.cmd_pipeline_barrier(
            command,
            transfer,
            vk::PipelineStageFlags::HOST,
            vk::DependencyFlags::empty(),
            &[],
            &[host],
            &[],
        );
    }
    // SAFETY: as above.
    unsafe {
        barrier(
            device,
            command,
            image.image,
            (source, image.layout),
            ignored,
            (transfer, none),
            (all, none),
        );
    }
    if foreign {
        // SAFETY: as above.
        unsafe {
            barrier(
                device,
                command,
                image.image,
                (image.layout, image.layout),
                (family, image.owner),
                (all, none),
                (vk::PipelineStageFlags::BOTTOM_OF_PIPE, none),
            );
        }
    }
}
