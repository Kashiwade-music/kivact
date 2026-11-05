//! Role: Reads an engine's Direct3D 12 texture back to the CPU as straight RGBA8.
//!
//! For a window that cannot import (it draws on the CPU, or its GPU is not
//! up yet): the UI shows the frame as `Pixels`. That needs no shared pool:
//! the engine draws into a texture of its own and reads that back.

#![allow(
    unsafe_code,
    reason = "the Direct3D 12 calls of a read back, below any safe API"
)]

use super::raw::{FORMAT, transition, wait};
use crate::common::{Error, api};
use kivact_types::color::unpremultiply;
use std::time::Duration;
use windows::Win32::Graphics::Direct3D12::{
    D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_FENCE_FLAG_NONE, D3D12_HEAP_FLAG_NONE,
    D3D12_HEAP_PROPERTIES, D3D12_HEAP_TYPE_READBACK, D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    D3D12_RESOURCE_DESC, D3D12_RESOURCE_DIMENSION_BUFFER, D3D12_RESOURCE_DIMENSION_TEXTURE2D,
    D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATES,
    D3D12_TEXTURE_COPY_LOCATION, D3D12_TEXTURE_COPY_LOCATION_0,
    D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT, D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
    D3D12_TEXTURE_LAYOUT_ROW_MAJOR, ID3D12CommandAllocator, ID3D12CommandList, ID3D12CommandQueue,
    ID3D12Device, ID3D12Fence, ID3D12GraphicsCommandList, ID3D12Resource,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC};
use windows::core::Interface;

/// `texture`'s pixels as straight-alpha RGBA8 (`width` × `height` × 4
/// bytes, rows top to bottom), copied on `queue` after its earlier work
/// there and waited for at most `timeout`. The texture is a 2D texture of
/// `d3d12::FORMAT` (premultiplied alpha), 1 mip level, 1 array layer, in
/// `state`, and is left in `state`.
///
/// # Safety
/// `queue` is a direct queue of `device`; `texture` is a resource of
/// `device` in `state`, not used meanwhile by other queues (their work on
/// it finished or is ordered before this call's work).
pub unsafe fn read_back(
    device: &ID3D12Device,
    queue: &ID3D12CommandQueue,
    texture: &ID3D12Resource,
    state: D3D12_RESOURCE_STATES,
    timeout: Duration,
) -> Result<Vec<u8>, Error> {
    // SAFETY: a live resource.
    let desc = unsafe { texture.GetDesc() };
    if desc.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE2D
        || desc.Format != FORMAT
        || desc.MipLevels != 1
        || desc.DepthOrArraySize != 1
    {
        return Err(Error::Mismatch(
            "not a 2D RGBA8 texture of one mip level and layer".into(),
        ));
    }
    let (width, height) = (
        u32::try_from(desc.Width).map_err(|_| Error::Mismatch("a texture too wide".into()))?,
        desc.Height,
    );
    let mut footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
    let mut total = 0u64;
    // SAFETY: a texture's description; outputs for one subresource.
    unsafe {
        device.GetCopyableFootprints(
            &desc,
            0,
            1,
            0,
            Some(&mut footprint),
            None,
            None,
            Some(&mut total),
        );
    }
    // SAFETY: the device and checked footprint size describe this staging allocation.
    let readback = unsafe { staging_buffer(device, total) }?;
    // SAFETY: a valid list type.
    let allocator: ID3D12CommandAllocator =
        unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
            .map_err(api("CreateCommandAllocator"))?;
    // SAFETY: a new list of a live allocator.
    let list: ID3D12GraphicsCommandList =
        unsafe { device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None) }
            .map_err(api("CreateCommandList"))?;
    // SAFETY: the open list; live resources (as the caller says).
    unsafe { record(&list, texture, &readback, state, footprint) };
    // SAFETY: the recorded list.
    unsafe { list.Close() }.map_err(api("Close"))?;
    // SAFETY: a new fence of the device.
    let fence: ID3D12Fence =
        unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }.map_err(api("CreateFence"))?;
    let lists = [Some(
        list.cast::<ID3D12CommandList>()
            .map_err(api("QueryInterface"))?,
    )];
    // SAFETY: the closed list, on the caller's queue of the device.
    unsafe { queue.ExecuteCommandLists(&lists) };
    // SAFETY: a GPU-side signal after the copy.
    unsafe { queue.Signal(&fence, 1) }.map_err(api("ID3D12CommandQueue::Signal"))?;
    if let Err(error) = wait(&fence, 1, timeout, "the read back") {
        // The copy may still run: keep what it uses.
        std::mem::forget((readback, allocator, list, fence));
        return Err(error);
    }
    let mut mapped = std::ptr::null_mut();
    // SAFETY: a readback buffer the finished copy wrote.
    unsafe { readback.Map(0, None, Some(&mut mapped)) }.map_err(api("Map"))?;
    let row = width as usize * 4;
    let pitch = footprint.Footprint.RowPitch as usize;
    let mut rgba = vec![0u8; row * height as usize];
    let base = mapped.cast::<u8>().cast_const();
    for (y, out) in rgba.chunks_exact_mut(row).enumerate() {
        // SAFETY: the mapping holds `height` rows `pitch` bytes apart from
        // the footprint's offset, each at least `row` bytes.
        let source = unsafe { base.add(footprint.Offset as usize + y * pitch) };
        // SAFETY: as above; `out` is a row of the result.
        unsafe { std::ptr::copy_nonoverlapping(source, out.as_mut_ptr(), row) };
    }
    // SAFETY: the mapped buffer; nothing was written.
    unsafe { readback.Unmap(0, None) };
    unpremultiply(&mut rgba);
    Ok(rgba)
}

