//! Role: The Direct3D 12 half of the wgpu helpers: device, wrapped textures, hand-off.
//!
//! Devices: wgpu's own `request_device`; shared heaps and fences need no
//! more. Pools: the backend's pool ([`crate::d3d12::Pool`]) makes the
//! shared textures and fences on wgpu's `ID3D12Device`, and each texture
//! is wrapped for wgpu (`texture_from_raw`, holding a reference). Two
//! closed command lists per texture, recorded once, move it between
//! `COMMON` (the hand-off state) and the shader-resource state wgpu's
//! tracker holds for `TextureUses::RESOURCE`
//! (`PIXEL_SHADER_RESOURCE | NON_PIXEL_SHADER_RESOURCE`); a texture's
//! first frame skips the first (wgpu's tracker starts it at
//! `UNINITIALIZED`, which is `COMMON`).

use crate::common::{Error, api};
use crate::d3d12::raw::transition;
use crate::d3d12::{self as backend, HAND_OFF_STATE};
use crate::pool::{Acquired, FrameReady, PoolConfig};
use ::wgpu::hal::api::Dx12;
use kivact_types::{GpuAdapter, SharedTextures};
use std::time::Duration;
use windows::Win32::Graphics::Direct3D12::{
    D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE, D3D12_RESOURCE_STATES, ID3D12CommandAllocator,
    ID3D12CommandList, ID3D12CommandQueue, ID3D12GraphicsCommandList, ID3D12Resource,
};
use windows::core::Interface;

/// The state wgpu's tracker holds for `TextureUses::RESOURCE`
/// (`wgpu_hal::dx12::conv::map_texture_usage_to_state`).
const SAMPLED: D3D12_RESOURCE_STATES = D3D12_RESOURCE_STATES(
    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE.0 | D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE.0,
);

/// Whether the Direct3D 12 adapter `found` is `adapter` (its LUID).
pub(super) fn is_adapter(found: &::wgpu::Adapter, adapter: &GpuAdapter) -> Result<bool, Error> {
    let wanted = backend::luid(adapter)?;
    // SAFETY: only reads the adapter's description while the guard lives.
    let Some(hal) = (unsafe { found.as_hal::<Dx12>() }) else {
        return Ok(false);
    };
    // SAFETY: a live DXGI adapter.
    let desc = unsafe { hal.raw_adapter().GetDesc2() }.map_err(api("GetDesc2"))?;
    Ok(crate::d3d12_contract::luid_bytes(desc.AdapterLuid) == wanted)
}

