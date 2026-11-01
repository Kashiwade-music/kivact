//! Role: The engine's Direct3D 12 export pool: shared textures, fences, hand-off.

use super::adapter::luid;
use super::raw::FORMAT;
use super::raw::{transition, wait};
use super::read_back::read_back;
use crate::common::{Error, api};
use crate::d3d12_contract::HAND_OFF_STATE;
use crate::d3d12_contract::luid_bytes;
use crate::d3d12_contract::resource_flags as flags;
use crate::pool::{DROP_WAIT, FrameReady, PoolConfig, Slots};
use kivact_types::{
    AdapterMatch, GpuAdapter, GpuBackend, SharedHandle, SharedTextureFormat, SharedTextures,
};
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::time::Duration;
use windows::Win32::Foundation::GENERIC_ALL;
use windows::Win32::Graphics::Direct3D12::{
    D3D12_FENCE_FLAG_SHARED, D3D12_HEAP_FLAG_SHARED, D3D12_HEAP_PROPERTIES,
    D3D12_HEAP_TYPE_DEFAULT, D3D12_RESOURCE_DESC, D3D12_RESOURCE_DIMENSION_TEXTURE2D,
    D3D12_RESOURCE_STATES, D3D12_TEXTURE_LAYOUT_UNKNOWN, ID3D12CommandList, ID3D12CommandQueue,
    ID3D12Device, ID3D12DeviceChild, ID3D12Fence, ID3D12GraphicsCommandList, ID3D12Resource,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT, DXGI_SAMPLE_DESC};
use windows::core::{Interface, PCWSTR};

/// The texture the next frame is drawn into, from [`Pool::acquire`], until
/// [`Pool::submit`] or [`Pool::finish`] takes it.
#[derive(Debug)]
pub struct Target {
    slot: u32,
    resource: ID3D12Resource,
    width: u32,
    height: u32,
    /// The value `release` reaches before the queue writes (0: no wait).
    wait: u64,
}

impl Target {
    /// The texture's index in the pool.
    pub fn slot(&self) -> u32 {
        self.slot
    }

    /// The texture to draw into.
    pub fn resource(&self) -> &ID3D12Resource {
        &self.resource
    }

    /// The texture's size: width and height.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// The texture's format ([`FORMAT`]).
    pub fn format(&self) -> DXGI_FORMAT {
        FORMAT
    }

    /// The value `release` reaches before the queue writes (0: no wait).
    #[cfg(feature = "wgpu")]
    pub(crate) fn release_wait(&self) -> u64 {
        self.wait
    }
}

/// The fences an engine's own submission of a frame waits for and
/// signals ([`Pool::finish`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Submission {
    /// `ID3D12CommandQueue::Wait` for `release` to reach this value before
    /// the frame's command lists (`None`: the texture was never drawn).
    pub wait: Option<(ID3D12Fence, u64)>,
    /// `ID3D12CommandQueue::Signal` of `ready` to this value after them.
    pub signal: (ID3D12Fence, u64),
    /// The frame to publish to the UI.
    pub frame: FrameReady,
}

/// A pool of shared textures and its `ready` and `release` fences, on the
/// engine's own device.
pub struct Pool {
    device: ID3D12Device,
    textures: Vec<ID3D12Resource>,
    width: u32,
    height: u32,
    storage: bool,
    ready: ID3D12Fence,
    release: ID3D12Fence,
    frames: Slots,
}

