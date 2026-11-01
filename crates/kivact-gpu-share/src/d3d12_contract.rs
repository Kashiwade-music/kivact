//! Role: Direct3D 12 flags and shared resource transition descriptions.

#![allow(
    unsafe_code,
    reason = "a borrowed COM resource pointer in a transition barrier"
)]

#[cfg(any(feature = "export", feature = "import"))]
use std::mem::ManuallyDrop;
use windows::Win32::Graphics::Direct3D12::{
    D3D12_RESOURCE_BARRIER, D3D12_RESOURCE_BARRIER_0, D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
    D3D12_RESOURCE_BARRIER_FLAG_NONE, D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
    D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
    D3D12_RESOURCE_FLAGS, D3D12_RESOURCE_STATES, D3D12_RESOURCE_TRANSITION_BARRIER, ID3D12Resource,
};

/// The resource flags required for a shared texture.
pub fn resource_flags(storage: bool) -> D3D12_RESOURCE_FLAGS {
    if storage {
        D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET | D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS
    } else {
        D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET
    }
}

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
