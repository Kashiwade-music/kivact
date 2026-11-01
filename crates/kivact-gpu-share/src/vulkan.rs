//! Role: A shared pool on the engine's own ash device: images, semaphores, barriers.
//!
//! The engine brings its Vulkan instance, physical device, device and
//! queue family ([`find_physical_device`] finds the physical device the
//! window reported; the device enables [`DEVICE_EXTENSIONS`], API
//! [`API_VERSION`] and the `timelineSemaphore` feature). [`Pool::new`]
//! makes the images and the two timeline semaphores as
//! [`kivact_types::SharedTextures`] says and exports them once. A frame:
//! [`Pool::acquire`], [`Pool::record_acquire`], the engine's own drawing,
//! [`Pool::record_release`], then [`Pool::submit`] (or the engine's own
//! submission with [`Pool::finish`]'s semaphores).
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

mod physical_device;
mod pool;
mod raw;
mod read_back;

pub use crate::vulkan_contract::{API_VERSION, DEVICE_EXTENSIONS, HAND_OFF_LAYOUT};
pub use physical_device::find_physical_device;
#[cfg(feature = "wgpu")]
pub(crate) use physical_device::{identity, uuids};
pub use pool::{Pool, Submission, Target};
pub use raw::FORMAT;
pub use read_back::{ReadImage, read_back};
