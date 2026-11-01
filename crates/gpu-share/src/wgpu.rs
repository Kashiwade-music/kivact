//! Role: A render engine's side of GPU texture sharing on Kivact's wgpu (29.0.4).
//!
//! For an engine written with wgpu of the version Kivact uses (feature
//! `wgpu`; the dependency is pinned to `=29.0.4`, arvane-render's, and the
//! two are raised together). [`request_device`] opens a device on the
//! adapter the window reported, able to share; [`Pool::new`] makes the
//! pool of the contract ([`ui_types::SharedTextures`]) with the backend's
//! pool (`d3d12::Pool`, `vulkan::Pool`) and wraps its textures as
//! `wgpu::Texture`s (`create_texture_from_hal`) of the usage [`usage`]
//! says. A frame: [`Pool::acquire`] gives a [`Target`], the engine
//! records its own command buffers on the target's texture, and
//! [`Pool::submit`] submits them between the hand-off:
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
//!   wgpu's use of it: [`Pool::submit`] and [`Pool::read_back`] are
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
mod read_back;
mod vulkan;

pub use read_back::read_back;

use crate::common::Error;
use crate::pool::{DROP_WAIT, FrameReady, PoolConfig};
use std::time::Duration;
use ui_types::{GpuAdapter, GpuBackend, SharedTextures};

/// The pool textures' format in wgpu (premultiplied alpha).
///
/// The values are sRGB-encoded: the window composites them as it does
/// `Pixels`. An engine that computes in linear light encodes to sRGB
/// last; the texture is not sampled through an sRGB view (`Rgba8Unorm`,
/// not `Rgba8UnormSrgb`).
pub const FORMAT: ::wgpu::TextureFormat = ::wgpu::TextureFormat::Rgba8Unorm;

/// The usage of a pool's `wgpu::Texture`s: render target, sampled, copy
/// source and destination, and storage when the pool is made for storage
/// writes ([`PoolConfig::storage`]).
pub fn usage(storage: bool) -> ::wgpu::TextureUsages {
    let usage = ::wgpu::TextureUsages::RENDER_ATTACHMENT
        | ::wgpu::TextureUsages::TEXTURE_BINDING
        | ::wgpu::TextureUsages::COPY_SRC
        | ::wgpu::TextureUsages::COPY_DST;
    if storage {
        usage | ::wgpu::TextureUsages::STORAGE_BINDING
    } else {
        usage
    }
}

/// The description of a pool's `wgpu::Texture`s.
fn texture_desc(config: &PoolConfig) -> ::wgpu::TextureDescriptor<'static> {
    ::wgpu::TextureDescriptor {
        label: Some("kivact shared texture"),
        size: ::wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: ::wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: usage(config.storage),
        view_formats: &[],
    }
}

/// A device and queue on the adapter of `instance` that is `adapter` (its
/// LUID, or its Vulkan device and driver UUIDs), with the window's
/// graphics API and `desc`'s features and limits, able to share textures:
/// a Vulkan device also enables [`crate::vulkan::DEVICE_EXTENSIONS`]
/// (wgpu does not enable the external semaphore ones) and needs Vulkan
/// [`crate::vulkan::API_VERSION`]; a Direct3D 12 device is wgpu's own
/// `request_device` (shared heaps and fences need nothing more).
///
/// In a process where Kivact's window also runs wgpu on Direct3D 12,
/// `D3D12CreateDevice` returns the process's one device of the adapter:
/// both wgpu devices wrap the same `ID3D12Device`, each with its own
/// queue. Sharing works on it (the window opens the handles on the device
/// that made them); a lost device is lost for both. Make `instance`
/// without `wgpu::InstanceFlags::DEBUG` and `VALIDATION` there (as Arvane
/// does: `InstanceFlags::empty().with_env()`): enabling Direct3D 12's
/// debug layer once a device exists removes that device, the window's
/// too. `InstanceDescriptor`'s defaults turn them on in a debug build.
pub fn request_device(
    instance: &::wgpu::Instance,
    adapter: &GpuAdapter,
    desc: &::wgpu::DeviceDescriptor<'_>,
) -> Result<(::wgpu::Adapter, ::wgpu::Device, ::wgpu::Queue), Error> {
    let backends = match adapter.backend {
        GpuBackend::Dx12 => ::wgpu::Backends::DX12,
        GpuBackend::Vulkan => ::wgpu::Backends::VULKAN,
    };
    for found in pollster::block_on(instance.enumerate_adapters(backends)) {
        match adapter.backend {
            #[cfg(windows)]
            GpuBackend::Dx12 if d3d12::is_adapter(&found, adapter)? => {
                return d3d12::open(found, desc);
            }
            GpuBackend::Vulkan if vulkan::is_adapter(&found, adapter)? => {
                return vulkan::open(found, desc);
            }
            _ => {}
        }
    }
    Err(Error::NoAdapter)
}

