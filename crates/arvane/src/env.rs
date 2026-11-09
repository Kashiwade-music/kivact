//! Role: Environment variable configuration for Arvane.
//!
//! The environment variables Arvane reads, all read here: once per native
//! window loop ([`crate::Shell::run`]) or test window, never on a hot
//! path.
//!
//! - `ARVANE_RENDERER`: `cpu` draws a native window on the CPU, `gpu` on
//!   the GPU with the device started with the window, `auto` starts the
//!   device when it is needed, overriding the host's preference. Stepped
//!   shells read it through the same rendering driver.
//! - `WGPU_BACKEND`, `WGPU_POWER_PREF` and the instance flags wgpu reads
//!   (`WGPU_VALIDATION`, `WGPU_DEBUG` ..): how GPU devices are made
//!   (feature `gpu`; read by `arvane-render`, which alone knows wgpu). The
//!   first two override the host's [`kivact_types::GpuPreference`].

use kivact_types::gpu::GpuStart;

/// What `ARVANE_RENDERER` asks of the renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RendererChoice {
    /// `cpu`: frames are drawn on the CPU only.
    Cpu,
    /// `gpu`: the device starts with the window.
    Gpu,
    /// `auto`: the device starts when it is needed.
    Auto,
}

/// When a window starts the GPU renderer: `None` for the CPU only, else
/// `forced` (`ARVANE_RENDERER`) or, without it, the host's preference
/// (`prefer_gpu`) and the device wish's `start`.
pub(crate) fn gpu_start(
    forced: Option<RendererChoice>,
    prefer_gpu: bool,
    start: GpuStart,
) -> Option<GpuStart> {
    match forced {
        Some(RendererChoice::Cpu) => None,
        Some(RendererChoice::Gpu) => Some(GpuStart::AtLaunch),
        Some(RendererChoice::Auto) => Some(GpuStart::OnDemand),
        None => prefer_gpu.then_some(start),
    }
}

/// What the environment asks of Arvane.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Env {
    /// `ARVANE_RENDERER`: the renderer native windows draw with, whatever
    /// the host prefers.
    pub(crate) renderer: Option<RendererChoice>,
    /// How GPU devices are made.
    pub(crate) gpu: arvane_compose::GpuEnv,
}

impl Env {
    /// Read the environment now, for GPU devices made as `preference`
    /// wishes where the environment does not say otherwise.
    pub(crate) fn read(preference: kivact_types::GpuPreference) -> Self {
        Self {
            renderer: match std::env::var("ARVANE_RENDERER").as_deref() {
                Ok("cpu") => Some(RendererChoice::Cpu),
                Ok("gpu") => Some(RendererChoice::Gpu),
                Ok("auto") => Some(RendererChoice::Auto),
                _ => None,
            },
            gpu: arvane_compose::GpuEnv::from_env(preference),
        }
    }
}