/// `adapter.request_device(desc)`: a Direct3D 12 device shares as it is.
pub(super) fn open(
    adapter: ::wgpu::Adapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Result<(::wgpu::Adapter, ::wgpu::Device, ::wgpu::Queue), Error> {
    let (device, queue) =
        pollster::block_on(adapter.request_device(desc)).map_err(api("Adapter::request_device"))?;
    Ok((adapter, device, queue))
}

/// One texture's transitions out of and back to the hand-off state.
struct Lists {
    acquire: ID3D12CommandList,
    release: ID3D12CommandList,
}

/// A pool's Direct3D 12 half: the backend's pool and the hand-off on
/// wgpu's queue.
pub(super) struct Raw {
    pool: backend::Pool,
    queue: ID3D12CommandQueue,
    /// Keeps the recorded lists' memory.
    _allocator: ID3D12CommandAllocator,
    lists: Vec<Lists>,
}

impl Raw {
    /// A pool on the Direct3D 12 `device`, its textures wrapped as `desc`
    /// says, and its handles.
    pub(super) fn new(
        device: &::wgpu::Device,
        adapter: &GpuAdapter,
        config: PoolConfig,
        desc: &::wgpu::TextureDescriptor<'_>,
    ) -> Result<(Self, Vec<::wgpu::Texture>, SharedTextures), Error> {
        let (raw, queue) = {
            // SAFETY: only clones the device's interfaces while the guard
            // lives.
            let hal = unsafe { device.as_hal::<Dx12>() }
                .ok_or_else(|| Error::Mismatch("not a Direct3D 12 device".into()))?;
            (hal.raw_device().clone(), hal.raw_queue().clone())
        };
        let (pool, shared) = backend::Pool::new(&raw, adapter, config)?;
        // SAFETY: a valid list type.
        let allocator: ID3D12CommandAllocator =
            unsafe { raw.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
                .map_err(api("CreateCommandAllocator"))?;
        let list = |resource: &ID3D12Resource, before, after| -> Result<ID3D12CommandList, Error> {
            // SAFETY: a new list of a live allocator of this device.
            let list: ID3D12GraphicsCommandList = unsafe {
                raw.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)
            }
            .map_err(api("CreateCommandList"))?;
            // SAFETY: recording into the open list; `resource` lives.
            unsafe { list.ResourceBarrier(&[transition(resource, before, after)]) };
            // SAFETY: the open list.
            unsafe { list.Close() }.map_err(api("ID3D12GraphicsCommandList::Close"))?;
            list.cast().map_err(api("QueryInterface"))
        };
        let mut lists = Vec::with_capacity(pool.textures().len());
        let mut textures = Vec::with_capacity(pool.textures().len());
        for resource in pool.textures() {
            lists.push(Lists {
                acquire: list(resource, HAND_OFF_STATE, SAMPLED)?,
                release: list(resource, SAMPLED, HAND_OFF_STATE)?,
            });
            // SAFETY: the resource is a 2D texture of `desc`'s format, size,
            // one mip level and sample (the pool made it so).
            let texture = unsafe {
                ::wgpu::hal::dx12::Device::texture_from_raw(
                    resource.clone(),
                    desc.format,
                    desc.dimension,
                    desc.size,
                    1,
                    1,
                )
            };
            // SAFETY: a texture of this device, made as `desc` says (its
            // flags allow the render target and, with `storage`, unordered
            // access the usage names).
            textures.push(unsafe { device.create_texture_from_hal::<Dx12>(texture, desc) });
        }
        Ok((
            Self {
                pool,
                queue,
                _allocator: allocator,
                lists,
            },
            textures,
            shared,
        ))
    }

    pub(super) fn pool(&self) -> &backend::Pool {
        &self.pool
    }

    /// Step 1: wait for `release` and, unless it is the texture's first
    /// frame, move it from `COMMON` to the shader-resource state.
    ///
    /// # Safety
    /// The order of wgpu's queue is not changed by another thread during
    /// the call.
    pub(super) unsafe fn take(&mut self, acquired: Acquired) -> Result<(), Error> {
        if let Some(wait) = acquired.wait {
            // SAFETY: a GPU-side wait on a live fence of the device.
            unsafe { self.queue.Wait(self.pool.release_fence(), wait.get()) }
                .map_err(api("ID3D12CommandQueue::Wait"))?;
            // Not the texture's first frame (wgpu's tracker starts a texture
            // at `UNINITIALIZED`, which is `COMMON`): from `COMMON` to the
            // sampled state.
            let list = self.lists[acquired.slot as usize].acquire.clone();
            // SAFETY: a closed list of this device, on wgpu's queue between
            // wgpu's submissions (the caller).
            unsafe { self.queue.ExecuteCommandLists(&[Some(list)]) };
        }
        Ok(())
    }

    /// Step 4: move the frame's texture back to `COMMON` and signal
    /// `ready`.
    ///
    /// # Safety
    /// As [`Raw::take`].
    pub(super) unsafe fn give(&mut self, acquired: Acquired) -> Result<FrameReady, Error> {
        let list = self.lists[acquired.slot as usize].release.clone();
        // SAFETY: wgpu's direct queue of the pool's device; a closed list of
        // the device.
        unsafe {
            self.pool
                .submit_acquired(&self.queue, acquired, &[Some(list)])
        }
    }

    /// Texture `slot`'s last frame, read back on wgpu's queue.
    ///
    /// # Safety
    /// As [`crate::wgpu::Pool::read_back`].
    pub(super) unsafe fn read_back(&self, slot: u32, timeout: Duration) -> Result<Vec<u8>, Error> {
        // SAFETY: wgpu's direct queue of the pool's device; the rest as the
        // caller says.
        unsafe { self.pool.read_back(&self.queue, slot, timeout) }
    }
}