/// The backend half of a pool.
enum Raw {
    #[cfg(windows)]
    D3d12(d3d12::Raw),
    Vulkan(Box<vulkan::Raw>),
}

/// The backend half of a target.
#[derive(Debug)]
enum Inner {
    #[cfg(windows)]
    D3d12(crate::d3d12::Target),
    Vulkan(crate::vulkan::Target),
}

/// The texture the next frame is drawn into, from [`Pool::acquire`], until
/// [`Pool::submit`] or [`Pool::cancel`] takes it.
///
/// The texture is the pool's: the engine uses it only in the command
/// buffers it hands to [`Pool::submit`] with this target (a clone used
/// after that, or in another submission, finds the texture with the
/// window: undefined pictures).
#[derive(Debug)]
pub struct Target {
    texture: ::wgpu::Texture,
    slot: u32,
    inner: Inner,
}

impl Target {
    /// The texture's index in the pool.
    pub fn slot(&self) -> u32 {
        self.slot
    }

    /// The texture to draw into, of [`FORMAT`] and the pool's [`usage`].
    pub fn texture(&self) -> &::wgpu::Texture {
        &self.texture
    }

    /// A view of the texture (`desc` as for any texture).
    pub fn create_view(&self, desc: &::wgpu::TextureViewDescriptor<'_>) -> ::wgpu::TextureView {
        self.texture.create_view(desc)
    }

    /// The texture's size: width and height.
    pub fn size(&self) -> (u32, u32) {
        (self.texture.width(), self.texture.height())
    }
}

/// A pool of shared textures as `wgpu::Texture`s, on an engine's wgpu
/// device (see the module documentation for a frame).
pub struct Pool {
    raw: Raw,
    textures: Vec<::wgpu::Texture>,
    /// Whether wgpu drew each texture yet (its tracker left
    /// `UNINITIALIZED`).
    drawn: Vec<bool>,
    /// A raw submission failed after wgpu's: the hand-off is broken.
    failed: bool,
    /// Keeps the native device alive while the raw objects live.
    device: ::wgpu::Device,
}

impl Pool {
    /// A pool of `config`'s textures on `device` (a device of
    /// [`request_device`] on `adapter`), and its handles for the window to
    /// import, made once. An error when the device is not on `adapter`,
    /// lacks what sharing needs (Vulkan's extensions and API version), or
    /// cannot make such textures (Vulkan storage images may be missing).
    pub fn new(
        device: &::wgpu::Device,
        adapter: &GpuAdapter,
        config: PoolConfig,
    ) -> Result<(Self, SharedTextures), Error> {
        config.check()?;
        let desc = texture_desc(&config);
        let (raw, textures, shared) = match adapter.backend {
            #[cfg(windows)]
            GpuBackend::Dx12 => {
                let (raw, textures, shared) = d3d12::Raw::new(device, adapter, config, &desc)?;
                (Raw::D3d12(raw), textures, shared)
            }
            #[cfg(not(windows))]
            GpuBackend::Dx12 => {
                return Err(Error::Mismatch("Direct3D 12 is Windows only".into()));
            }
            GpuBackend::Vulkan => {
                let (raw, textures, shared) = vulkan::Raw::new(device, adapter, config, &desc)?;
                (Raw::Vulkan(Box::new(raw)), textures, shared)
            }
        };
        let pool = Self {
            raw,
            drawn: vec![false; textures.len()],
            textures,
            failed: false,
            device: device.clone(),
        };
        Ok((pool, shared))
    }