impl Pool {
    /// A pool of `config`'s textures on `device` (on `adapter`), and its
    /// handles for the window to import ([`kivact_types::SharedTextures`],
    /// made here and only here).
    pub fn new(
        device: &ID3D12Device,
        adapter: &GpuAdapter,
        config: PoolConfig,
    ) -> Result<(Self, SharedTextures), Error> {
        config.check()?;
        let wanted = luid(adapter)?;
        // SAFETY: a live device.
        let luid = luid_bytes(unsafe { device.GetAdapterLuid() });
        if luid != wanted {
            return Err(Error::Mismatch(
                "the device is not on the window's adapter".into(),
            ));
        }
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Width: u64::from(config.width),
            Height: config.height,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: FORMAT,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: flags(config.storage),
            ..Default::default()
        };
        let properties = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            ..Default::default()
        };
        let mut textures = Vec::with_capacity(config.slots as usize);
        for _ in 0..config.slots {
            let mut texture: Option<ID3D12Resource> = None;
            // SAFETY: valid descriptions for the call.
            unsafe {
                device.CreateCommittedResource(
                    &properties,
                    D3D12_HEAP_FLAG_SHARED,
                    &desc,
                    HAND_OFF_STATE,
                    None,
                    &mut texture,
                )
            }
            .map_err(api("CreateCommittedResource"))?;
            textures.push(texture.ok_or_else(|| Error::Api {
                call: "CreateCommittedResource",
                message: "no resource".into(),
            })?);
        }
        let fence = || -> Result<ID3D12Fence, Error> {
            // SAFETY: a new fence of the device.
            unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_SHARED) }.map_err(api("CreateFence"))
        };
        let pool = Self {
            device: device.clone(),
            textures,
            width: config.width,
            height: config.height,
            storage: config.storage,
            ready: fence()?,
            release: fence()?,
            frames: Slots::new(config.slots),
        };
        let shared = pool.share(luid)?;
        Ok((pool, shared))
    }

    /// The NT handle of a shareable object of the device.
    fn handle<T: Interface>(&self, object: &T) -> Result<SharedHandle, Error> {
        let child: ID3D12DeviceChild = object.cast().map_err(api("QueryInterface"))?;
        // SAFETY: the object was made shareable (a shared heap or fence).
        let raw = unsafe {
            self.device
                .CreateSharedHandle(&child, None, GENERIC_ALL.0, PCWSTR::null())
        }
        .map_err(api("CreateSharedHandle"))?;
        // SAFETY: a new NT handle, owned by the result alone.
        Ok(unsafe { OwnedHandle::from_raw_handle(raw.0) }.into())
    }

    /// The handles of the textures and fences, made once.
    fn share(&self, luid: [u8; 8]) -> Result<SharedTextures, Error> {
        let textures = self
            .textures
            .iter()
            .map(|texture| self.handle(texture))
            .collect::<Result<_, _>>()?;
        Ok(SharedTextures {
            width: self.width,
            height: self.height,
            format: SharedTextureFormat::Rgba8UnormPremultiplied,
            backend: GpuBackend::Dx12,
            adapter: AdapterMatch::Luid(luid),
            storage: self.storage,
            textures,
            ready: self.handle(&self.ready)?,
            release: self.handle(&self.release)?,
        })
    }

    /// The texture of the next frame, once its last frame signaled
    /// `ready` (waited at most `timeout` on the CPU). The queue waits for
    /// the window's `release` of that frame: the submission
    /// ([`Pool::submit`], or [`Pool::finish`]'s `wait`) does, so the
    /// engine can draw ahead of the window.
    ///
    /// A [`Error::Timeout`] means the texture's last submission did not
    /// finish, usually because the queue still waits for a release the
    /// window never gives (a lost device, a window that stopped
    /// presenting).
    pub fn acquire(&mut self, timeout: Duration) -> Result<Target, Error> {
        let (slot, last) = self.frames.acquire()?;
        if let Err(error) = wait(&self.ready, last, timeout, "the texture's last frame") {
            self.frames.cancel();
            return Err(error);
        }
        Ok(Target {
            slot,
            resource: self.textures[slot as usize].clone(),
            width: self.width,
            height: self.height,
            wait: last,
        })
    }

    /// Give `target` up without a frame (nothing of it was submitted).
    pub fn cancel(&mut self, target: Target) {
        let _ = target;
        self.frames.cancel();
    }

    /// Record into `list` the transition of `target`'s texture from the
    /// hand-off state ([`HAND_OFF_STATE`]) to `state`, the engine's first
    /// use. Nothing when `state` is the hand-off state.
    ///
    /// # Safety
    /// `list` is open (reset and not closed), of the pool's device;
    /// `target` is of this pool.
    pub unsafe fn record_acquire(
        &self,
        list: &ID3D12GraphicsCommandList,
        target: &Target,
        state: D3D12_RESOURCE_STATES,
    ) {
        if state != HAND_OFF_STATE {
            let barrier = transition(&target.resource, HAND_OFF_STATE, state);
            // SAFETY: recording into the open list (the caller).
            unsafe { list.ResourceBarrier(&[barrier]) };
        }
    }

    /// Record into `list` the transition of `target`'s texture from
    /// `state`, the engine's last use, back to the hand-off state. Nothing
    /// when `state` is the hand-off state.
    ///
    /// # Safety
    /// As [`Pool::record_acquire`].
    pub unsafe fn record_release(
        &self,
        list: &ID3D12GraphicsCommandList,
        target: &Target,
        state: D3D12_RESOURCE_STATES,
    ) {
        if state != HAND_OFF_STATE {
            let barrier = transition(&target.resource, state, HAND_OFF_STATE);
            // SAFETY: recording into the open list (the caller).
            unsafe { list.ResourceBarrier(&[barrier]) };
        }
    }

    /// The fences the engine's own submission of `target`'s frame waits
    /// for and signals; the frame counts as submitted. The submission
    /// must follow: until `ready` reaches the frame's value, acquiring
    /// this texture again times out.
    pub fn finish(&mut self, target: Target) -> Submission {
        let frame = self.frames.finish(target.slot);
        Submission {
            wait: (target.wait > 0).then(|| (self.release.clone(), target.wait)),
            signal: (self.ready.clone(), frame.value),
            frame,
        }
    }

    /// On `queue`: wait for the window's release of `target`'s texture,
    /// execute `lists` (which draw the frame between
    /// [`Pool::record_acquire`] and [`Pool::record_release`]), signal
    /// `ready`: the frame to publish. When the wait or signal cannot be
    /// queued, the frame is given up.
    ///
    /// # Safety
    /// `queue` is a direct queue of the pool's device; `lists` are closed
    /// command lists of the device.
    pub unsafe fn submit(
        &mut self,
        queue: &ID3D12CommandQueue,
        target: Target,
        lists: &[Option<ID3D12CommandList>],
    ) -> Result<FrameReady, Error> {
        if target.wait > 0 {
            // SAFETY: a GPU-side wait on a live fence of the device.
            if let Err(error) = unsafe { queue.Wait(&self.release, target.wait) } {
                self.cancel(target);
                return Err(api("ID3D12CommandQueue::Wait")(error));
            }
        }
        // SAFETY: closed lists of the device, on its queue (the caller).
        unsafe { queue.ExecuteCommandLists(lists) };
        let value = self.frames.last() + 1;
        // SAFETY: a GPU-side signal of a live fence of the device.
        if let Err(error) = unsafe { queue.Signal(&self.ready, value) } {
            self.cancel(target);
            return Err(api("ID3D12CommandQueue::Signal")(error));
        }
        Ok(self.finish(target).frame)
    }

    /// The pool's textures, by slot.
    #[cfg(feature = "wgpu")]
    pub(crate) fn textures(&self) -> &[ID3D12Resource] {
        &self.textures
    }

    /// The `release` fence.
    #[cfg(feature = "wgpu")]
    pub(crate) fn release_fence(&self) -> &ID3D12Fence {
        &self.release
    }

    /// The `release` fence's value now: the window reads no frame of this
    /// value or less again.
    pub fn release_value(&self) -> Result<u64, Error> {
        // SAFETY: a live fence.
        Ok(unsafe { self.release.GetCompletedValue() })
    }

    /// Wait at most `timeout` on the CPU for `release` to reach `value`.
    pub fn wait_release(&self, value: u64, timeout: Duration) -> Result<(), Error> {
        wait(&self.release, value, timeout, "the window's release")
    }

    /// Wait at most `timeout` on the CPU for every submitted frame's
    /// `ready` (the engine's work on the pool finished).
    pub fn wait_idle(&self, timeout: Duration) -> Result<(), Error> {
        wait(&self.ready, self.frames.last(), timeout, "the last frame")
    }

    /// Texture `slot`'s last frame as straight-alpha RGBA8 (`width` ×
    /// `height` × 4 bytes, rows top to bottom), once its `ready` arrived
    /// (each wait at most `timeout`), copied on `queue`. A thin wrapper of
    /// [`read_back`].
    ///
    /// # Safety
    /// `queue` is a direct queue of the pool's device; no window uses the
    /// texture meanwhile (its frame is released, or the pool was never
    /// imported).
    pub unsafe fn read_back(
        &self,
        queue: &ID3D12CommandQueue,
        slot: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, Error> {
        let last = self.frames.drawn(slot)?;
        wait(&self.ready, last, timeout, "the texture's last frame")?;
        // SAFETY: the pool's device and texture; the queue and the
        // texture's use as the caller says.
        unsafe {
            read_back(
                &self.device,
                queue,
                &self.textures[slot as usize],
                HAND_OFF_STATE,
                timeout,
            )
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        if self.wait_idle(DROP_WAIT).is_err() {
            // A queue may still wait for a release that never comes: its
            // objects are kept for the process.
            for texture in &self.textures {
                std::mem::forget(texture.clone());
            }
            std::mem::forget(self.ready.clone());
            std::mem::forget(self.release.clone());
        }
    }
}
