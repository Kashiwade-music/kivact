//! Role: The test engine on its own Direct3D 12 device, drawing into a shared pool.
//!
//! An independent device (see the crate documentation) on the adapter
//! whose LUID the window reported; the pool
//! (`kivact_gpu_share::d3d12::Pool`) makes the shared textures
//! and fences and their NT handles. A frame: the CPU's pixels into the
//! slot's upload buffer, the pool's transition to `COPY_DEST`, a copy into
//! the texture, the pool's transition back and submission (the queue
//! waits for `release` first and signals `ready` after).

#![allow(
    unsafe_code,
    reason = "a Direct3D 12 engine of its own, as an application has"
)]

use crate::engine::{Engine, POOL, WAIT};
use kivact_gpu_share::d3d12::{self as interop, Pool};
use kivact_gpu_share::{FrameReady, PoolConfig};
use kivact_types::{GpuAdapter, GpuBackend, SharedTextures};
use std::time::Duration;
use windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0;
use windows::Win32::Graphics::Direct3D12::{
    CLSID_D3D12DeviceFactory, D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC,
    D3D12_DEVICE_FACTORY_FLAG_DISALLOW_STORING_NEW_DEVICE_AS_SINGLETON, D3D12_HEAP_FLAG_NONE,
    D3D12_HEAP_PROPERTIES, D3D12_HEAP_TYPE_UPLOAD, D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    D3D12_RESOURCE_DESC, D3D12_RESOURCE_DIMENSION_BUFFER, D3D12_RESOURCE_STATE_COPY_DEST,
    D3D12_RESOURCE_STATE_GENERIC_READ, D3D12_SUBRESOURCE_FOOTPRINT, D3D12_TEXTURE_COPY_LOCATION,
    D3D12_TEXTURE_COPY_LOCATION_0, D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
    D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX, D3D12_TEXTURE_DATA_PITCH_ALIGNMENT,
    D3D12_TEXTURE_LAYOUT_ROW_MAJOR, D3D12CreateDevice, D3D12GetInterface, ID3D12CommandAllocator,
    ID3D12CommandList, ID3D12CommandQueue, ID3D12Device, ID3D12DeviceFactory,
    ID3D12GraphicsCommandList, ID3D12Resource,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory2, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_CREATE_FACTORY_FLAGS, IDXGIFactory4,
};
use windows::core::Interface;

fn failed(what: &'static str) -> impl Fn(windows::core::Error) -> String {
    move |e| format!("{what} failed: {e}")
}

/// What draws into one texture of the pool.
struct Slot {
    upload: ID3D12Resource,
    mapped: *mut u8,
    allocator: ID3D12CommandAllocator,
    list: ID3D12GraphicsCommandList,
}

pub(crate) struct D3d12 {
    device: ID3D12Device,
    queue: ID3D12CommandQueue,
    width: u32,
    height: u32,
    pitch: u32,
    slots: Vec<Slot>,
    pool: Pool,
}

fn buffer(size: u64) -> D3D12_RESOURCE_DESC {
    D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Width: size,
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
    }
}

/// An upload buffer of `size` bytes.
fn upload(device: &ID3D12Device, size: u64) -> Result<ID3D12Resource, String> {
    let properties = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_UPLOAD,
        ..Default::default()
    };
    let mut out: Option<ID3D12Resource> = None;
    // SAFETY: valid descriptions for the call.
    unsafe {
        device.CreateCommittedResource(
            &properties,
            D3D12_HEAP_FLAG_NONE,
            &buffer(size),
            D3D12_RESOURCE_STATE_GENERIC_READ,
            None,
            &mut out,
        )
    }
    .map_err(failed("CreateCommittedResource"))?;
    out.ok_or_else(|| "CreateCommittedResource gave nothing".into())
}