    /// The texture of the next frame, once its last frame's submission
    /// signaled `ready` (waited at most `timeout` on the CPU); the GPU
    /// waits for the window's release in [`Pool::submit`].
    ///
    /// The texture holds the last frame drawn into it (the frame of
    /// `slots` frames ago), so the engine may load it
    /// (`LoadOp::Load`); a texture's first frame holds nothing defined.
    /// A [`Error::Timeout`] means the texture's last submission did not
    /// finish, usually because it still waits for a release the window
    /// never gives.
    pub fn acquire(&mut self, timeout: Duration) -> Result<Target, Error> {
        if self.failed {
            return Err(Error::Mismatch(
                "a submission of the pool failed: make a new pool".into(),
            ));
        }
        let inner = match &mut self.raw {
            #[cfg(windows)]
            Raw::D3d12(raw) => Inner::D3d12(raw.pool_mut().acquire(timeout)?),
            Raw::Vulkan(raw) => Inner::Vulkan(raw.pool_mut().acquire(timeout)?),
        };
        let slot = match &inner {
            #[cfg(windows)]
            Inner::D3d12(target) => target.slot(),
            Inner::Vulkan(target) => target.slot(),
        };
        Ok(Target {
            texture: self.textures[slot as usize].clone(),
            slot,
            inner,
        })
    }

    /// Give `target` up without a frame (nothing of it was submitted).
    pub fn cancel(&mut self, target: Target) {
        match (&mut self.raw, target.inner) {
            #[cfg(windows)]
            (Raw::D3d12(raw), Inner::D3d12(target)) => raw.pool_mut().cancel(target),
            (Raw::Vulkan(raw), Inner::Vulkan(target)) => raw.pool_mut().cancel(target),
            #[cfg(windows)]
            _ => unreachable!("invariant: a pool's targets are of its backend"),
        }
    }

