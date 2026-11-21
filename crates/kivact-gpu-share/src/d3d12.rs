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

use windows::Win32::Graphics::Direct3D12::{
    D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
    D3D12_RESOURCE_FLAGS, D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATES,
};
// The transition barrier exists only where a side records one.
#[cfg(any(feature = "export", feature = "import"))]
use std::mem::ManuallyDrop;
#[cfg(any(feature = "export", feature = "import"))]
use windows::Win32::Graphics::Direct3D12::{
    D3D12_RESOURCE_BARRIER, D3D12_RESOURCE_BARRIER_0, D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
    D3D12_RESOURCE_BARRIER_FLAG_NONE, D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
    D3D12_RESOURCE_TRANSITION_BARRIER, ID3D12Resource,
};

#[cfg(feature = "export")]
mod adapter;
#[cfg(feature = "export")]
mod pool;
#[cfg(feature = "export")]
pub(crate) mod raw;
#[cfg(feature = "export")]
mod read_back;

/// The state a texture is in between the window and the engine.
pub const HAND_OFF_STATE: D3D12_RESOURCE_STATES = D3D12_RESOURCE_STATE_COMMON;

/// The resource flags required for a shared texture.
pub fn resource_flags(storage: bool) -> D3D12_RESOURCE_FLAGS {
    if storage {
        D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET | D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS
    } else {
        D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET
    }
}

/// A transition barrier borrowing `resource` (no reference is taken).
#[cfg(any(feature = "export", feature = "import"))]
pub(crate) fn transition(
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) -> D3D12_RESOURCE_BARRIER {
    D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                // SAFETY: a copy of the pointer in `ManuallyDrop`, never
                // released; `resource` outlives the recording.
                pResource: unsafe { std::mem::transmute_copy(resource) },
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    }
}

/// A DXGI adapter LUID in the byte order used by `AdapterMatch::Luid`.
pub fn luid_bytes(luid: windows::Win32::Foundation::LUID) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes[..4].copy_from_slice(&luid.LowPart.to_le_bytes());
    bytes[4..].copy_from_slice(&luid.HighPart.to_le_bytes());
    bytes
}

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