/// An independent device on the DXGI adapter that is `adapter`.
fn device(adapter: &GpuAdapter) -> Result<ID3D12Device, String> {
    let adapter = interop::find_adapter(adapter).map_err(|e| e.to_string())?;
    let mut device: Option<ID3D12Device> = None;
    let mut factory: Option<ID3D12DeviceFactory> = None;
    // SAFETY: asks for the device factory; fails on systems without it.
    if unsafe { D3D12GetInterface(&CLSID_D3D12DeviceFactory, &mut factory) }.is_ok()
        && let Some(factory) = factory
    {
        // SAFETY: a live factory.
        unsafe { factory.InitializeFromGlobalState() }
            .map_err(failed("InitializeFromGlobalState"))?;
        // SAFETY: as above.
        unsafe {
            factory.SetFlags(D3D12_DEVICE_FACTORY_FLAG_DISALLOW_STORING_NEW_DEVICE_AS_SINGLETON)
        }
        .map_err(failed("SetFlags"))?;
        // SAFETY: as above, with a live adapter.
        let _ = unsafe { factory.CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device) };
    }
    if device.is_none() {
        // SAFETY: a live adapter.
        unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device) }
            .map_err(failed("D3D12CreateDevice"))?;
    }
    device.ok_or_else(|| "no device".into())
}

/// This machine's hardware DXGI adapters, as a window reports them.
pub(crate) fn adapters() -> Vec<GpuAdapter> {
    // SAFETY: plain factory creation.
    let Ok(factory) =
        (unsafe { CreateDXGIFactory2::<IDXGIFactory4>(DXGI_CREATE_FACTORY_FLAGS(0)) })
    else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for i in 0.. {
        // SAFETY: enumerating until the factory says there are no more.
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(i) }) else {
            break;
        };
        // SAFETY: a live adapter.
        let Ok(desc) = (unsafe { adapter.GetDesc1() }) else {
            continue;
        };
        if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
            continue;
        }
        let mut luid = [0u8; 8];
        luid[..4].copy_from_slice(&desc.AdapterLuid.LowPart.to_le_bytes());
        luid[4..].copy_from_slice(&desc.AdapterLuid.HighPart.to_le_bytes());
        let name = String::from_utf16_lossy(&desc.Description);
        found.push(GpuAdapter {
            backend: GpuBackend::Dx12,
            name: name.trim_end_matches('\0').to_owned(),
            luid: Some(luid),
            device_uuid: None,
            driver_uuid: None,
            generation: 1,
        });
    }
    found
}

impl D3d12 {
    /// An engine of a pool of `width` × `height` textures, made for
    /// storage writes when `storage`.
    pub(crate) fn new(
        adapter: &GpuAdapter,
        (width, height): (u32, u32),
        storage: bool,
    ) -> Result<(Self, SharedTextures), String> {
        let device = device(adapter)?;
        let desc = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            ..Default::default()
        };
        // SAFETY: a valid description.
        let queue: ID3D12CommandQueue =
            unsafe { device.CreateCommandQueue(&desc) }.map_err(failed("CreateCommandQueue"))?;
        let pitch = (width * 4).div_ceil(D3D12_TEXTURE_DATA_PITCH_ALIGNMENT)
            * D3D12_TEXTURE_DATA_PITCH_ALIGNMENT;
        let mut slots = Vec::with_capacity(POOL);
        for _ in 0..POOL {
            let upload = upload(&device, u64::from(pitch) * u64::from(height))?;
            let mut mapped = std::ptr::null_mut();
            // SAFETY: an upload buffer, mapped once for its life.
            unsafe { upload.Map(0, None, Some(&mut mapped)) }.map_err(failed("Map"))?;
            // SAFETY: a valid list type.
            let allocator: ID3D12CommandAllocator =
                unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
                    .map_err(failed("CreateCommandAllocator"))?;
            // SAFETY: a new list of a live allocator.
            let list: ID3D12GraphicsCommandList = unsafe {
                device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)
            }
            .map_err(failed("CreateCommandList"))?;
            // SAFETY: the open new list.
            unsafe { list.Close() }.map_err(failed("Close"))?;
            slots.push(Slot {
                upload,
                mapped: mapped.cast(),
                allocator,
                list,
            });
        }
        let config = PoolConfig {
            slots: POOL as u32,
            storage,
            ..PoolConfig::new(width, height)
        };
        let (pool, shared) = Pool::new(&device, adapter, config).map_err(|e| e.to_string())?;
        Ok((
            Self {
                device,
                queue,
                width,
                height,
                pitch,
                slots,
                pool,
            },
            shared,
        ))
    }
}

