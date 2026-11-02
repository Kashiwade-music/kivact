//! Role: Imports an external engine's GPU textures and fences into wgpu.
//!
//! Responsibility: what GPU texture sharing needs below wgpu's safe API: reading an adapter's identity (DXGI's LUID, Vulkan's
//! device and driver UUIDs), opening a Vulkan device with the external
//! semaphore extensions wgpu does not enable, importing a pool of shared
//! textures and its two timeline fences ([`kivact_types::SharedTextures`],
//! whose rustdoc is the engine's contract) into a wgpu device, and the
//! raw work on wgpu's queue around wgpu's own submissions.
//! `arvane-render` uses it (feature `gpu-interop`); the facade never names
//! a wgpu, ash or windows type.
//!
//! Hand-off, per frame of the presenting thread:
//! - [`Import::ready_value`] and [`Import::wait_ready`] read the ready
//!   fence on the CPU (bounded), so the queue is never made to wait on the
//!   GPU for a frame that has not arrived.
//! - [`Import::acquire`], before the frame's submission: a GPU-side wait
//!   for the ready value (already reached), then the texture taken from the
//!   engine (Vulkan: owned by `VK_QUEUE_FAMILY_EXTERNAL` in
//!   `SHADER_READ_ONLY_OPTIMAL`, acquired keeping the layout; Direct3D 12:
//!   `COMMON` to the shader-resource state wgpu's tracker holds).
//! - [`Import::release`] after it: the texture given back, and
//!   [`Import::signal`] the release fence (after every earlier submission,
//!   by submission order).
//!
//! An import is prepared once ([`Import::new`]): wgpu's tracker is moved
//! to the sampled state (`transition_resources`) while the textures hold
//! nothing yet, and each texture is given to the engine, so wgpu's first
//! use does not start from an undefined layout.
//!
//! Invariants:
//! - `unsafe` lives in the backend modules (`dx12`, `vulkan`), each block
//!   with its `SAFETY` note; the crate root denies it.
//! - The raw submissions go to wgpu's own queue (`VkQueue`,
//!   `ID3D12CommandQueue`), which needs external synchronization with
//!   wgpu's `Queue::submit`, `present` and `poll`: they are made only on
//!   the thread that owns the device's context, between its calls to wgpu
//!   (`arvane-render`'s `GpuContext` is used serially on its owner's
//!   thread).
//! - An import keeps a clone of its `wgpu::Device`, so the native device
//!   outlives the raw objects; dropping it waits (bounded) for the queue
//!   before destroying them and the textures, and keeps them all for the
//!   process instead when the queue does not finish.
//! - Waits are bounded; failures are `Err`, never panics.
//!
//! Platforms: Direct3D 12 (Windows) and Vulkan (Windows, Linux with
//! `OPAQUE_FD` handles). Elsewhere [`identity`] is `None`, devices open as
//! wgpu opens them and every import fails.

#![deny(unsafe_code)]

mod device;
#[cfg(windows)]
mod dx12;
mod idle;
mod pool;
#[cfg(any(windows, target_os = "linux"))]
mod vulkan;

pub use device::{Opened, identity, request_device};
pub use pool::Import;
