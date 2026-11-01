//! Role: Direct3D 12 imports of shared resources and fences onto wgpu's queue.
//!
//! Imports: the shared resources and fences opened on wgpu's
//! `ID3D12Device` (`OpenSharedHandle`, which does not take the NT handle:
//! it is closed after), each resource's description checked against the
//! engine's contract (its flags against the textures' `storage`), wrapped
//! by `wgpu_hal::dx12::Device::texture_from_raw` and
//! `create_texture_from_hal`. One closed command list per texture and
//! direction moves it between `COMMON` (the hand-off state) and the
//! shader-resource state wgpu's tracker holds for a sampled texture
//! (`PIXEL_SHADER_RESOURCE | NON_PIXEL_SHADER_RESOURCE`).
//!
//! Submissions on wgpu's `ID3D12CommandQueue`: acquire is `Wait(ready, v)`
//! (a GPU-side wait: later work on the queue starts after it) then the
//! list to the shader-resource state; release is the list back to
//! `COMMON`; a signal is `Signal(release, v)` after the queue's earlier
//! work. An own fence and an event tell when the queue finished (bounded
//! waits).

#![allow(unsafe_code, reason = "Direct3D 12 calls below wgpu's safe API")]

use crate::common::{Error, Identity, same_backend};
use ::wgpu::hal::api::Dx12;
use kivact_types::{GpuBackend, SharedHandle, SharedTextures};
use std::os::windows::io::{AsHandle, AsRawHandle, FromRawHandle, OwnedHandle};
use std::time::Duration;
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D12::{
    D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_FENCE_FLAG_NONE, D3D12_HEAP_FLAG_SHARED,
    D3D12_HEAP_FLAGS, D3D12_RESOURCE_DIMENSION_TEXTURE2D,
    D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS, D3D12_RESOURCE_STATE_COMMON,
    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE, D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
    D3D12_RESOURCE_STATES, ID3D12CommandAllocator, ID3D12CommandList, ID3D12CommandQueue,
    ID3D12Device, ID3D12Fence, ID3D12GraphicsCommandList, ID3D12Resource,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM;
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::core::{Interface, PCWSTR};

/// The state wgpu's tracker holds a sampled texture in
/// (`wgpu_hal::dx12::conv::map_texture_usage_to_state(RESOURCE)`).
const SAMPLED: D3D12_RESOURCE_STATES = D3D12_RESOURCE_STATES(
    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE.0 | D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE.0,
);

/// The identity of a Direct3D 12 adapter (`None`: not Direct3D 12).
pub(crate) fn identity(adapter: &::wgpu::Adapter) -> Option<Identity> {
    // SAFETY: only reads the adapter's description while the guard lives.
    let hal = unsafe { adapter.as_hal::<Dx12>() }?;
    // SAFETY: a live DXGI adapter.
    let desc = unsafe { hal.raw_adapter().GetDesc2() }.ok()?;
    let luid = crate::d3d12_contract::luid_bytes(desc.AdapterLuid);
    Some(Identity {
        backend: GpuBackend::Dx12,
        luid: Some(luid),
        device_uuid: None,
        driver_uuid: None,
    })
}

/// Whether `device` is a Direct3D 12 device.
pub(crate) fn is_dx12(device: &::wgpu::Device) -> bool {
    // SAFETY: only asks for the backend; the guard is dropped at once.
    unsafe { device.as_hal::<Dx12>() }.is_some()
}

/// A transition barrier borrowing `resource` (no reference is taken).
use crate::d3d12_contract::transition;

/// The object `handle` names, opened on `device`.
fn open<T: Interface>(device: &ID3D12Device, handle: &SharedHandle) -> Result<T, Error> {
    let mut out: Option<T> = None;
    // SAFETY: `handle` is a live NT handle; opening does not take it.
    unsafe { device.OpenSharedHandle(HANDLE(handle.as_handle().as_raw_handle()), &mut out) }
        .map_err(crate::common::api("OpenSharedHandle"))?;
    out.ok_or_else(|| "OpenSharedHandle gave nothing".into())
}

/// The resource flags of a shared texture: made for storage writes or
/// not (the engine's contract).
use crate::d3d12_contract::resource_flags;

/// Fail unless `resource` is a texture the engine's contract describes
/// (made for storage writes when `storage`).
fn check(
    resource: &ID3D12Resource,
    (width, height): (u32, u32),
    storage: bool,
) -> Result<(), Error> {
    // SAFETY: a live resource.
    let desc = unsafe { resource.GetDesc() };
    let mut flags = D3D12_HEAP_FLAGS::default();
    // SAFETY: a live committed resource; `flags` outlives the call.
    unsafe { resource.GetHeapProperties(None, Some(&mut flags)) }
        .map_err(crate::common::api("GetHeapProperties"))?;
    if desc.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE2D
        || desc.Format != DXGI_FORMAT_R8G8B8A8_UNORM
        || desc.DepthOrArraySize != 1
        || desc.MipLevels != 1
        || desc.SampleDesc.Count != 1
    {
        return Err(
            "the shared resource is not a 2D RGBA8 texture of one mip, layer and sample".into(),
        );
    }
    if (desc.Width, desc.Height) != (u64::from(width), height) {
        return Err(Error::Mismatch(format!(
            "the shared texture is {} x {}, not {width} x {height}",
            desc.Width, desc.Height
        )));
    }
    if desc
        .Flags
        .contains(D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS)
    {
        return Err("the shared texture allows simultaneous access".into());
    }
    if desc.Flags != resource_flags(storage) {
        return Err(Error::Mismatch(format!(
            "the shared texture's flags {:#x} are not {:#x} (storage {storage})",
            desc.Flags.0,
            resource_flags(storage).0
        )));
    }
    if !flags.contains(D3D12_HEAP_FLAG_SHARED) {
        return Err("the shared texture is not in a shared heap".into());
    }
    Ok(())
}

/// Import `shared` into the Direct3D 12 `device`.
pub(crate) fn import(
    device: &::wgpu::Device,
    shared: SharedTextures,
    desc: &::wgpu::TextureDescriptor<'_>,
) -> Result<(Vec<::wgpu::Texture>, Shared), Error> {
    same_backend(&shared, GpuBackend::Dx12)?;
    let (width, height) = (shared.width, shared.height);
    let (raw, queue) = {
        // SAFETY: only clones the device's interfaces while the guard lives.
        let hal = unsafe { device.as_hal::<Dx12>() }.ok_or("not a Direct3D 12 device")?;
        (hal.raw_device().clone(), hal.raw_queue().clone())
    };
    let mut resources = Vec::with_capacity(shared.textures.len());
    for handle in &shared.textures {
        let resource: ID3D12Resource = open(&raw, handle)?;
        check(&resource, (width, height), shared.storage)?;
        resources.push(resource);
    }
    let ready: ID3D12Fence = open(&raw, &shared.ready)?;
    let release: ID3D12Fence = open(&raw, &shared.release)?;
    // SAFETY: a valid list type.
    let allocator: ID3D12CommandAllocator =
        unsafe { raw.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
            .map_err(crate::common::api("CreateCommandAllocator"))?;
    let list = |resource: &ID3D12Resource, before, after| -> Result<ID3D12CommandList, Error> {
        // SAFETY: a new list of a live allocator of this device.
        let list: ID3D12GraphicsCommandList =
            unsafe { raw.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None) }
                .map_err(crate::common::api("CreateCommandList"))?;
        // SAFETY: recording into the open list; `resource` lives.
        unsafe { list.ResourceBarrier(&[transition(resource, before, after)]) };
        // SAFETY: the open list.
        unsafe { list.Close() }.map_err(crate::common::api("closing a command list"))?;
        list.cast()
            .map_err(crate::common::api("casting a command list"))
    };
    let mut slots = Vec::with_capacity(resources.len());
    for resource in &resources {
        slots.push(Slot {
            acquire: list(resource, D3D12_RESOURCE_STATE_COMMON, SAMPLED)?,
            release: list(resource, SAMPLED, D3D12_RESOURCE_STATE_COMMON)?,
        });
    }
    // SAFETY: a new fence of this device.
    let own: ID3D12Fence = unsafe { raw.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
        .map_err(crate::common::api("CreateFence"))?;
    // SAFETY: an unnamed auto-reset event.
    let event = unsafe { CreateEventW(None, false, false, PCWSTR::null()) }
        .map_err(crate::common::api("CreateEventW"))?;
    // SAFETY: the new event's handle, owned (and closed) by `event` alone.
    let event = unsafe { OwnedHandle::from_raw_handle(event.0) };
    drop(shared);
    let textures = resources
        .into_iter()
        .map(|resource| {
            // SAFETY: the resource is a 2D texture of `desc`'s format, size,
            // mip and sample counts (checked above).
            let texture = unsafe {
                ::wgpu::hal::dx12::Device::texture_from_raw(
                    resource,
                    desc.format,
                    desc.dimension,
                    desc.size,
                    1,
                    1,
                )
            };
            // SAFETY: a texture of this device, made as `desc` says.
            unsafe { device.create_texture_from_hal::<Dx12>(texture, desc) }
        })
        .collect();
    Ok((
        textures,
        Shared {
            queue,
            ready,
            release,
            _allocator: allocator,
            slots,
            own,
            own_value: 0,
            event,
            leaked: false,
            _keep: device.clone(),
        },
    ))
}

struct Slot {
    acquire: ID3D12CommandList,
    release: ID3D12CommandList,
}

/// An import's fences and transitions on wgpu's queue.
pub(crate) struct Shared {
    queue: ID3D12CommandQueue,
    ready: ID3D12Fence,
    release: ID3D12Fence,
    /// Keeps the recorded lists' memory.
    _allocator: ID3D12CommandAllocator,
    slots: Vec<Slot>,
    /// Tells when the queue finished (`idle`).
    own: ID3D12Fence,
    own_value: u64,
    /// Waited on for a fence's value.
    event: OwnedHandle,
    /// The GPU may still use the objects: they are kept for the process.
    leaked: bool,
    /// Keeps the native device alive while the objects live.
    _keep: ::wgpu::Device,
}

impl Shared {
    pub(crate) fn ready_value(&self) -> u64 {
        // SAFETY: a live fence.
        unsafe { self.ready.GetCompletedValue() }
    }

    pub(crate) fn wait_ready(&self, value: u64, timeout: Duration) -> bool {
        self.wait(&self.ready, value, timeout)
    }

    fn wait(&self, fence: &ID3D12Fence, value: u64, timeout: Duration) -> bool {
        // SAFETY: a live fence.
        if unsafe { fence.GetCompletedValue() } >= value {
            return true;
        }
        let event = HANDLE(self.event.as_raw_handle());
        // SAFETY: a live fence and event; the event fires once the fence
        // reaches `value`.
        if unsafe { fence.SetEventOnCompletion(value, event) }.is_err() {
            return false;
        }
        // An earlier wait's event may fire late: wait until the value is
        // reached or the time is up.
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            let millis = u32::try_from(left.as_millis()).unwrap_or(u32::MAX - 1);
            // SAFETY: a live event handle; the wait is bounded.
            let woken = unsafe { WaitForSingleObject(event, millis) } == WAIT_OBJECT_0;
            // SAFETY: a live fence.
            if unsafe { fence.GetCompletedValue() } >= value {
                return true;
            }
            if !woken || left.is_zero() {
                return false;
            }
        }
    }

    fn execute(&self, list: &ID3D12CommandList) {
        // SAFETY: a closed list of this device, on wgpu's queue between
        // wgpu's own submissions.
        unsafe { self.queue.ExecuteCommandLists(&[Some(list.clone())]) };
    }

    pub(crate) fn acquire(&mut self, slot: usize, value: u64) -> Result<(), Error> {
        let list = self
            .slots
            .get(slot)
            .ok_or("no such texture")?
            .acquire
            .clone();
        // SAFETY: a GPU-side wait on a live shared fence.
        unsafe { self.queue.Wait(&self.ready, value) }.map_err(crate::common::api("Wait"))?;
        self.execute(&list);
        Ok(())
    }

    pub(crate) fn release(&mut self, slot: usize) -> Result<(), Error> {
        let list = self
            .slots
            .get(slot)
            .ok_or("no such texture")?
            .release
            .clone();
        self.execute(&list);
        Ok(())
    }

    pub(crate) fn signal(&mut self, value: u64) -> Result<(), Error> {
        // SAFETY: a GPU-side signal of a live shared fence after the queue's
        // earlier work.
        unsafe { self.queue.Signal(&self.release, value) }.map_err(crate::common::api("Signal"))
    }

    pub(crate) fn idle(&mut self, timeout: Duration) -> bool {
        self.own_value += 1;
        // SAFETY: a signal of the own fence after the queue's earlier work.
        if unsafe { self.queue.Signal(&self.own, self.own_value) }.is_err() {
            return false;
        }
        self.wait(&self.own, self.own_value, timeout)
    }

    pub(crate) fn leak(&mut self) {
        self.leaked = true;
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        if self.leaked {
            // The GPU may still execute the lists: keep their memory.
            std::mem::forget(self._allocator.clone());
            for fence in [&self.ready, &self.release, &self.own] {
                std::mem::forget(fence.clone());
            }
            for slot in &self.slots {
                std::mem::forget(slot.acquire.clone());
                std::mem::forget(slot.release.clone());
            }
        }
    }
}
