//! Role: Vulkan version, extensions, layouts and usage of shared textures.

use ash::vk;
use std::ffi::CStr;

/// The Vulkan API version the engine's instance and device need at least
/// (timeline semaphores, external memory and semaphores, dedicated
/// allocations are core in 1.2).
pub const API_VERSION: u32 = vk::API_VERSION_1_2;

/// The device extensions the engine's `VkDevice` enables for the pool:
/// exporting memory and semaphores as NT handles.
#[cfg(windows)]
pub const DEVICE_EXTENSIONS: [&CStr; 2] = [
    ash::khr::external_memory_win32::NAME,
    ash::khr::external_semaphore_win32::NAME,
];
/// The device extensions the engine's `VkDevice` enables for the pool:
/// exporting memory and semaphores as file descriptors.
#[cfg(target_os = "linux")]
pub const DEVICE_EXTENSIONS: [&CStr; 2] = [
    ash::khr::external_memory_fd::NAME,
    ash::khr::external_semaphore_fd::NAME,
];

/// The layout a texture has between the window and the engine.
pub const HAND_OFF_LAYOUT: vk::ImageLayout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;

#[cfg(windows)]
#[cfg(any(feature = "export", feature = "import"))]
pub(crate) const MEMORY_HANDLE: vk::ExternalMemoryHandleTypeFlags =
    vk::ExternalMemoryHandleTypeFlags::OPAQUE_WIN32;
#[cfg(target_os = "linux")]
#[cfg(any(feature = "export", feature = "import"))]
pub(crate) const MEMORY_HANDLE: vk::ExternalMemoryHandleTypeFlags =
    vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD;
#[cfg(windows)]
#[cfg(any(feature = "export", feature = "import"))]
pub(crate) const SEMAPHORE_HANDLE: vk::ExternalSemaphoreHandleTypeFlags =
    vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32;
#[cfg(target_os = "linux")]
#[cfg(any(feature = "export", feature = "import"))]
pub(crate) const SEMAPHORE_HANDLE: vk::ExternalSemaphoreHandleTypeFlags =
    vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD;

/// The contract's image usage without storage.
const USAGE: vk::ImageUsageFlags = vk::ImageUsageFlags::from_raw(
    vk::ImageUsageFlags::TRANSFER_SRC.as_raw()
        | vk::ImageUsageFlags::TRANSFER_DST.as_raw()
        | vk::ImageUsageFlags::SAMPLED.as_raw()
        | vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw(),
);

/// The image usage required by the sharing contract, with `STORAGE` for a pool made for storage
/// writes.
pub fn usage(storage: bool) -> vk::ImageUsageFlags {
    if storage {
        USAGE | vk::ImageUsageFlags::STORAGE
    } else {
        USAGE
    }
}

/// The pool images' format (premultiplied alpha).
///
/// The values are sRGB-encoded: the window composites them as it does
/// `Pixels`. An engine that computes in linear light encodes to sRGB
/// last; the texture is not sampled through an sRGB view.
pub const FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
