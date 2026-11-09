//! Role: The GPU context, its environment and the window surfaces.
//!
//! The aggregate of the GPU executor. Its fields are passive state from the
//! leaf modules (targets, pipelines, shader_cache, export_pool, external).

use super::export_pool::Exports;
use super::external::External;
use super::pipelines::Pipelines;
use super::shader_cache::Shaders;
use super::targets::Pool;
use crate::cache::{CacheIndex, ChunkKey};
use arvane_render::GpuRaster;
use kivact_types::cell::Mailbox;
use kivact_types::render::LayerKey;
use kivact_types::{GpuAdapter, GpuBackend, GpuPower, GpuPreference};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// How GPU devices are made: as the application wishes
/// ([`GpuPreference`]), except where the environment says otherwise: the
/// backends to try (`WGPU_BACKEND`), the adapter's power preference
/// (`WGPU_POWER_PREF`) and the instance flags wgpu reads from the
/// environment (`WGPU_VALIDATION`, `WGPU_DEBUG` ..). The environment is
/// read once ([`GpuEnv::from_env`]) and the result handed to
/// [`GpuContext::new`].
#[derive(Clone, Copy, Debug)]
pub struct GpuEnv {
    // xtask-allow(state-fields): option: `WGPU_BACKEND`
    /// `WGPU_BACKEND`.
    pub(crate) backends: Option<wgpu::Backends>,
    // xtask-allow(state-fields): option: `WGPU_POWER_PREF`
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

/// A GPU executor used serially on its owner's thread: one adapter, device,
/// pipelines and the textures of the frames drawn so far.
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
    /// Draws the chunks (Vello's GPU stage).
    pub(crate) raster: GpuRaster,
    pub(crate) pipelines: Pipelines,
    pub(crate) failure: Arc<DeviceLost>,
    /// The previous frame's records; the plan reads it.
    pub(crate) cache: CacheIndex,
    /// Free textures of the targets, reused by size.
    pub(crate) pool: Pool,
    /// The textures of the layer surfaces kept across frames, by layer.
    pub(crate) kept: HashMap<LayerKey, wgpu::Texture>,
    /// The chunk textures, by chunk.
    pub(crate) chunks: HashMap<ChunkKey, wgpu::Texture>,
    /// The user shaders' pipelines.
    pub(crate) shaders: Shaders,
    /// The shader programs that failed since the last frame report: (hash, message).
    pub(crate) shader_failures: Vec<(u64, String)>,
    /// The export pools and what each sink last received.
    pub(crate) exports: Exports,
}

/// The first failure the device reported (validation error or loss).
#[derive(Default)]
pub(crate) struct DeviceLost {
    message: Mailbox<String>,
    /// Serialize the mailbox's read-and-restore operations to retain the first failure.
    access: Mutex<()>,
}

impl DeviceLost {
    /// Keep `message` unless an earlier failure is already recorded.
    pub(crate) fn record(&self, message: String) {
        let _access = self
            .access
            .lock()
            .expect("invariant: device loss lock is not poisoned");
        self.message.put(self.message.take().unwrap_or(message));
    }

    /// The first recorded failure, if any.
    pub(crate) fn message(&self) -> Option<String> {
        let _access = self
            .access
            .lock()
            .expect("invariant: device loss lock is not poisoned");
        let message = self.message.take()?;
        self.message.put(message.clone());
        Some(message)
    }
}

/// Whether a frame's surface is kept for [`GpuContext::read_back`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Retain {
    Drop,
    Keep,
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

/// A window's swapchain and what the executor keeps of its frames.
pub struct GpuSurface {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) swapchain: Swapchain,
    /// Keep each presented frame for [`GpuContext::read_back`].
    pub(crate) keep: Retain,
    /// The last presented frame's output, kept only when `keep` is [`Retain::Keep`].
    pub(crate) last: Option<wgpu::Texture>,
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
}
