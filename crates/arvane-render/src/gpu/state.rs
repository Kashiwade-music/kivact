//! Role: Stores GPU context, surface and image cache state.

use super::external::External;
use crate::draw;
use crate::images::ImageCache;
use kivact_types::{GpuAdapter, GpuBackend, GpuPower, GpuPreference};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use vello_common::paint::ImageId;
use vello_hybrid::{Renderer, Resources, TextureBindings, TextureId};

/// The format the renderer draws in: the 8-bit unorm format every desktop
/// surface offers (a surface without it gets a copy, `surface.rs`).
pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// The first failure the device reported (validation error or loss).
pub(crate) type Failure = Arc<Mutex<Option<String>>>;

/// How GPU devices are made: as the application wishes
/// ([`GpuPreference`]), except where the environment says otherwise: the
/// backends to try (`WGPU_BACKEND`), the adapter's power preference
/// (`WGPU_POWER_PREF`) and the instance flags wgpu reads from the
/// environment (`WGPU_VALIDATION`, `WGPU_DEBUG` ..). The environment is
/// read once ([`GpuEnv::from_env`]) and the result handed to
/// [`GpuContext::new`].
#[derive(Clone, Copy, Debug)]
pub struct GpuEnv {
    /// `WGPU_BACKEND`.
    pub(crate) backends: Option<wgpu::Backends>,
    /// `WGPU_POWER_PREF`.
    pub(crate) power: Option<wgpu::PowerPreference>,
    pub(crate) flags: wgpu::InstanceFlags,
    pub(crate) preference: GpuPreference,
}

impl GpuEnv {
    /// Read the environment now, for devices made as `preference` wishes.
    pub fn from_env(preference: GpuPreference) -> Self {
        Self {
            backends: wgpu::Backends::from_env(),
            power: wgpu::PowerPreference::from_env(),
            flags: wgpu::InstanceFlags::empty().with_env(),
            preference,
        }
    }

    /// The adapter's power preference: the environment's, else the
    /// application's.
    pub(crate) fn power(&self) -> wgpu::PowerPreference {
        self.power.unwrap_or(match self.preference.power {
            GpuPower::LowPower => wgpu::PowerPreference::LowPower,
            GpuPower::HighPerformance => wgpu::PowerPreference::HighPerformance,
        })
    }

    /// The backends the application asks for, unless the environment
    /// names its own.
    pub(crate) fn backends(&self) -> Option<wgpu::Backends> {
        self.backends
            .or(self.preference.backend.map(|backend| match backend {
                GpuBackend::Dx12 => wgpu::Backends::DX12,
                GpuBackend::Vulkan => wgpu::Backends::VULKAN,
            }))
    }
}

/// GPU rasterizer used serially on its owner's thread: one adapter, device
/// and vello_hybrid renderer.
pub struct GpuContext {
    /// The images external engines share (first: dropped, and their
    /// fences signaled, before the device).
    pub(crate) external: External,
    /// The adapter, when the device can import shared textures.
    pub(crate) interop: Option<GpuAdapter>,
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) renderer: Renderer,
    pub(crate) resources: Resources,
    /// The scene the frames are recorded into, kept for its allocations.
    pub(crate) scene: vello_hybrid::Scene,
    pub(crate) caches: draw::Caches,
    /// The textures of the last frame's images and the atlas slots of its
    /// patterns.
    pub(crate) images: Images,
    pub(crate) failure: Failure,
    /// The offscreen texture of [`GpuContext::render`], reused while the
    /// size holds.
    pub(crate) offscreen: Option<wgpu::Texture>,
}

/// An image's texture.
#[derive(Clone)]
pub(crate) struct GpuImage {
    pub(crate) id: TextureId,
    pub(crate) view: wgpu::TextureView,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

/// A pattern image's atlas slot.
pub(crate) struct AtlasImage {
    /// Keeps the address of the key from being reused.
    pub(crate) _pixels: Arc<[u8]>,
    pub(crate) id: ImageId,
    /// The slot's pixel size (the image's, or smaller when averaged down).
    pub(crate) width: u16,
    pub(crate) height: u16,
    /// The last frame that drew it.
    pub(crate) frame: u64,
}

/// The images the frames draw, and their bindings for the frame being
/// drawn.
pub(crate) struct Images {
    pub(crate) cache: ImageCache<GpuImage>,
    pub(crate) bindings: TextureBindings,
    pub(crate) next: u64,
    /// Pattern images in the atlas, by their pixels' identity and size.
    pub(crate) atlas: HashMap<(usize, u32, u32), AtlasImage>,
}

impl Images {
    pub(crate) fn new() -> Self {
        Self {
            cache: ImageCache::new(),
            bindings: TextureBindings::new(),
            next: 0,
            atlas: HashMap::new(),
        }
    }
}

/// What [`GpuContext::present`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presentation {
    /// The frame is on its way to the screen.
    Shown,
    /// The surface had no buffer to give (timeout, occluded window); the
    /// frame was not shown and should be drawn again later.
    Skipped,
}

/// A window's swapchain, and the texture the renderer draws into when it
/// cannot draw into the swapchain's.
pub struct GpuSurface {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) swapchain: Swapchain,
    /// The frame drawn for a copy to the surface.
    pub(crate) raster: Option<wgpu::Texture>,
    /// Draw every frame into `raster`, for [`GpuContext::read_back`].
    pub(crate) keep: bool,
}

/// Whether a surface's swapchain is configured yet.
pub(crate) enum Swapchain {
    /// Made, not configured yet: a frame or [`GpuContext::configure`]
    /// configures it.
    Unconfigured,
    Configured(Configured),
}

/// A configured swapchain.
pub(crate) struct Configured {
    pub(crate) config: wgpu::SurfaceConfiguration,
    /// The copy to the swapchain's format (fixed by the first configuration).
    pub(crate) blitter: wgpu::util::TextureBlitter,
    /// The last acquire found no buffer (timeout, occluded window): the next
    /// frame takes the buffer before recording.
    pub(crate) skipped: bool,
}
