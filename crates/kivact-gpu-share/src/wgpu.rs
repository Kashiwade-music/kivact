//! Role: A render engine's side of GPU texture sharing on Kivact's wgpu (29.0.4).
//!
//! For an engine written with wgpu of the version Kivact uses (feature
//! `wgpu`; the dependency is pinned to `=29.0.4`, arvane-render's, and the
//! two are raised together). [`request_device`] opens a device on the
//! adapter the window reported, able to share; [`Pool::new`] makes the
//! pool of the contract ([`kivact_types::SharedTextures`]) with the backend's
//! pool (`d3d12::Pool`, `vulkan::Pool`) and wraps its textures as
//! `wgpu::Texture`s (`create_texture_from_hal`) of the usage [`usage`]
//! says. A frame: [`Pool::acquire`] gives a [`Target`], the engine
//! records its own command buffers on the target's texture, and
//! [`Target::submit`] submits them between the hand-off:
//!
//! 1. a raw submission on wgpu's queue: a GPU-side wait for the window's
//!    `release` of the texture's last frame, then the texture taken from
//!    the window (Vulkan: acquired from `VK_QUEUE_FAMILY_EXTERNAL` keeping
//!    `SHADER_READ_ONLY_OPTIMAL`; Direct3D 12: `COMMON` to the
//!    shader-resource state);
//! 2. `Queue::submit` of the engine's command buffers and
//! 3. of an encoder that moves wgpu's tracker of the texture to
//!    `TextureUses::RESOURCE` (`transition_resources`), the state of step 1;
//! 4. a raw submission: the texture given back (released to
//!    `VK_QUEUE_FAMILY_EXTERNAL`; back to `COMMON`) and `ready` signaled.
//!
//! So wgpu's tracker always names the texture's real state when wgpu's
//! commands run. wgpu starts a texture from `UNINITIALIZED` (`UNDEFINED`,
//! `COMMON`): a texture's first frame skips Direct3D 12's move to the
//! shader-resource state, and on Vulkan wgpu's first barrier discards the
//! contents (the window's preparation left nothing in them anyway).
//!
//! Invariants:
//! - The raw submissions go to wgpu's own queue (`VkQueue`,
//!   `ID3D12CommandQueue`), which needs external synchronization with
//!   wgpu's use of it: [`Target::submit`] and [`Pool::read_back`] are
//!   `unsafe fn`s whose callers keep every other use of the device's
//!   queue off other threads during the call.
//! - The pool keeps a clone of the `wgpu::Device`; wgpu-hal owns the
//!   Vulkan images and memory (handed over by the backend's pool), and
//!   dropping the pool waits (bounded) for its last frame, leaking the
//!   textures and raw objects instead when it does not finish.
//! - Every `unsafe` block has its `SAFETY` note (here and in the backend
//!   halves `vulkan` and `d3d12`).

#![allow(
    unsafe_code,
    reason = "raw submissions on wgpu's queue and wgpu's hal entry points, below wgpu's safe API"
)]

#[cfg(windows)]
mod d3d12;
mod device;
mod pool;
mod read_back;
mod vulkan;

pub use device::request_device;
pub use pool::{FORMAT, Pool, Target, usage};
pub use read_back::read_back;