    /// Submit `command_buffers` (the engine's drawing into `target`'s
    /// texture, recorded on the pool's device) on `queue` between the
    /// hand-off (see the module documentation): the frame to publish.
    /// An error before wgpu's submission gives the frame up; one after it
    /// (a lost device) leaves the pool unusable (later acquires fail).
    ///
    /// Writes with `Queue::write_texture` to the target, made after
    /// [`Pool::acquire`], run in this submission (wgpu flushes them
    /// first); no other `Queue::submit` may run between them.
    ///
    /// # Safety
    /// `queue` is the queue of the pool's device. No other thread uses
    /// that queue during the call (`Queue::submit`, `Surface::present`,
    /// `Device::poll` and the like): the raw submissions go to wgpu's
    /// `VkQueue` or `ID3D12CommandQueue`, which Vulkan requires to be
    /// externally synchronized and whose order the hand-off needs.
    pub unsafe fn submit(
        &mut self,
        queue: &::wgpu::Queue,
        target: Target,
        command_buffers: impl IntoIterator<Item = ::wgpu::CommandBuffer>,
    ) -> Result<FrameReady, Error> {
        let slot = target.slot as usize;
        let first = !self.drawn[slot];
        // 1. The window's release, then the texture taken from it.
        // SAFETY: wgpu's queue, not used meanwhile (the caller).
        let taken = unsafe { self.raw.take(&target.inner, first) };
        if let Err(error) = taken {
            self.cancel(target);
            return Err(error);
        }
        // 2. and 3. The engine's commands, then wgpu's tracker back to
        // the state of the hand-off.
        let mut encoder = self
            .device
            .create_command_encoder(&::wgpu::CommandEncoderDescriptor {
                label: Some("kivact shared texture hand-off"),
            });
        encoder.transition_resources(
            std::iter::empty(),
            std::iter::once(::wgpu::TextureTransition {
                texture: &target.texture,
                selector: None,
                state: ::wgpu::TextureUses::RESOURCE,
            }),
        );
        queue.submit(
            command_buffers
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        self.drawn[slot] = true;
        // 4. The texture given back, `ready` signaled.
        // SAFETY: as above.
        let given = unsafe { self.raw.give(target.inner) };
        if given.is_err() {
            self.failed = true;
        }
        given
    }

    /// The `release` value now: the window reads no frame of this value
    /// or less again.
    pub fn release_value(&self) -> Result<u64, Error> {
        match &self.raw {
            #[cfg(windows)]
            Raw::D3d12(raw) => raw.pool().release_value(),
            Raw::Vulkan(raw) => raw.pool().release_value(),
        }
    }

    /// Wait at most `timeout` on the CPU for `release` to reach `value`.
    pub fn wait_release(&self, value: u64, timeout: Duration) -> Result<(), Error> {
        match &self.raw {
            #[cfg(windows)]
            Raw::D3d12(raw) => raw.pool().wait_release(value, timeout),
            Raw::Vulkan(raw) => raw.pool().wait_release(value, timeout),
        }
    }

    /// Wait at most `timeout` on the CPU for every submitted frame's
    /// `ready` (the pool's work finished).
    pub fn wait_idle(&self, timeout: Duration) -> Result<(), Error> {
        match &self.raw {
            #[cfg(windows)]
            Raw::D3d12(raw) => raw.pool().wait_idle(timeout),
            Raw::Vulkan(raw) => raw.pool().wait_idle(timeout),
        }
    }

    /// Texture `slot`'s last frame as straight-alpha RGBA8 (`width` ×
    /// `height` × 4 bytes, rows top to bottom), once its `ready` arrived
    /// (each wait at most `timeout`), copied by a raw submission on wgpu's
    /// queue (the backend pool's `read_back`).
    ///
    /// # Safety
    /// - No other thread uses the pool's device's queue during the call
    ///   (as [`Pool::submit`]).
    /// - No window uses the texture meanwhile: its frame is released, or
    ///   the pool was never imported (then, on Vulkan, the texture was
    ///   never in the hand-off state; see the crate documentation).
    pub unsafe fn read_back(&self, slot: u32, timeout: Duration) -> Result<Vec<u8>, Error> {
        match &self.raw {
            // SAFETY: wgpu's queue, not used meanwhile; the texture's use
            // as the caller says.
            #[cfg(windows)]
            Raw::D3d12(raw) => unsafe { raw.read_back(slot, timeout) },
            // SAFETY: as above.
            Raw::Vulkan(raw) => unsafe { raw.read_back(slot, timeout) },
        }
    }
}

impl Raw {
    /// Step 1 of a frame of `target`: the GPU-side wait for `release`,
    /// then the texture taken (`first`: wgpu never drew it).
    ///
    /// # Safety
    /// No other thread uses the device's queue during the call.
    unsafe fn take(&mut self, target: &Inner, first: bool) -> Result<(), Error> {
        match (self, target) {
            // SAFETY: as the caller says.
            #[cfg(windows)]
            (Self::D3d12(raw), Inner::D3d12(target)) => unsafe { raw.take(target, first) },
            (Self::Vulkan(raw), Inner::Vulkan(target)) => {
                // Vulkan takes a texture the same way in its first frame
                // (wgpu's first barrier then starts from `UNDEFINED`).
                let _ = first;
                // SAFETY: as above.
                unsafe { raw.take(target) }
            }
            #[cfg(windows)]
            _ => unreachable!("invariant: a pool's targets are of its backend"),
        }
    }

    /// Step 4 of a frame of `target`: the texture given back and `ready`
    /// signaled.
    ///
    /// # Safety
    /// As [`Raw::take`].
    unsafe fn give(&mut self, target: Inner) -> Result<FrameReady, Error> {
        match (self, target) {
            // SAFETY: as the caller says.
            #[cfg(windows)]
            (Self::D3d12(raw), Inner::D3d12(target)) => unsafe { raw.give(target) },
            // SAFETY: as above.
            (Self::Vulkan(raw), Inner::Vulkan(target)) => unsafe { raw.give(target) },
            #[cfg(windows)]
            _ => unreachable!("invariant: a pool's targets are of its backend"),
        }
    }

    /// Keep the raw objects for the process (the GPU may still use them).
    fn leak(&mut self) {
        match self {
            #[cfg(windows)]
            Self::D3d12(raw) => raw.leak(),
            Self::Vulkan(raw) => raw.leak(),
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        if self.wait_idle(DROP_WAIT).is_err() {
            // A submission may still wait for a release: the textures and
            // raw objects stay for the process.
            for texture in &self.textures {
                std::mem::forget(texture.clone());
            }
            self.raw.leak();
        }
    }
}
