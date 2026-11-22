//! Role: A shared pool on the engine's own Direct3D 12 device: textures, fences.
//!
//! The engine brings its `ID3D12Device` on the adapter the window reported
//! ([`find_adapter`] finds it by LUID). [`Pool::new`] makes the textures
//! in shared heaps and the two shared fences as
//! [`kivact_types::SharedTextures`] says and makes their NT handles once. A
//! frame: [`Pool::acquire`] gives a [`Target`]; its
//! [`Target::record_acquire`] (from `COMMON`, the hand-off state), the
//! engine's own drawing, [`Target::record_release`] (back to `COMMON`),
//! then [`Target::submit`] on the engine's queue (or the engine's own
//! `Wait`, lists and `Signal` with [`Target::finish`]'s fences).
//!
//! The device: in one process, `D3D12CreateDevice` on an adapter returns
//! the process's singleton device of that adapter, the very device the
//! window's wgpu made. The sharing works on it as well (the window opens
//! the handles on the device that made them; the engine's queue is its
//! own): an engine needs no independent device, though it may make one
//! (`ID3D12DeviceFactory` with
//! `D3D12_DEVICE_FACTORY_FLAG_DISALLOW_STORING_NEW_DEVICE_AS_SINGLETON`)
//! to keep its device's lifetime and removal apart from the window's.
//!
//! Invariants:
//! - A texture is written again only after the queue waited for
//!   `release` to reach its last frame's value (a GPU-side `Wait`); the
//!   CPU waits, bounded, only for the texture's last `ready`.
//! - Dropping the pool waits (bounded) for the last frame's `ready` and
//!   leaks the textures and fences instead when it does not arrive.

#![allow(
    unsafe_code,
    reason = "the Direct3D 12 calls of an engine's shared pool, which windows marks unsafe"
)]

mod contract;

#[cfg(feature = "export")]
mod adapter;
#[cfg(feature = "export")]
mod pool;
#[cfg(feature = "export")]
pub(crate) mod raw;
#[cfg(feature = "export")]
mod read_back;

#[cfg(feature = "export")]
pub use adapter::find_adapter;
#[cfg(feature = "wgpu")]
pub(crate) use adapter::luid;
#[cfg(feature = "export")]
pub use pool::{Pool, Submission, Target};
#[cfg(feature = "export")]
pub use raw::FORMAT;
#[cfg(feature = "export")]
pub use read_back::read_back;

#[cfg(feature = "import")]
pub(crate) use contract::transition;
pub use contract::{HAND_OFF_STATE, luid_bytes, resource_flags};
