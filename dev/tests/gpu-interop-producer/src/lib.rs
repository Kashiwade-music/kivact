//! Role: A render engine of its own device that shares frames with Arvane, for tests.

#![deny(clippy::too_many_lines, clippy::disallowed_types)]
//!
//! The engine an application would bring, for the GPU texture sharing
//! tests (`crates/arvane/tests/e2e/external.rs`): a device of its own (not
//! wgpu) on the adapter a window reports ([`kivact_types::GpuAdapter`]),
//! Direct3D 12 or Vulkan, with a pool of [`POOL`] textures and two
//! timeline fences from `kivact-gpu-share`, which keeps the
//! engine's side of the contract ([`kivact_types::SharedTextures`]): the
//! hand-off barriers, the `ready` values, a GPU-side wait for `release`
//! before a texture is written again and a bounded CPU wait ([`WAIT`]) for
//! the texture's last frame, so a release that never arrives is an error,
//! not a hang. Each frame is the known picture of [`pattern`], copied into
//! the frame's texture from a buffer the CPU fills.
//!
//! On Direct3D 12 the device is independent of wgpu's (the device factory
//! with `DISALLOW_STORING_NEW_DEVICE_AS_SINGLETON`; without it,
//! `D3D12CreateDevice` hands back the process's singleton, the same device
//! wgpu has). On Vulkan it is a device of its own instance.
//!
//! With the feature `wgpu`, [`Producer::with_wgpu`] is the same engine
//! written with Kivact's wgpu (`kivact_gpu_share::wgpu`): its own
//! `wgpu::Instance` and device on the adapter (on Direct3D 12, the
//! process's singleton device, as the window's wgpu has), the picture
//! written to a texture of its own and copied into the pool's texture.

mod adapters;
#[cfg(windows)]
mod d3d12;
mod engine;
pub mod pattern;
mod producer;
#[cfg(any(windows, target_os = "linux"))]
mod vulkan;
#[cfg(all(feature = "wgpu", any(windows, target_os = "linux")))]
mod wgpu_engine;

pub use adapters::adapters;
pub use engine::{POOL, WAIT};
pub use producer::Producer;
