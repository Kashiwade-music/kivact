//! Role: Producer: runs a backend engine for an adapter and draws pattern frames.

use crate::engine::Engine;
use crate::engine::POOL;
use kivact_types::{
    ExternalFrame, ExternalImageId, GpuAdapter, GpuBackend, SHARED_PREPARED, SharedTextures,
};
use std::time::Duration;

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
    // xtask-allow(state-fields): owned resource - the pool's handles until they are shared; taken once
    /// The pool's handles, until they are shared.
    shared: Option<SharedTextures>,
    // xtask-allow(state-fields): S-8 ID - the number of the frame drawn next
    /// The frame drawn next.
    next: u64,
}

impl Producer {
    /// An engine on `adapter` (with its graphics API) drawing
    /// `width` × `height` frames ([`crate::pattern::fits`]), named `image`.
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
        if !crate::pattern::fits(width, height) {
            return Err(format!("the pattern does not fit {width} x {height}"));
        }
        let storage = match kind {
            Kind::Native { storage } => storage,
            #[cfg(all(feature = "wgpu", any(windows, target_os = "linux")))]
            Kind::Wgpu => {
                let (engine, shared) = crate::wgpu_engine::Wgpu::new(adapter, (width, height))?;
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
                let (engine, shared) = crate::d3d12::D3d12::new(adapter, (width, height), storage)?;
                (Box::new(engine), shared)
            }
            #[cfg(not(windows))]
            GpuBackend::Dx12 => return Err("Direct3D 12 is Windows only".into()),
            #[cfg(any(windows, target_os = "linux"))]
            GpuBackend::Vulkan => {
                let (engine, shared) =
                    crate::vulkan::Vulkan::new(adapter, (width, height), storage)?;
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
    /// and signal `ready` to `frame + SHARED_PREPARED + 1` (frame values
    /// start above the preparation signal): the frame to name in a scene.
    pub fn draw(&mut self, frame: u64) -> Result<ExternalFrame, String> {
        if frame != self.next {
            return Err(format!("frame {frame} drawn after {}", self.next));
        }
        let rgba = crate::pattern::rgba(self.width, self.height, frame, self.engine.pitch());
        let ready = self.engine.draw(&rgba)?;
        self.next += 1;
        debug_assert_eq!(
            (u64::from(ready.slot), ready.value),
            (frame % POOL as u64, frame + SHARED_PREPARED + 1),
            "the pool counts frames as the producer does"
        );
        Ok(ready.frame(self.image))
    }

    /// The preparation signal ([`SHARED_PREPARED`]) that a window sends
    /// once it has imported the pool, for a pool no window imports (the
    /// read back tests): the engine's first frames wait for it. The pool's
    /// handles are taken, as an import takes them, so the pool is not
    /// shared afterwards. Send it once.
    pub fn signal_prepared(&mut self) -> Result<(), String> {
        let shared = self
            .shared
            .take()
            .ok_or_else(|| "the pool was shared already".to_owned())?;
        self.engine.prepare(&shared.release)
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
