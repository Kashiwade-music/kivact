//! Role: Contracts and native import and export helpers for GPU texture sharing.

//!
//! Without features, this crate exposes shared identities and errors.
//! Feature `export` enables native pools; `wgpu` includes `export` and
//! adds wgpu engine helpers. Feature `import` enables the `import` module.
//!
//! An application's render engine (its own device, not the window's)
//! shares its frames with a Kivact window through
//! [`kivact_types::SharedTextures`] (re-exported as `kivact::gpu_interop`),
//! whose rustdoc is the engine's contract. This crate keeps that contract
//! for the engine: it makes the pool of textures and the two timeline
//! fences as the contract says and exports them once, moves a texture
//! between the window and the engine (Vulkan's queue family ownership,
//! Direct3D 12's `COMMON` state), counts the `ready` values, makes the
//! GPU wait for the window's `release` before a texture is written again
//! and bounds every CPU wait. The engine writes only the drawing into the
//! texture it is given.
//!
//! What the engine records for that drawing is its own, and so are the
//! contract's other rules. Breaking the contract is undefined behavior of
//! the graphics API (Vulkan and Direct3D 12): a broken picture, a lost
//! device, a crashed process, or access to GPU memory outside the
//! resource are all possible. Kivact and Arvane cannot check most of it,
//! which is why handing a pool over (`record_acquire`, `record_release`,
//! `submit`, and the window's `import`) is `unsafe`.
//!
//! It does not know the UI: an engine thread depends on it, not on
//! `kivact`. The modules are `vulkan` (Windows and Linux, on
//! [`ash`](https://docs.rs/ash/0.38)), `d3d12` (Windows, on
//! [`windows`](https://docs.rs/windows/0.62)) and, with the feature
//! `wgpu` (off by default), `wgpu` (Windows and Linux, for an engine on
//! wgpu 29.0.4, Kivact's version); on other platforms only the shared vocabulary is available.
//!
//! # A frame
//!
//! The engine's device is on the adapter the window reports
//! (`kivact::gpu_interop::adapter()`, a [`kivact_types::GpuAdapter`]), with
//! the window's graphics API. One pool per import:
//!
//! ```ignore
//! use kivact_gpu_share::{PoolConfig, vulkan};
//! use std::time::Duration;
//!
//! // Once: the physical device the window reported, a device of it with
//! // `vulkan::DEVICE_EXTENSIONS`, API 1.2 and `timelineSemaphore`.
//! let physical = unsafe { vulkan::find_physical_device(&instance, &adapter)? };
//! let (mut pool, shared) = unsafe {
//!     vulkan::Pool::new(&instance, physical, &device, family, &adapter, PoolConfig::new(1920, 1080))?
//! };
//! send_to_ui(shared); // the UI calls `unsafe { gpu_interop::import(shared) }`
//!
//! // Each frame, once the import is `Ready`:
//! let target = pool.acquire(Duration::from_secs(1))?; // 1. the texture (borrows the pool)
//! begin(command);
//! let write = (vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
//! unsafe { target.record_acquire(command, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, write) }; // 2.
//! draw_into(command, target.image(), target.extent()); // 3. the engine's own drawing
//! unsafe { target.record_release(command, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, write) }; // 4.
//! end(command);
//! let ready = unsafe { target.submit(queue, &[command], vk::Fence::null())? }; // 5. signal
//! send_to_ui(ready.frame(image_id)); // 6. publish: the UI draws `GpuFrame` of it
//! ```
//!
//! Direct3D 12 is the same with `d3d12::Pool`: `acquire`, then on its
//! target `record_acquire` (from `COMMON` to the engine's state), the
//! drawing, `record_release` (back to `COMMON`), and `submit` on the
//! engine's `ID3D12CommandQueue` (the GPU-side `Wait` for `release`, the
//! command lists, the `Signal` of `ready`).
//!
//! An engine that submits on its own takes the fences and values from
//! the target's `finish` instead of `submit` (`vulkan::Submission`,
//! `d3d12::Submission`): a GPU-side wait for `release` before the
//! frame's work and a signal of `ready` after it.
//!
//! # Waiting
//!
//! The GPU waits for the window's `release` of a texture's last frame (in
//! the frame's submission), so the engine can draw ahead of the window.
//! The CPU waits only in `acquire`, bounded, for the texture's last frame
//! to have signaled `ready`; that frame's submission waited for the
//! release before it, so a release that never comes (a lost device, a
//! window that stopped presenting) is an [`Error::Timeout`] one round of
//! the pool later, not a hang. `wait_release` waits on the CPU for a
//! `release` value, bounded, for an engine that wants to know.
//!
//! # Showing frames as `Pixels`
//!
//! A window that cannot import (it draws on the CPU, or its GPU is not up
//! yet) shows the frame as `Pixels`. That needs no shared pool: the engine
//! draws into a texture of its own and reads it back with
//! `vulkan::read_back` or `d3d12::read_back` (straight-alpha RGBA8).
//! A pool's texture reads back with the pools' `read_back`.
//!
//! # An engine on wgpu (feature `wgpu`)
//!
//! The dependency is pinned to wgpu `=29.0.4`, the version Kivact's
//! window (arvane-render) uses; the two are raised together, since the
//! module reaches below wgpu's safe API (`as_hal`, `open_with_callback`,
//! `create_texture_from_hal`). `wgpu::request_device` opens a device on
//! the window's adapter able to share; `wgpu::Pool` makes the pool and
//! gives each frame's texture as a `wgpu::Texture`; the target's `submit`
//! puts the engine's command buffers between the hand-off on wgpu's queue.
//! A frame:
//!
//! ```ignore
//! use kivact_gpu_share::{PoolConfig, wgpu as share};
//! use std::time::Duration;
//!
//! // Once: a device on the adapter the window reported, and the pool.
//! // (No `InstanceFlags::DEBUG`/`VALIDATION` next to a Direct3D 12 window.)
//! let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
//!     flags: wgpu::InstanceFlags::empty().with_env(),
//!     ..wgpu::InstanceDescriptor::new_without_display_handle()
//! });
//! let (_, device, queue) = share::request_device(&instance, &adapter, &Default::default())?;
//! let (mut pool, shared) = share::Pool::new(&device, &adapter, PoolConfig::new(1920, 1080))?;
//! send_to_ui(shared); // the UI imports it (`ShareLink::provide`)
//!
//! // Each frame, once the import is ready:
//! let target = pool.acquire(Duration::from_secs(1))?; // 1. the texture
//! let view = target.create_view(&Default::default());
//! let mut encoder = device.create_command_encoder(&Default::default());
//! draw_into(&mut encoder, &view); // 2. a render pass (`LoadOp::Load` keeps the slot's last frame)
//! // 3. submit between the hand-off; the queue is not used by another
//! //    thread meanwhile (it gets raw submissions too).
//! let ready = unsafe { target.submit(&queue, [encoder.finish()])? };
//! send_to_ui(ready.frame(image_id)); // 4. publish
//! ```
//!
//! `wgpu::read_back` reads a texture of the engine's own back with wgpu
//! alone (for `Pixels`).
//!
//! # An engine on wgpu of another version
//!
//! An engine on its own wgpu takes the native handles with `as_hal`
//! (`wgpu::Device::as_hal::<wgpu::hal::api::Vulkan>()` gives the `ash`
//! instance, physical device, device and queue family; the Direct3D 12
//! backend gives the `ID3D12Device` and queue). Its `ash` and `windows`
//! must be this crate's versions (0.38 and 0.62) for the handles to cross.
//! A wgpu device does not enable `vulkan::DEVICE_EXTENSIONS`: the engine
//! opens the device itself with them added (wgpu's `open_with_callback`,
//! or `create_device_from_hal` over a device made with ash). The pool's
//! raw submissions go to wgpu's queue, which needs external
//! synchronization with wgpu's own `submit`: they are made on the thread
//! that uses that queue, between wgpu's calls.
//!
//! # Limits
//!
//! - The first acquire of a Vulkan texture takes it from
//!   `VK_QUEUE_FAMILY_EXTERNAL` in `SHADER_READ_ONLY_OPTIMAL`, where the
//!   window's import put it. A pool never imported (its textures still
//!   `UNDEFINED`, owned by nobody) is outside Vulkan's rules there, though
//!   drivers take it; an engine showing `Pixels` reads back a texture of
//!   its own instead.
//! - The engine draws one frame at a time: `acquire` gives the textures
//!   round robin, and its target borrows the pool until it is submitted
//!   (or dropped, which gives the frame up).

#![deny(unsafe_code)]

mod common;
#[cfg(all(feature = "export", windows))]
pub mod d3d12;
#[cfg(windows)]
pub mod d3d12_contract;
#[cfg(feature = "export")]
mod pool;
#[cfg(all(feature = "export", any(windows, target_os = "linux")))]
pub mod vulkan;
#[cfg(any(windows, target_os = "linux"))]
pub mod vulkan_contract;
#[cfg(all(
    any(feature = "import", feature = "export"),
    any(windows, target_os = "linux")
))]
mod vulkan_raw;
#[cfg(all(feature = "wgpu", any(windows, target_os = "linux")))]
pub mod wgpu;
#[cfg(all(
    any(feature = "import", feature = "wgpu"),
    any(windows, target_os = "linux")
))]
mod wgpu_device;

pub use common::{Error, Identity};
#[cfg(feature = "export")]
pub use pool::{DEFAULT_SLOTS, FrameReady, PoolConfig};
#[cfg(feature = "import")]
pub mod device;
#[cfg(feature = "import")]
pub mod import;