/// Allocate a CPU-visible staging buffer on the caller's device.
///
/// # Safety
/// `device` is live and `total` is a valid copy-footprint size.
unsafe fn staging_buffer(device: &ID3D12Device, total: u64) -> Result<ID3D12Resource, Error> {
    let buffer = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Width: total,
        Height: 1,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_UNKNOWN,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
        ..Default::default()
    };
    let properties = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_READBACK,
        ..Default::default()
    };
    let mut readback: Option<ID3D12Resource> = None;
    // SAFETY: valid descriptions for the call.
    unsafe {
        device.CreateCommittedResource(
            &properties,
            D3D12_HEAP_FLAG_NONE,
            &buffer,
            D3D12_RESOURCE_STATE_COPY_DEST,
            None,
            &mut readback,
        )
    }
    .map_err(api("CreateCommittedResource"))?;
    readback.ok_or_else(|| Error::Api {
        call: "CreateCommittedResource",
        message: "no resource".into(),
    })
}

/// Record the copy of `texture` into `readback`, from and back to
/// `state`.
///
/// # Safety
/// `list` is open; `texture` and `readback` are live resources of its
/// device, `readback` a buffer of the footprint's size.
unsafe fn record(
    list: &ID3D12GraphicsCommandList,
    texture: &ID3D12Resource,
    readback: &ID3D12Resource,
    state: D3D12_RESOURCE_STATES,
    footprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
) {
    let source_state = D3D12_RESOURCE_STATE_COPY_SOURCE;
    if state != source_state {
        // SAFETY: recording into the open list.
        unsafe { list.ResourceBarrier(&[transition(texture, state, source_state)]) };
    }
    let target = D3D12_TEXTURE_COPY_LOCATION {
        // SAFETY: a copy of the pointer, never released (the location
        // drops nothing); the buffer outlives the recording.
        pResource: unsafe { std::mem::transmute_copy(readback) },
        Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
        Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
            PlacedFootprint: footprint,
        },
    };
    let source = D3D12_TEXTURE_COPY_LOCATION {
        // SAFETY: as above, of the texture.
        pResource: unsafe { std::mem::transmute_copy(texture) },
        Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
        Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
            SubresourceIndex: 0,
        },
    };
    // SAFETY: as above, with live resources.
    unsafe { list.CopyTextureRegion(&target, 0, 0, 0, &source, None) };
    if state != source_state {
        // SAFETY: as above.
        unsafe { list.ResourceBarrier(&[transition(texture, source_state, state)]) };
    }
}
