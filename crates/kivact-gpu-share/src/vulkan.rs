//! Role: A shared pool on the engine's own ash device: images, semaphores, barriers.
//!
//! The engine brings its Vulkan instance, physical device, device and
//! queue family ([`find_physical_device`] finds the physical device the
//! window reported; the device enables [`DEVICE_EXTENSIONS`], API
//! [`API_VERSION`] and the `timelineSemaphore` feature). [`Pool::new`]
//! makes the images and the two timeline semaphores as
//! [`kivact_types::SharedTextures`] says and exports them once. A frame:
//! [`Pool::acquire`] gives a [`Target`]; [`Target::record_acquire`], the
//! engine's own drawing, [`Target::record_release`], then
//! [`Target::submit`] (or the engine's own submission with
//! [`Target::finish`]'s semaphores).
//!
//! Invariants:
//! - A texture is written again only after the GPU waited for `release`
//!   to reach its last frame's value (the submission's timeline wait);
//!   the CPU waits, bounded, only for the texture's last submission to
//!   have signaled `ready`.
//! - Every `unsafe` block has its `SAFETY` note; the raw handles a caller
//!   passes are covered by the `# Safety` sections of the `unsafe fn`s.
//! - Dropping the pool waits (bounded) for the last frame's `ready` and
//!   leaks the images and semaphores instead when it does not arrive.

#![allow(
    unsafe_code,
    reason = "the Vulkan calls of an engine's shared pool, below any safe API"
)]

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

#[cfg(feature = "export")]
mod physical_device;
#[cfg(feature = "export")]
mod pool;
#[cfg(any(feature = "adapter", feature = "export"))]
pub(crate) mod raw;
#[cfg(feature = "export")]
mod read_back;

#[cfg(feature = "export")]
pub use physical_device::find_physical_device;
#[cfg(feature = "wgpu")]
pub(crate) use physical_device::{identity, uuids};
#[cfg(feature = "export")]
pub use pool::{Pool, Submission, Target};
#[cfg(feature = "export")]
pub use read_back::{ReadImage, read_back};