impl Engine for D3d12 {
    fn pitch(&self) -> usize {
        self.pitch as usize
    }

    fn draw(&mut self, rgba: &[u8]) -> Result<FrameReady, String> {
        // The pool waited for the slot's last `ready`: its list finished.
        let target = self.pool.acquire(WAIT).map_err(|e| e.to_string())?;
        let slot = &self.slots[target.slot() as usize];
        let len = rgba.len().min(self.pitch as usize * self.height as usize);
        // SAFETY: the mapped upload buffer holds `pitch * height` bytes and
        // the GPU no longer reads it (the slot's `ready`).
        unsafe { std::ptr::copy_nonoverlapping(rgba.as_ptr(), slot.mapped, len) };
        // SAFETY: the allocator's lists finished (as above).
        unsafe { slot.allocator.Reset() }.map_err(failed("Reset"))?;
        // SAFETY: as above.
        unsafe { slot.list.Reset(&slot.allocator, None) }.map_err(failed("Reset"))?;
        let texture = D3D12_TEXTURE_COPY_LOCATION {
            // SAFETY: a copy of the pointer, never released (the location
            // drops nothing); the pool keeps the texture.
            pResource: unsafe { std::mem::transmute_copy(target.resource()) },
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                SubresourceIndex: 0,
            },
        };
        let source = D3D12_TEXTURE_COPY_LOCATION {
            // SAFETY: as above, of the slot's upload buffer.
            pResource: unsafe { std::mem::transmute_copy(&slot.upload) },
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                    Offset: 0,
                    Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                        Format: target.format(),
                        Width: self.width,
                        Height: self.height,
                        Depth: 1,
                        RowPitch: self.pitch,
                    },
                },
            },
        };
        let dest = D3D12_RESOURCE_STATE_COPY_DEST;
        // SAFETY: the reset list of the pool's device.
        unsafe { target.record_acquire(&slot.list, dest) };
        // SAFETY: recording into the reset list, with live resources.
        unsafe {
            slot.list
                .CopyTextureRegion(&texture, 0, 0, 0, &source, None)
        };
        // SAFETY: as above.
        unsafe { target.record_release(&slot.list, dest) };
        // SAFETY: the recorded list.
        unsafe { slot.list.Close() }.map_err(failed("Close"))?;
        let list: ID3D12CommandList = slot.list.cast().map_err(failed("cast"))?;
        // SAFETY: the engine's direct queue of the pool's device; the
        // closed list.
        unsafe { target.submit(&self.queue, &[Some(list)]) }.map_err(|e| e.to_string())
    }

    fn release_value(&self) -> u64 {
        self.pool.release_value().unwrap_or(0)
    }

    fn wait_release(&self, value: u64, timeout: Duration) -> bool {
        self.pool.wait_release(value, timeout).is_ok()
    }

    fn read_back(&self, slot: u32) -> Result<Vec<u8>, String> {
        // SAFETY: the engine's direct queue of the pool's device; the
        // caller's window does not use the texture.
        unsafe { self.pool.read_back(&self.queue, slot, WAIT) }.map_err(|e| e.to_string())
    }
}

impl Drop for D3d12 {
    fn drop(&mut self) {
        // The queue may still wait for a release that never comes: then
        // its objects are kept for the process (the pool keeps its own).
        if self.pool.wait_idle(WAIT).is_err() {
            std::mem::forget(self.device.clone());
            std::mem::forget(self.queue.clone());
            for slot in &self.slots {
                std::mem::forget(slot.upload.clone());
                std::mem::forget(slot.allocator.clone());
                std::mem::forget(slot.list.clone());
            }
        }
    }
}
