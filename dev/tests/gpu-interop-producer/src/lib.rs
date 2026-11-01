//! Role: A render engine of its own device that shares frames with Arvane, for tests.
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

#[cfg(windows)]
mod d3d12;
mod engine;
pub mod pattern;
#[cfg(any(windows, target_os = "linux"))]
mod vulkan;
#[cfg(all(feature = "wgpu", any(windows, target_os = "linux")))]
mod wgpu_engine;

use engine::Engine;
pub use engine::{POOL, WAIT};
use kivact_types::{ExternalFrame, ExternalImageId, GpuAdapter, GpuBackend, SharedTextures};
use std::time::Duration;

/// This machine's hardware adapters of `backend` that a window could
/// report (generation 1), for tests without a window; none where the
/// backend is missing.
pub fn adapters(backend: GpuBackend) -> Vec<GpuAdapter> {
    match backend {
        #[cfg(windows)]
        GpuBackend::Dx12 => d3d12::adapters(),
        #[cfg(any(windows, target_os = "linux"))]
        GpuBackend::Vulkan => vulkan::adapters(),
        #[allow(unreachable_patterns, reason = "the backends of other platforms")]
        _ => Vec::new(),
    }
}

/// Which engine a [`Producer`] runs.
#[derive(Clone, Copy)]
enum Kind {
    /// On the graphics API itself, its pool made for storage writes or
    /// not.
    Native { storage: bool },
    /// On Kivact's wgpu.
    #[cfg(all(feature = "wgpu", any(windows, target_os = "linux")))]
    Wgpu,
}

/// An engine sharing its frames as one image of a window.
pub struct Producer {
    image: ExternalImageId,
    width: u32,
    height: u32,
    engine: Box<dyn Engine>,
    /// The pool's handles, until they are shared.
    shared: Option<SharedTextures>,
    /// The frame drawn next.
    next: u64,
}

impl Producer {
    /// An engine on `adapter` (with its graphics API) drawing
    /// `width` × `height` frames ([`pattern::fits`]), named `image`.
    pub fn new(
        adapter: &GpuAdapter,
        image: ExternalImageId,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        Self::start(
            adapter,
            image,
            (width, height),
            Kind::Native { storage: false },
        )
    }

    /// As [`Producer::new`], its pool made for storage writes as well
    /// ([`SharedTextures::storage`]); an error when the adapter cannot
    /// export such textures.
    pub fn with_storage(
        adapter: &GpuAdapter,
        image: ExternalImageId,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        Self::start(
            adapter,
            image,
            (width, height),
            Kind::Native { storage: true },
        )
    }

    /// As [`Producer::new`], the engine written with Kivact's wgpu
    /// (`kivact_gpu_share::wgpu`) on a device of its own.
    #[cfg(all(feature = "wgpu", any(windows, target_os = "linux")))]
    pub fn with_wgpu(
        adapter: &GpuAdapter,
        image: ExternalImageId,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        Self::start(adapter, image, (width, height), Kind::Wgpu)
    }

    fn start(
        adapter: &GpuAdapter,
        image: ExternalImageId,
        (width, height): (u32, u32),
        kind: Kind,
    ) -> Result<Self, String> {
        if !pattern::fits(width, height) {
            return Err(format!("the pattern does not fit {width} x {height}"));
        }
        let storage = match kind {
            Kind::Native { storage } => storage,
            #[cfg(all(feature = "wgpu", any(windows, target_os = "linux")))]
            Kind::Wgpu => {
                let (engine, shared) = wgpu_engine::Wgpu::new(adapter, (width, height))?;
                return Ok(Self {
                    image,
                    width,
                    height,
                    engine: Box::new(engine),
                    shared: Some(shared),
                    next: 0,
                });
            }
        };
        let (engine, shared): (Box<dyn Engine>, _) = match adapter.backend {
            #[cfg(windows)]
            GpuBackend::Dx12 => {
                let (engine, shared) = d3d12::D3d12::new(adapter, (width, height), storage)?;
                (Box::new(engine), shared)
            }
            #[cfg(not(windows))]
            GpuBackend::Dx12 => return Err("Direct3D 12 is Windows only".into()),
            #[cfg(any(windows, target_os = "linux"))]
            GpuBackend::Vulkan => {
                let (engine, shared) = vulkan::Vulkan::new(adapter, (width, height), storage)?;
                (Box::new(engine), shared)
            }
            #[cfg(not(any(windows, target_os = "linux")))]
            GpuBackend::Vulkan => return Err("no Vulkan engine on this platform".into()),
        };
        Ok(Self {
            image,
            width,
            height,
            engine,
            shared: Some(shared),
            next: 0,
        })
    }

    /// The pool's and the fences' handles, to import: once per engine (an
    /// NVIDIA Vulkan driver refuses to import memory whose handle was
    /// exported before).
    pub fn shared_textures(&mut self) -> Result<SharedTextures, String> {
        self.shared
            .take()
            .ok_or_else(|| "the pool was shared already".into())
    }

    /// Draw `frame` (counted from 0, in order) into texture `frame % POOL`
    /// and signal `ready` to `frame + 1`: the frame to name in a scene.
    pub fn draw(&mut self, frame: u64) -> Result<ExternalFrame, String> {
        if frame != self.next {
            return Err(format!("frame {frame} drawn after {}", self.next));
        }
        let rgba = pattern::rgba(self.width, self.height, frame, self.engine.pitch());
        let ready = self.engine.draw(&rgba)?;
        self.next += 1;
        debug_assert_eq!(
            (u64::from(ready.slot), ready.value),
            (frame % POOL as u64, frame + 1),
            "the pool counts frames as the producer does"
        );
        Ok(ready.frame(self.image))
    }

    /// Texture `slot`'s last frame, read back as straight-alpha RGBA8
    /// (`width` × `height` × 4 bytes), while no window uses it.
    pub fn read_back(&self, slot: u32) -> Result<Vec<u8>, String> {
        self.engine.read_back(slot)
    }

    /// The release fence's value now.
    pub fn release_value(&self) -> u64 {
        self.engine.release_value()
    }

    /// Wait at most `timeout` for the release fence to reach `value`;
    /// whether it did.
    pub fn wait_release(&self, value: u64, timeout: Duration) -> bool {
        self.engine.wait_release(value, timeout)
    }
}
